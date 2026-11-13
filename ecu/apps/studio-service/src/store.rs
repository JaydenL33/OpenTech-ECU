//! Local, append-only calibration revision + audit store (TUNE-AUD-01).
//!
//! # What this is — and is not
//!
//! Two JSON-lines files in a data directory:
//!
//! * `revisions.jsonl` — every sealed [`Calibration`] revision the service
//!   creates (plus the baseline it started from), with its payload.
//! * `audit.jsonl` — one entry per calibration-affecting operation, applied
//!   *or rejected*, with before/after checksums, actor label and approval info.
//!
//! Each record carries a `seq`, the previous record's hash and its own SHA-256
//! (`record_hash`), so removal, reordering or editing of a record is detected
//! on load, and every revision payload is re-verified with
//! [`Calibration::verify`].
//!
//! This is **local evidence, not a signed certification**. Anyone with write
//! access to the directory can rebuild a self-consistent store, and removing
//! records from the *end* of a file is not detectable without an external
//! anchor (e.g. a copy of the latest `record_hash` kept elsewhere). The actor
//! label is an unauthenticated string supplied by the caller; if none is
//! supplied it is the explicit value `local-operator`. No human identity is
//! ever inferred.
//!
//! # Durability
//!
//! Every acknowledged append is `fsync`ed. Newly created files are made
//! durable by also `fsync`ing their parent directory; `store.json` is written
//! to a temp file, fsynced, renamed into place and the directory fsynced; newly
//! created data directories are fsynced up the created chain. This is the
//! standard POSIX recipe, **verified only by code-path tests and a SIGKILL
//! process-crash test** — it is not proof against physical power loss (disk
//! write caches and filesystem behaviour are outside what these tests can
//! exercise). On non-Unix targets directory fsync is a no-op.
//!
//! # Path handling limits
//!
//! Environment-provided data directories (`OPENTECH_ECU_DATA_DIR`,
//! `XDG_DATA_HOME`, `HOME`) must be absolute; relative values are refused so the
//! current directory is never used implicitly. An explicit `--data-dir` is
//! taken as given. Files are opened **without** `O_NOFOLLOW`: if the data
//! directory or a file in it is a symlink it is followed. Anyone able to plant
//! symlinks in the data directory can already rewrite the store, which this
//! design does not defend against.
//!
//! The store has no physical-output path and never writes to hardware.

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use ecu_calibration::{hex_sha256, now_unix_ms, Calibration};
use serde::{Deserialize, Serialize};

pub const STORE_FORMAT: &str = "opentech-ecu-calibration-store/1";
pub const DEFAULT_ACTOR: &str = "local-operator";
pub const STORE_NOTICE: &str =
    "Local append-only evidence log. Hash-chained and checksum-verified on load; \
not a signed certification. Actor labels are unauthenticated.";
const META_FILE: &str = "store.json";
const REVISIONS_FILE: &str = "revisions.jsonl";
const AUDIT_FILE: &str = "audit.jsonl";
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const MAX_LABEL_CHARS: usize = 80;

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    /// The store on disk failed verification. The offending file is left
    /// byte-for-byte untouched so it can be inspected.
    Corrupt {
        file: PathBuf,
        line: usize,
        reason: String,
    },
    Invalid(String),
    /// A previous write failed part-way; refuse further writes until restart.
    Poisoned,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(m) => write!(f, "calibration store I/O error: {m}"),
            StoreError::Corrupt { file, line, reason } => write!(
                f,
                "calibration store is corrupt or tampered: {} line {}: {}. \
                 The file has been left untouched for inspection; the service will not serve this store.",
                file.display(),
                line,
                reason
            ),
            StoreError::Invalid(m) => write!(f, "calibration store: {m}"),
            StoreError::Poisoned => write!(
                f,
                "calibration store refused: an earlier write failed part-way; restart to re-verify"
            ),
        }
    }
}

impl std::error::Error for StoreError {}

/// Who performed an operation. `label` is whatever the caller supplied (or the
/// explicit default); it is **not** an authenticated identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActorInfo {
    pub label: String,
    /// `false` when no label was supplied and [`DEFAULT_ACTOR`] was used.
    pub supplied: bool,
}

