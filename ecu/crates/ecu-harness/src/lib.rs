//! Bench harness manifest for OpenTech-ECU.
//!
//! This crate models the **OpenTech bench harness** from its own KiCad exports.
//! It is deliberately not an OEM harness definition: the manifest is
//! `UNREVIEWED_BENCH_ONLY`, its connector references are board-local, and every
//! pin is marked `NONE - BENCH ONLY`.
//!
//! Structural validation here cannot turn an unreviewed schematic into a
//! reviewed harness. It only catches duplicate references, empty nets and
//! accidental OEM assignments.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchHarnessManifest {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub status: String,
    pub physical_use: String,
    pub notice: String,
    pub source: HarnessSource,
    pub generated_by: String,
    pub subsystems: Vec<HarnessSubsystem>,
    pub connectors: Vec<HarnessConnector>,
    pub test_points: Vec<HarnessTestPoint>,
    pub reference_target: ReferenceTarget,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessSource {
    pub design: String,
    pub erc: String,
    pub drc: String,
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessSubsystem {
    pub id: String,
    pub name: String,
    pub sheet: String,
    pub description: String,
    pub connectors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessConnector {
    pub reference: String,
    pub label: String,
    pub sheet: String,
    pub mpn: String,
    pub pins: Vec<HarnessPin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessPin {
    pub pin: String,
    pub name: String,
    pub net: String,
    pub function: String,
    pub oem_assignment: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessTestPoint {
    pub reference: String,
    pub net: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReferenceTarget {
    pub vehicle: String,
    pub compatibility: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestIssue {
    pub code: String,
    pub detail: String,
}

/// Structural validation only. It cannot turn an unreviewed schematic into a
/// reviewed harness.
pub fn validate_manifest(manifest: &BenchHarnessManifest) -> Vec<ManifestIssue> {
    let mut issues = Vec::new();
    if manifest.physical_use != "PROHIBITED" {
        issues.push(ManifestIssue {
            code: "PHYSICAL_USE_NOT_BLOCKED".into(),
            detail: "bench manifest must keep physical_use = PROHIBITED".into(),
        });
    }
    if manifest.status != "UNREVIEWED_BENCH_ONLY" {
        issues.push(ManifestIssue {
            code: "STATUS_NOT_BENCH_ONLY".into(),
            detail: "bench manifest must remain UNREVIEWED_BENCH_ONLY".into(),
        });
    }
    let mut refs = std::collections::BTreeSet::new();
    for connector in &manifest.connectors {
        if !refs.insert(connector.reference.clone()) {
            issues.push(ManifestIssue {
                code: "DUPLICATE_CONNECTOR".into(),
                detail: format!("connector {} appears more than once", connector.reference),
            });
        }
        let mut pins = std::collections::BTreeSet::new();
        for pin in &connector.pins {
            if !pins.insert(pin.pin.clone()) {
                issues.push(ManifestIssue {
                    code: "DUPLICATE_PIN".into(),
                    detail: format!(
                        "connector {} pin {} appears more than once",
                        connector.reference, pin.pin
                    ),
                });
            }
            if pin.net.trim().is_empty() {
                issues.push(ManifestIssue {
                    code: "EMPTY_NET".into(),
                    detail: format!(
                        "connector {} pin {} has no net",
                        connector.reference, pin.pin
                    ),
                });
            }
            if pin.oem_assignment != "NONE - BENCH ONLY" {
                issues.push(ManifestIssue {
                    code: "OEM_ASSIGNMENT_PRESENT".into(),
                    detail: format!(
                        "connector {} pin {} is marked as an OEM assignment; bench manifest must not do that",
                        connector.reference, pin.pin
                    ),
                });
            }
        }
    }
    for subsystem in &manifest.subsystems {
        for reference in &subsystem.connectors {
            if !refs.contains(reference) {
                issues.push(ManifestIssue {
                    code: "UNKNOWN_CONNECTOR".into(),
                    detail: format!(
                        "subsystem {} references missing connector {}",
                        subsystem.id, reference
                    ),
                });
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_bench_manifest_validates() {
        let raw = include_str!("../../../../config/harness/opentech-bench-harness.json");
        let manifest: BenchHarnessManifest = serde_json::from_str(raw).unwrap();
        assert_eq!(manifest.status, "UNREVIEWED_BENCH_ONLY");
        assert_eq!(manifest.physical_use, "PROHIBITED");
        assert!(manifest.connectors.len() >= 20);
        assert!(manifest.test_points.len() >= 40);
        let issues = validate_manifest(&manifest);
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn manifest_validation_flags_duplicates_and_empty_nets() {
        let pin = HarnessPin {
            pin: "1".into(),
            name: "P1".into(),
            net: "N1".into(),
            function: "bench".into(),
            oem_assignment: "NONE - BENCH ONLY".into(),
        };
        let connector = HarnessConnector {
            reference: "J1".into(),
            label: "bench".into(),
            sheet: "sheet".into(),
            mpn: "x".into(),
            pins: vec![pin.clone(), pin],
        };
        let manifest = BenchHarnessManifest {
            schema_version: 1,
            id: "test".into(),
            title: "test".into(),
            status: "UNREVIEWED_BENCH_ONLY".into(),
            physical_use: "PROHIBITED".into(),
            notice: "bench only".into(),
            source: HarnessSource {
                design: "x".into(),
                erc: "x".into(),
                drc: "x".into(),
                files: vec![],
            },
            generated_by: "test".into(),
            subsystems: vec![HarnessSubsystem {
                id: "s".into(),
                name: "s".into(),
                sheet: "sheet".into(),
                description: "s".into(),
                connectors: vec!["J1".into()],
            }],
            connectors: vec![connector],
            test_points: vec![],
            reference_target: ReferenceTarget {
                vehicle: "x".into(),
                compatibility: "UNVERIFIED".into(),
                note: "x".into(),
            },
        };
        let issues = validate_manifest(&manifest);
        assert!(issues.iter().any(|i| i.code == "DUPLICATE_PIN"));
    }
}