impl ActorInfo {
    pub fn from_supplied(label: Option<&str>) -> Self {
        let cleaned: String = label
            .unwrap_or("")
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_LABEL_CHARS)
            .collect();
        let cleaned = cleaned.trim().to_string();
        if cleaned.is_empty() {
            Self {
                label: DEFAULT_ACTOR.to_string(),
                supplied: false,
            }
        } else {
            Self {
                label: cleaned,
                supplied: true,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approval {
    pub required: bool,
    pub granted: bool,
    /// How approval was obtained, in plain words.
    pub mechanism: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevisionRecord {
    pub seq: u64,
    pub prev_hash: String,
    pub stored_unix_ms: u64,
    pub revision_id: String,
    pub revision: u32,
    pub checksum: String,
    pub parent_id: Option<String>,
    /// Store-level parent link. `Calibration::id` is *not* unique (re-editing
    /// the same parent after an undo reproduces the id), so lineage is by
    /// checksum.
    pub parent_checksum: Option<String>,
    pub source: String,
    pub actor: ActorInfo,
    pub reason: String,
    pub calibration: Calibration,
    pub record_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub prev_hash: String,
    pub timestamp_unix_ms: u64,
    /// e.g. `baseline`, `table_edit`, `axis_resample`, `mixture_apply`,
    /// `undo`, `store_loaded`.
    pub action: String,
    /// `applied`, `rejected` or `info`.
    pub outcome: String,
    pub actor: ActorInfo,
    pub approval: Approval,
    pub reason: String,
    pub before_checksum: Option<String>,
    /// Active calibration checksum after the operation (== before when rejected).
    pub after_checksum: Option<String>,
    /// Checksum of the newly stored revision, if one was created.
    pub new_revision_checksum: Option<String>,
    pub detail: String,
    pub record_hash: String,
}

/// Caller-supplied fields of an audit entry; the store fills seq/hash/time.
#[derive(Clone, Debug)]
pub struct NewAudit {
    pub action: String,
    pub outcome: String,
    pub actor: ActorInfo,
    pub approval: Approval,
    pub reason: String,
    pub before_checksum: Option<String>,
    pub after_checksum: Option<String>,
    pub new_revision_checksum: Option<String>,
    pub detail: String,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    format: String,
    created_unix_ms: u64,
    notice: String,
}

trait Hashed: Serialize + Clone {
    fn set_hash(&mut self, h: String);
    fn hash(&self) -> &str;
    fn seq(&self) -> u64;
    fn prev_hash(&self) -> &str;
    fn computed_hash(&self) -> String {
        let mut copy = self.clone();
        copy.set_hash(String::new());
        hex_sha256(&serde_json::to_vec(&copy).expect("record serialisation"))
    }
}

macro_rules! hashed {
    ($t:ty) => {
        impl Hashed for $t {
            fn set_hash(&mut self, h: String) {
                self.record_hash = h;
            }
            fn hash(&self) -> &str {
                &self.record_hash
            }
            fn seq(&self) -> u64 {
                self.seq
            }
            fn prev_hash(&self) -> &str {
                &self.prev_hash
            }
        }
    };
}
hashed!(RevisionRecord);
hashed!(AuditEntry);

/// (revisions, audit entries, warnings) as loaded and verified from disk.
type Loaded = (Vec<RevisionRecord>, Vec<AuditEntry>, Vec<String>);

pub struct CalibrationStore {
    dir: PathBuf,
    revisions: Vec<RevisionRecord>,
    audit: Vec<AuditEntry>,
    warnings: Vec<String>,
    poisoned: bool,
}

/// Read a JSONL file, enforcing canonical form and the hash chain.
fn read_chain<T>(path: &Path) -> Result<Vec<T>, StoreError>
where
    T: Hashed + for<'de> Deserialize<'de>,
{
    let corrupt = |line: usize, reason: String| StoreError::Corrupt {
        file: path.to_path_buf(),
        line,
        reason,
    };
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|e| corrupt(0, format!("unreadable (not valid UTF-8 text?): {e}")))?;
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if !text.ends_with('\n') {
        return Err(corrupt(
            text.lines().count(),
            "file does not end with a newline (torn or truncated last record)".into(),
        ));
    }
    let mut out: Vec<T> = Vec::new();
    let mut prev = GENESIS_HASH.to_string();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        let rec: T = serde_json::from_str(line)
            .map_err(|e| corrupt(n, format!("record does not parse: {e}")))?;
        let canonical = serde_json::to_string(&rec).map_err(|e| corrupt(n, e.to_string()))?;
        if canonical != line {
            return Err(corrupt(
                n,
                "record is not in canonical form (edited outside the service)".into(),
            ));
        }
        if rec.computed_hash() != rec.hash() {
            return Err(corrupt(
                n,
                "record_hash does not match record contents".into(),
            ));
        }
        if rec.seq() != i as u64 {
            return Err(corrupt(n, format!("seq {} != expected {}", rec.seq(), i)));
        }
        if rec.prev_hash() != prev {
            return Err(corrupt(
                n,
                "prev_hash does not link to the previous record (record removed, inserted or reordered)"
                    .into(),
            ));
        }
        prev = rec.hash().to_string();
        out.push(rec);
    }
    Ok(out)
}

fn last_hash<T: Hashed>(v: &[T]) -> String {
    v.last()
        .map(|r| r.hash().to_string())
        .unwrap_or_else(|| GENESIS_HASH.to_string())
}

// Test-only record of every directory fsync, so tests can prove the
// durable-creation code path runs (a real power-loss cannot be simulated
// in-process).
#[cfg(test)]
thread_local! {
    pub(crate) static DIR_SYNCS: std::cell::RefCell<Vec<PathBuf>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// fsync a directory so that entries created/renamed inside it survive power
/// loss. On Windows directories cannot be opened for fsync this way; there
/// this is a documented no-op (the store is only exercised on Unix).
fn sync_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    DIR_SYNCS.with(|v| v.borrow_mut().push(dir.to_path_buf()));
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        Ok(())
    }
}

/// The directory that holds `path`'s directory entry.
fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

/// Durably create `path` with `bytes`: write a sibling temp file, fsync it,
/// rename it into place, then fsync the parent directory. A crash leaves
/// either no `path` or the complete file, never a partial one.
fn write_file_durable(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    sync_dir(parent_dir(path))
}

/// `create_dir_all` that also fsyncs every directory it created and the parent
/// that holds each new entry, so the directory chain itself is durable.
fn create_dir_all_durable(dir: &Path) -> std::io::Result<()> {
    let mut missing: Vec<&Path> = Vec::new();
    for a in dir.ancestors() {
        if a.as_os_str().is_empty() || a.exists() {
            break;
        }
        missing.push(a);
    }
    fs::create_dir_all(dir)?;
    // Deepest-first: sync the new directory, then the parent holding its entry.
    for d in missing {
        sync_dir(d)?;
        sync_dir(parent_dir(d))?;
    }
    Ok(())
}

/// Append one line and fsync. When the call creates the file, the parent
/// directory is fsynced too so the directory entry is durable before the
/// write is acknowledged.
fn append_line(path: &Path, line: &str) -> Result<(), StoreError> {
    let ioerr = |e: std::io::Error| StoreError::Io(format!("{}: {e}", path.display()));
    let (mut f, created): (File, bool) =
        match OpenOptions::new().create_new(true).append(true).open(path) {
            Ok(f) => (f, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (
                OpenOptions::new().append(true).open(path).map_err(ioerr)?,
                false,
            ),
            Err(e) => return Err(ioerr(e)),
        };
    f.write_all(line.as_bytes())
        .and_then(|_| f.write_all(b"\n"))
        .and_then(|_| f.sync_all())
        .map_err(ioerr)?;
    if created {
        sync_dir(parent_dir(path)).map_err(ioerr)?;
    }
    Ok(())
}

/// Result of a full verification pass.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyReport {
    pub revisions: usize,
    pub audit_entries: usize,
    pub head_checksum: Option<String>,
    pub revisions_tail_hash: String,
    pub audit_tail_hash: String,
    pub warnings: Vec<String>,
}

impl CalibrationStore {
    /// Open (creating if absent) and fully verify the store in `dir`.
    /// On any verification failure returns [`StoreError::Corrupt`] and leaves
    /// every file untouched.
    pub fn open(dir: &Path) -> Result<Self, StoreError> {
        let (revisions, audit, warnings) = if dir.join(META_FILE).exists() {
            Self::load(dir)?
        } else {
            if dir.join(REVISIONS_FILE).exists() || dir.join(AUDIT_FILE).exists() {
                return Err(StoreError::Corrupt {
                    file: dir.join(META_FILE),
                    line: 0,
                    reason: "store.json is missing but record files exist".into(),
                });
            }
            create_dir_all_durable(dir)
                .map_err(|e| StoreError::Io(format!("{}: {e}", dir.display())))?;
            let meta = Meta {
                format: STORE_FORMAT.into(),
                created_unix_ms: now_unix_ms(),
                notice: STORE_NOTICE.into(),
            };
            let text = serde_json::to_string_pretty(&meta).expect("meta serialisation");
            write_file_durable(&dir.join(META_FILE), (text + "\n").as_bytes())
                .map_err(|e| StoreError::Io(format!("{}: {e}", dir.display())))?;
            (Vec::new(), Vec::new(), Vec::new())
        };
        Ok(Self {
            dir: dir.to_path_buf(),
            revisions,
            audit,
            warnings,
            poisoned: false,
        })
    }

    fn load(dir: &Path) -> Result<Loaded, StoreError> {
        let meta_path = dir.join(META_FILE);
        let meta_err = |reason: String| StoreError::Corrupt {
            file: meta_path.clone(),
            line: 0,
            reason,
        };
        let meta: Meta = serde_json::from_slice(
            &fs::read(&meta_path).map_err(|e| meta_err(format!("unreadable: {e}")))?,
        )
        .map_err(|e| meta_err(format!("does not parse: {e}")))?;
        if meta.format != STORE_FORMAT {
            return Err(meta_err(format!(
                "unsupported store format '{}' (expected '{STORE_FORMAT}')",
                meta.format
            )));
        }

        let rpath = dir.join(REVISIONS_FILE);
        let apath = dir.join(AUDIT_FILE);
        let revisions: Vec<RevisionRecord> = read_chain(&rpath)?;
        let audit: Vec<AuditEntry> = read_chain(&apath)?;
        let rc = |i: usize, reason: String| StoreError::Corrupt {
            file: rpath.clone(),
            line: i + 1,
            reason,
        };
        let ac = |i: usize, reason: String| StoreError::Corrupt {
            file: apath.clone(),
            line: i + 1,
            reason,
        };

        // Revisions: payload integrity + lineage.
        let mut seen: Vec<&str> = Vec::new();
        for (i, r) in revisions.iter().enumerate() {
            let c = &r.calibration;
            if !c.verify() {
                return Err(rc(
                    i,
                    "calibration payload SHA-256 does not match its checksum".into(),
                ));
            }
            if c.checksum != r.checksum || c.id != r.revision_id || c.revision != r.revision {
                return Err(rc(
                    i,
                    "record metadata (checksum/id/revision) disagrees with its payload".into(),
                ));
            }
            if seen.contains(&r.checksum.as_str()) {
                return Err(rc(i, "duplicate revision checksum".into()));
            }
            match &r.parent_checksum {
                None if i != 0 => {
                    return Err(rc(i, "non-baseline revision has no parent_checksum".into()))
                }
                Some(p) if !seen.contains(&p.as_str()) => {
                    return Err(rc(
                        i,
                        format!("parent_checksum {p} does not refer to an earlier revision"),
                    ))
                }
                _ => {}
            }
            seen.push(r.checksum.as_str());
        }

        // Audit: references + state continuity.
        let mut state: Option<String> = None;
        for (i, a) in audit.iter().enumerate() {
            if a.before_checksum != state {
                return Err(ac(
                    i,
                    "before_checksum does not continue from the previous entry's after_checksum"
                        .into(),
                ));
            }
            for c in [
                &a.before_checksum,
                &a.after_checksum,
                &a.new_revision_checksum,
            ]
            .into_iter()
            .flatten()
            {
                if !seen.contains(&c.as_str()) {
                    return Err(ac(i, format!("references unknown revision checksum {c}")));
                }
            }
            if a.after_checksum.is_some() {
                state = a.after_checksum.clone();
            }
        }
        if revisions.is_empty() != audit.is_empty() {
            return Err(StoreError::Corrupt {
                file: if revisions.is_empty() { rpath } else { apath },
                line: 0,
                reason:
                    "revisions and audit trail are inconsistent (one is empty, the other is not)"
                        .into(),
            });
        }

        let mut warnings = Vec::new();
        for r in &revisions {
            let referenced = audit
                .iter()
                .any(|a| a.new_revision_checksum.as_deref() == Some(r.checksum.as_str()));
            if !referenced {
                warnings.push(format!(
                    "revision {} ({}) has no audit entry (interrupted write?)",
                    r.checksum, r.revision_id
                ));
            }
        }
        Ok((revisions, audit, warnings))
    }

    /// Re-read and re-verify the files currently on disk without touching them.
    pub fn verify_on_disk(dir: &Path) -> Result<VerifyReport, StoreError> {
        let (revisions, audit, warnings) = Self::load(dir)?;
        Ok(VerifyReport {
            revisions: revisions.len(),
            audit_entries: audit.len(),
            head_checksum: head_of(&audit),
            revisions_tail_hash: last_hash(&revisions),
            audit_tail_hash: last_hash(&audit),
            warnings,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    pub fn revisions(&self) -> &[RevisionRecord] {
        &self.revisions
    }
    pub fn audit(&self) -> &[AuditEntry] {
        &self.audit
    }
    pub fn revision_by_checksum(&self, checksum: &str) -> Option<&RevisionRecord> {
        self.revisions.iter().find(|r| r.checksum == checksum)
    }
    pub fn revisions_tail_hash(&self) -> String {
        last_hash(&self.revisions)
    }
    pub fn audit_tail_hash(&self) -> String {
        last_hash(&self.audit)
    }

    /// Active calibration checksum: the `after_checksum` of the latest
    /// non-rejected audit entry.
    pub fn head_checksum(&self) -> Option<String> {
        head_of(&self.audit)
    }

    /// Lineage root → head (the in-memory undo stack, reconstructed).
    pub fn active_chain(&self) -> Result<Vec<Calibration>, StoreError> {
        let head = self
            .head_checksum()
            .ok_or_else(|| StoreError::Invalid("store is empty".into()))?;
        let mut chain: Vec<Calibration> = Vec::new();
        let mut cursor = Some(head);
        while let Some(c) = cursor {
            let rec = self
                .revision_by_checksum(&c)
                .ok_or_else(|| StoreError::Invalid(format!("head lineage missing revision {c}")))?;
            chain.push(rec.calibration.clone());
            cursor = rec.parent_checksum.clone();
        }
        chain.reverse();
        Ok(chain)
    }

    fn check_writable(&self) -> Result<(), StoreError> {
        if self.poisoned {
            Err(StoreError::Poisoned)
        } else {
            Ok(())
        }
    }

    /// Append a sealed revision. `cal` must verify; duplicates are refused.
    pub fn append_revision(
        &mut self,
        cal: &Calibration,
        parent_checksum: Option<&str>,
        source: &str,
        actor: &ActorInfo,
        reason: &str,
    ) -> Result<(), StoreError> {
        self.check_writable()?;
        if !cal.verify() {
            return Err(StoreError::Invalid(
                "refusing to store a calibration whose checksum does not verify".into(),
            ));
        }
        if self.revision_by_checksum(&cal.checksum).is_some() {
            return Err(StoreError::Invalid(format!(
                "revision with checksum {} is already stored",
                cal.checksum
            )));
        }
        match (parent_checksum, self.revisions.is_empty()) {
            (None, false) => {
                return Err(StoreError::Invalid(
                    "only the first stored revision may lack a parent".into(),
                ))
            }
            (Some(p), _) if self.revision_by_checksum(p).is_none() => {
                return Err(StoreError::Invalid(format!(
                    "parent revision {p} is not in the store"
                )))
            }
            _ => {}
        }
        let mut rec = RevisionRecord {
            seq: self.revisions.len() as u64,
            prev_hash: last_hash(&self.revisions),
            stored_unix_ms: now_unix_ms(),
            revision_id: cal.id.clone(),
            revision: cal.revision,
            checksum: cal.checksum.clone(),
            parent_id: cal.parent.clone(),
            parent_checksum: parent_checksum.map(str::to_string),
            source: source.to_string(),
            actor: actor.clone(),
            reason: reason.to_string(),
            calibration: cal.clone(),
            record_hash: String::new(),
        };
        rec.record_hash = rec.computed_hash();
        let line = serde_json::to_string(&rec).map_err(|e| StoreError::Io(e.to_string()))?;
        if let Err(e) = append_line(&self.dir.join(REVISIONS_FILE), &line) {
            self.poisoned = true;
            return Err(e);
        }
        self.revisions.push(rec);
        Ok(())
    }

    /// Append an audit entry. Referenced checksums must exist in the store.
    pub fn append_audit(&mut self, new: NewAudit) -> Result<(), StoreError> {
        self.check_writable()?;
        for c in [
            &new.before_checksum,
            &new.after_checksum,
            &new.new_revision_checksum,
        ]
        .into_iter()
        .flatten()
        {
            if self.revision_by_checksum(c).is_none() {
                return Err(StoreError::Invalid(format!(
                    "audit entry references unknown revision {c}"
                )));
            }
        }
        if new.before_checksum != self.head_checksum() {
            return Err(StoreError::Invalid(
                "audit entry before_checksum is not the store's current head".into(),
            ));
        }
        let mut entry = AuditEntry {
            seq: self.audit.len() as u64,
            prev_hash: last_hash(&self.audit),
            timestamp_unix_ms: now_unix_ms(),
            action: new.action,
            outcome: new.outcome,
            actor: new.actor,
            approval: new.approval,
            reason: new.reason,
            before_checksum: new.before_checksum,
            after_checksum: new.after_checksum,
            new_revision_checksum: new.new_revision_checksum,
            detail: new.detail,
            record_hash: String::new(),
        };
        entry.record_hash = entry.computed_hash();
        let line = serde_json::to_string(&entry).map_err(|e| StoreError::Io(e.to_string()))?;
        if let Err(e) = append_line(&self.dir.join(AUDIT_FILE), &line) {
            self.poisoned = true;
            return Err(e);
        }
        self.audit.push(entry);
        Ok(())
    }
}

fn head_of(audit: &[AuditEntry]) -> Option<String> {
    audit
        .iter()
        .rev()
        .find(|a| a.outcome != "rejected" && a.after_checksum.is_some())
        .and_then(|a| a.after_checksum.clone())
}

/// Resolve the data directory: explicit flag > `OPENTECH_ECU_DATA_DIR` > the
/// per-user data dir. Never falls back to the current directory, so a stray
/// run cannot write into the repository checkout.
pub fn resolve_data_dir(flag: Option<PathBuf>) -> Result<PathBuf, StoreError> {
    resolve_data_dir_from(flag, |k| std::env::var_os(k))
}

/// An environment-provided location must be absolute: a relative value would
/// silently resolve against the current directory.
fn absolute_env(name: &str, value: std::ffi::OsString) -> Result<PathBuf, StoreError> {
    let p = PathBuf::from(value);
    if p.is_absolute() {
        Ok(p)
    } else {
        Err(StoreError::Invalid(format!(
            "{name} must be an absolute path (got {:?}); relative paths would resolve against the \
             current directory. Fix or unset it, or pass --data-dir",
            p
        )))
    }
}

fn resolve_data_dir_from(
    flag: Option<PathBuf>,
    env: impl Fn(&str) -> Option<std::ffi::OsString>,
) -> Result<PathBuf, StoreError> {
    if let Some(p) = flag {
        return Ok(p);
    }
    if let Some(v) = env("OPENTECH_ECU_DATA_DIR").filter(|v| !v.is_empty()) {
        return absolute_env("OPENTECH_ECU_DATA_DIR", v);
    }
    let home = |rel: &str| -> Result<Option<PathBuf>, StoreError> {
        env("HOME")
            .filter(|v| !v.is_empty())
            .map(|h| absolute_env("HOME", h).map(|h| h.join(rel)))
            .transpose()
    };
    let base = if cfg!(target_os = "windows") {
        env("APPDATA")
            .filter(|v| !v.is_empty())
            .map(|v| absolute_env("APPDATA", v))
            .transpose()?
    } else if cfg!(target_os = "macos") {
        home("Library/Application Support")?
    } else {
        match env("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
            Some(v) => Some(absolute_env("XDG_DATA_HOME", v)?),
            None => home(".local/share")?,
        }
    };
    base.map(|b| b.join("opentech-ecu").join("studio"))
        .ok_or_else(|| {
            StoreError::Invalid(
                "cannot determine a user data directory; pass --data-dir or set OPENTECH_ECU_DATA_DIR"
                    .into(),
            )
        })
}

#[cfg(test)]
pub(crate) mod testutil {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Isolated temp directory removed on drop. Tests never write into the repo.
    pub struct TempDir(PathBuf);
    impl TempDir {
        pub fn new(tag: &str) -> Self {
            static N: AtomicU64 = AtomicU64::new(0);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let p = std::env::temp_dir().join(format!(
                "opentech-ecu-store-test-{tag}-{}-{nanos}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&p).expect("temp dir");
            Self(p)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::TempDir;
    use super::*;
    use ecu_calibration::wizard::{generate_starter, WizardRequest};

    fn base() -> Calibration {
        generate_starter(&WizardRequest::default_inline4(), 1).unwrap()
    }

    fn actor() -> ActorInfo {
        ActorInfo::from_supplied(Some("unit-test"))
    }

    fn approval() -> Approval {
        Approval {
            required: false,
            granted: false,
            mechanism: "test".into(),
        }
    }

    fn audit(
        action: &str,
        before: Option<&Calibration>,
        after: &Calibration,
        new: bool,
    ) -> NewAudit {
        NewAudit {
            action: action.into(),
            outcome: "applied".into(),
            actor: actor(),
            approval: approval(),
            reason: "unit test".into(),
            before_checksum: before.map(|c| c.checksum.clone()),
            after_checksum: Some(after.checksum.clone()),
            new_revision_checksum: new.then(|| after.checksum.clone()),
            detail: String::new(),
        }
    }

    /// baseline -> child1 -> child2 on disk; returns (dir, [c0, c1, c2]).
    fn populated(tag: &str) -> (TempDir, Vec<Calibration>) {
        let dir = TempDir::new(tag);
        let mut s = CalibrationStore::open(dir.path()).unwrap();
        let c0 = base();
        s.append_revision(&c0, None, "baseline", &actor(), "start")
            .unwrap();
        s.append_audit(audit("baseline", None, &c0, true)).unwrap();
        let c1 = c0.fork(c0.revision + 1, 1_000);
        s.append_revision(&c1, Some(&c0.checksum), "table_edit", &actor(), "r1")
            .unwrap();
        s.append_audit(audit("table_edit", Some(&c0), &c1, true))
            .unwrap();
        let c2 = c1.fork(c1.revision + 1, 2_000);
        s.append_revision(&c2, Some(&c1.checksum), "table_edit", &actor(), "r2")
            .unwrap();
        s.append_audit(audit("table_edit", Some(&c1), &c2, true))
            .unwrap();
        (dir, vec![c0, c1, c2])
    }

    #[test]
    fn restart_recovery_reloads_latest_revision_and_lineage() {
        let (dir, cals) = populated("restart");
        let reopened = CalibrationStore::open(dir.path()).unwrap();
        assert_eq!(reopened.revisions().len(), 3);
        assert_eq!(reopened.audit().len(), 3);
        assert_eq!(
            reopened.head_checksum().as_deref(),
            Some(cals[2].checksum.as_str())
        );
        let chain = reopened.active_chain().unwrap();
        assert_eq!(chain, cals, "lineage root->head must round-trip byte-exact");
        assert!(chain.iter().all(Calibration::verify));
        eprintln!(
            "restart-recovery: reloaded {} revisions, head={} lineage_depth={}",
            reopened.revisions().len(),
            reopened.head_checksum().unwrap(),
            chain.len()
        );
    }

    #[test]
    fn append_only_ordering_chain_and_prefix_preserved() {
        let (dir, cals) = populated("ordering");
        let rpath = dir.path().join(REVISIONS_FILE);
        let apath = dir.path().join(AUDIT_FILE);
        let r_before = fs::read(&rpath).unwrap();
        let a_before = fs::read(&apath).unwrap();

        let mut s = CalibrationStore::open(dir.path()).unwrap();
        let c3 = cals[2].fork(4, 3_000);
        s.append_revision(
            &c3,
            Some(&cals[2].checksum),
            "mixture_apply",
            &actor(),
            "r3",
        )
        .unwrap();
        s.append_audit(audit("mixture_apply", Some(&cals[2]), &c3, true))
            .unwrap();

        // Existing bytes are an exact prefix: nothing was rewritten.
        assert!(fs::read(&rpath).unwrap().starts_with(&r_before));
        assert!(fs::read(&apath).unwrap().starts_with(&a_before));
        let seqs: Vec<u64> = s.revisions().iter().map(|r| r.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2, 3]);
        let aseqs: Vec<u64> = s.audit().iter().map(|a| a.seq).collect();
        assert_eq!(aseqs, vec![0, 1, 2, 3]);
        for w in s.revisions().windows(2) {
            assert_eq!(w[1].prev_hash, w[0].record_hash);
            assert!(w[1].stored_unix_ms >= w[0].stored_unix_ms);
        }
        for w in s.audit().windows(2) {
            assert_eq!(w[1].prev_hash, w[0].record_hash);
            assert_eq!(w[1].before_checksum, w[0].after_checksum);
        }
        // Duplicates and orphan parents are refused, nothing is written.
        let len = fs::read(&rpath).unwrap().len();
        assert!(s
            .append_revision(&c3, Some(&cals[2].checksum), "x", &actor(), "dup")
            .is_err());
        let orphan = cals[0].fork(9, 9_000);
        assert!(s
            .append_revision(&orphan, Some("deadbeef"), "x", &actor(), "orphan")
            .is_err());
        assert_eq!(fs::read(&rpath).unwrap().len(), len);
        // A calibration with a broken checksum is refused.
        let mut bad = cals[2].fork(5, 5_000);
        bad.notes.push("edited after seal".into());
        assert!(s
            .append_revision(&bad, Some(&cals[2].checksum), "x", &actor(), "bad")
            .is_err());
    }

    #[test]
    fn checksum_verification_every_stored_payload() {
        let (dir, _) = populated("checksums");
        let report = CalibrationStore::verify_on_disk(dir.path()).unwrap();
        assert_eq!(report.revisions, 3);
        assert_eq!(report.audit_entries, 3);
        let s = CalibrationStore::open(dir.path()).unwrap();
        for r in s.revisions() {
            assert!(r.calibration.verify());
            assert_eq!(r.calibration.checksum, r.checksum);
            assert_eq!(r.computed_hash(), r.record_hash);
        }
    }

    fn assert_refused_and_untouched(dir: &TempDir, expect_in: &str, expect_reason: &str) {
        let before: Vec<(String, Vec<u8>)> = [META_FILE, REVISIONS_FILE, AUDIT_FILE]
            .iter()
            .map(|f| (f.to_string(), fs::read(dir.path().join(f)).unwrap()))
            .collect();
        let err = match CalibrationStore::open(dir.path()) {
            Ok(_) => panic!("tampered store must be refused"),
            Err(e) => e,
        };
        let text = err.to_string();
        eprintln!("tamper-refusal[{expect_in}]: {text}");
        match &err {
            StoreError::Corrupt { file, reason, .. } => {
                assert!(file.ends_with(expect_in), "{file:?} vs {expect_in}");
                assert!(reason.contains(expect_reason), "{reason}");
            }
            other => panic!("expected Corrupt, got {other:?}"),
        }
        assert!(text.contains("left untouched"));
        for (f, bytes) in before {
            assert_eq!(
                fs::read(dir.path().join(&f)).unwrap(),
                bytes,
                "{f} must be preserved byte-for-byte"
            );
        }
    }

    #[test]
    fn tamper_edited_payload_is_refused_and_file_preserved() {
        let (dir, _) = populated("tamper-edit");
        let p = dir.path().join(REVISIONS_FILE);
        let text = fs::read_to_string(&p).unwrap();
        // Flip a stored value inside the 2nd record without fixing any hash.
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        lines[1] = lines[1].replacen("\"revision\":2", "\"revision\":7", 1);
        fs::write(&p, lines.join("\n") + "\n").unwrap();
        assert_refused_and_untouched(&dir, REVISIONS_FILE, "record_hash");
    }

    #[test]
    fn tamper_payload_with_recomputed_record_hash_still_fails_calibration_checksum() {
        let (dir, _) = populated("tamper-payload");
        let p = dir.path().join(REVISIONS_FILE);
        let text = fs::read_to_string(&p).unwrap();
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        // An attacker edits the payload AND repairs the record hash (but not the
        // chain). Re-link the chain too so only the SHA-256 payload check can catch it.
        let mut recs: Vec<RevisionRecord> = lines
            .iter()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        recs[1].calibration.notes.push("silently edited".into());
        let mut prev = recs[0].record_hash.clone();
        for r in recs.iter_mut().skip(1) {
            r.prev_hash = prev.clone();
            r.record_hash = r.computed_hash();
            prev = r.record_hash.clone();
        }
        lines = recs
            .iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect();
        fs::write(&p, lines.join("\n") + "\n").unwrap();
        assert_refused_and_untouched(&dir, REVISIONS_FILE, "payload SHA-256");
    }

    #[test]
    fn tamper_removed_record_truncation_and_audit_edit_are_refused() {
        // Middle record removed from audit.
        let (dir, _) = populated("tamper-remove");
        let p = dir.path().join(AUDIT_FILE);
        let text = fs::read_to_string(&p).unwrap();
        let kept: Vec<&str> = text
            .lines()
            .enumerate()
            .filter(|(i, _)| *i != 1)
            .map(|(_, l)| l)
            .collect();
        fs::write(&p, kept.join("\n") + "\n").unwrap();
        assert_refused_and_untouched(&dir, AUDIT_FILE, "seq");

        // Torn / truncated last line.
        let (dir, _) = populated("tamper-torn");
        let p = dir.path().join(REVISIONS_FILE);
        let bytes = fs::read(&p).unwrap();
        fs::write(&p, &bytes[..bytes.len() - 20]).unwrap();
        assert_refused_and_untouched(&dir, REVISIONS_FILE, "newline");

        // Audit entry edited (actor rewritten) without fixing its hash.
        let (dir, _) = populated("tamper-audit");
        let p = dir.path().join(AUDIT_FILE);
        let text = fs::read_to_string(&p)
            .unwrap()
            .replacen("unit-test", "someone-else", 1);
        fs::write(&p, text).unwrap();
        assert_refused_and_untouched(&dir, AUDIT_FILE, "record_hash");

        // Non-canonical rewrite (whitespace only) is still flagged.
        let (dir, _) = populated("tamper-ws");
        let p = dir.path().join(AUDIT_FILE);
        let text = fs::read_to_string(&p)
            .unwrap()
            .replacen("\"seq\":0,", "\"seq\": 0,", 1);
        fs::write(&p, text).unwrap();
        assert_refused_and_untouched(&dir, AUDIT_FILE, "canonical");
    }

    #[test]
    fn missing_meta_or_one_sided_store_is_refused() {
        let (dir, _) = populated("meta");
        fs::remove_file(dir.path().join(META_FILE)).unwrap();
        assert!(matches!(
            CalibrationStore::open(dir.path()),
            Err(StoreError::Corrupt { .. })
        ));
        let (dir, _) = populated("onesided");
        fs::remove_file(dir.path().join(AUDIT_FILE)).unwrap();
        assert!(matches!(
            CalibrationStore::open(dir.path()),
            Err(StoreError::Corrupt { .. })
        ));
    }

    #[test]
    fn actor_label_is_never_invented() {
        let a = ActorInfo::from_supplied(None);
        assert_eq!((a.label.as_str(), a.supplied), (DEFAULT_ACTOR, false));
        let a = ActorInfo::from_supplied(Some("  \n\t "));
        assert_eq!((a.label.as_str(), a.supplied), (DEFAULT_ACTOR, false));
        let a = ActorInfo::from_supplied(Some("bench\x07-op"));
        assert_eq!((a.label.as_str(), a.supplied), ("bench-op", true));
        let long = "x".repeat(500);
        assert_eq!(
            ActorInfo::from_supplied(Some(&long)).label.len(),
            MAX_LABEL_CHARS
        );
    }

    #[test]
    fn data_dir_flag_wins_and_is_never_cwd() {
        let p = resolve_data_dir(Some(PathBuf::from("/explicit/dir"))).unwrap();
        assert_eq!(p, PathBuf::from("/explicit/dir"));
        // Without a flag the default is absolute (env or per-user dir), or an error.
        if let Ok(p) = resolve_data_dir(None) {
            assert!(p.is_absolute(), "{p:?}");
        }
    }

    #[test]
    fn creation_fsyncs_metadata_and_parent_directories() {
        let root = TempDir::new("durable-create");
        let dir = root.path().join("a").join("b").join("store");
        DIR_SYNCS.with(|v| v.borrow_mut().clear());
        let mut s = CalibrationStore::open(&dir).unwrap();
        let syncs = || DIR_SYNCS.with(|v| v.borrow().clone());
        // created chain a, a/b, a/b/store plus their parents, plus the meta rename
        for d in [
            root.path(),
            &root.path().join("a"),
            &root.path().join("a/b"),
            &dir,
        ] {
            assert!(
                syncs().iter().any(|x| x == d),
                "dir {d:?} not fsynced: {:?}",
                syncs()
            );
        }
        assert!(dir.join(META_FILE).exists());
        assert!(
            !dir.join("store.json.tmp").exists(),
            "temp file must be renamed away"
        );
        let after_open = syncs().len();
        let c0 = base();
        s.append_revision(&c0, None, "baseline", &actor(), "start")
            .unwrap();
        assert_eq!(
            syncs().len(),
            after_open + 1,
            "creating revisions.jsonl fsyncs the dir"
        );
        s.append_audit(audit("baseline", None, &c0, true)).unwrap();
        assert_eq!(
            syncs().len(),
            after_open + 2,
            "creating audit.jsonl fsyncs the dir"
        );
        let c1 = c0.fork(c0.revision + 1, 1_000);
        s.append_revision(&c1, Some(&c0.checksum), "table_edit", &actor(), "e")
            .unwrap();
        s.append_audit(audit("table_edit", Some(&c0), &c1, true))
            .unwrap();
        assert_eq!(
            syncs().len(),
            after_open + 2,
            "plain appends do not re-sync the dir"
        );
        assert!(syncs().iter().filter(|x| **x == dir).count() >= 3);
    }

    #[test]
    fn meta_write_leaves_no_partial_file_and_stale_temp_is_overwritten() {
        let dir = TempDir::new("meta-tmp");
        // A crash between temp write and rename leaves only the temp file.
        fs::write(dir.path().join("store.json.tmp"), b"{ partial").unwrap();
        let s = CalibrationStore::open(dir.path()).unwrap();
        assert!(s.is_empty());
        assert!(!dir.path().join("store.json.tmp").exists());
        let text = fs::read_to_string(dir.path().join(META_FILE)).unwrap();
        assert!(text.contains(STORE_FORMAT), "{text}");
        drop(s);
        assert!(
            CalibrationStore::open(dir.path()).is_ok(),
            "reopens cleanly"
        );
    }

    /// Process-crash test (NOT a power-loss test): a child process acknowledges
    /// a durable write and is then SIGKILLed with no chance to flush or run
    /// destructors; the parent must reload a verified store containing it.
    /// This proves nothing about physical power loss.
    #[cfg(unix)]
    #[test]
    fn process_crash_sigkill_after_ack_store_reloads_verified() {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        const ENV: &str = "OPENTECH_STORE_CRASH_CHILD_DIR";
        if let Some(d) = std::env::var_os(ENV) {
            // ---- child mode ----
            let mut s = CalibrationStore::open(Path::new(&d)).unwrap();
            let c0 = base();
            s.append_revision(&c0, None, "baseline", &actor(), "start")
                .unwrap();
            s.append_audit(audit("baseline", None, &c0, true)).unwrap();
            let c1 = c0.fork(c0.revision + 1, 1_000);
            s.append_revision(&c1, Some(&c0.checksum), "table_edit", &actor(), "e")
                .unwrap();
            s.append_audit(audit("table_edit", Some(&c0), &c1, true))
                .unwrap();
            println!("ACKED {}", c1.checksum);
            std::thread::sleep(std::time::Duration::from_secs(120));
            return;
        }
        let dir = TempDir::new("sigkill");
        let target = dir.path().join("fresh-store");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "store::tests::process_crash_sigkill_after_ack_store_reloads_verified",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(ENV, &target)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut acked = None;
        for line in BufReader::new(child.stdout.take().unwrap()).lines() {
            let line = line.unwrap();
            // libtest prefixes the line with "test <name> ... "
            if let Some(i) = line.find("ACKED ") {
                acked = Some(line[i + 6..].trim().to_string());
                break;
            }
        }
        let head = acked.expect("child never acknowledged its writes");
        child.kill().unwrap(); // SIGKILL
        let status = child.wait().unwrap();
        assert!(!status.success(), "child must have been killed: {status:?}");
        let s = CalibrationStore::open(&target).expect("store reloads after SIGKILL");
        assert_eq!(s.revisions().len(), 2);
        assert_eq!(s.audit().len(), 2);
        assert_eq!(s.head_checksum().as_deref(), Some(head.as_str()));
        assert!(s.warnings().is_empty(), "{:?}", s.warnings());
        eprintln!("process-crash(SIGKILL) test: reloaded head {head:.12}; NOT a power-loss test");
    }

    #[test]
    fn relative_env_data_dirs_are_refused_not_resolved_against_cwd() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| std::ffi::OsString::from(*v))
            }
        };
        let e =
            resolve_data_dir_from(None, env(&[("OPENTECH_ECU_DATA_DIR", "rel/dir")])).unwrap_err();
        assert!(
            e.to_string()
                .contains("OPENTECH_ECU_DATA_DIR must be an absolute path"),
            "{e}"
        );
        if cfg!(not(any(target_os = "windows", target_os = "macos"))) {
            let e = resolve_data_dir_from(
                None,
                env(&[("XDG_DATA_HOME", "./data"), ("HOME", "/home/u")]),
            )
            .unwrap_err();
            assert!(
                e.to_string()
                    .contains("XDG_DATA_HOME must be an absolute path"),
                "{e}"
            );
            let e = resolve_data_dir_from(None, env(&[("HOME", "relhome")])).unwrap_err();
            assert!(
                e.to_string().contains("HOME must be an absolute path"),
                "{e}"
            );
            // absolute values are accepted
            assert_eq!(
                resolve_data_dir_from(None, env(&[("XDG_DATA_HOME", "/x/data")])).unwrap(),
                PathBuf::from("/x/data/opentech-ecu/studio")
            );
            assert_eq!(
                resolve_data_dir_from(None, env(&[("HOME", "/home/u")])).unwrap(),
                PathBuf::from("/home/u/.local/share/opentech-ecu/studio")
            );
        }
        assert_eq!(
            resolve_data_dir_from(None, env(&[("OPENTECH_ECU_DATA_DIR", "/abs/d")])).unwrap(),
            PathBuf::from("/abs/d")
        );
        // flag still wins over a bad env value
        assert_eq!(
            resolve_data_dir_from(
                Some("/flag".into()),
                env(&[("OPENTECH_ECU_DATA_DIR", "rel")])
            )
            .unwrap(),
            PathBuf::from("/flag")
        );
    }
}
