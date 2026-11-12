use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// One simulated injector channel and one simulated coil channel per cylinder (D4).
pub const MAX_SEQUENTIAL_CYLINDERS: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aspiration {
    NaturallyAspirated,
    Turbocharged,
    Supercharged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerPattern {
    /// A single crank wheel with regularly spaced teeth and one missing-tooth
    /// gap. `missing` teeth are absent; a 36-1 wheel has 36 nominal positions.
    MissingTooth { teeth: u16, missing: u16 },
    /// Crank and cam signals supplied independently (not implemented in v1).
    DualWheel { crank_teeth: u16, cam_teeth: u16 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub pattern: TriggerPattern,
    /// Sequential operation requires a cam phase reference. Without it the
    /// simulator can only run wasted-spark / batch modes; v1 requires cam.
    pub cam_required: bool,
    /// Decoder angle assigned at the first tooth after the gap. This is a
    /// calibration parameter, not a physical TDC claim.
    pub sync_offset_deg: f32,
    /// Maximum time without a tooth before sync is declared lost.
    pub tooth_timeout_ms: u16,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            pattern: TriggerPattern::MissingTooth {
                teeth: 36,
                missing: 1,
            },
            cam_required: true,
            sync_offset_deg: 0.0,
            tooth_timeout_ms: 250,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InjectorConfig {
    pub id: String,
    /// Rated flow in cc/min at `rated_pressure_kpa`, using the declared fuel
    /// density. Flow is not inferred from a part number.
    pub flow_cc_min: f32,
    pub rated_pressure_kpa: f32,
    /// Dead time at 13.8 V. Other voltages interpolate from `dead_time_curve`.
    pub dead_time_ms: f32,
    /// (volts, dead_time_ms) points, strictly increasing voltage.
    pub dead_time_curve: Vec<(f32, f32)>,
    pub min_pw_ms: f32,
    pub max_duty: f32,
    /// Cylinders (1-based) fitted with this injector specification, one physical
    /// injector per listed cylinder. Sequential profiles list every cylinder
    /// exactly once (`INJ_COVERAGE`).
    pub cylinders: Vec<u8>,
}

impl InjectorConfig {
    pub fn dead_time_at(&self, volts: f32) -> f32 {
        if self.dead_time_curve.len() < 2 {
            return self.dead_time_ms;
        }
        let mut pts = self.dead_time_curve.clone();
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        if volts <= pts[0].0 {
            return pts[0].1;
        }
        if volts >= pts[pts.len() - 1].0 {
            return pts[pts.len() - 1].1;
        }
        for w in pts.windows(2) {
            if volts >= w[0].0 && volts <= w[1].0 {
                let t = (volts - w[0].0) / (w[1].0 - w[0].0);
                return w[0].1 + t * (w[1].1 - w[0].1);
            }
        }
        self.dead_time_ms
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadStrategy {
    SpeedDensity,
    AlphaN,
    /// `alpha` is the TPS weight (0 = pure speed density, 1 = pure alpha-N).
    Blended {
        alpha_percent: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelConfig {
    pub stoich_afr: f32,
    pub fuel_density_g_cc: f32,
    pub lower_heating_value_mj_kg: f32,
    pub pressure_reference_kpa: f32,
    pub load_strategy: LoadStrategy,
    pub injection_mode: InjectionMode,
    /// When true, injector flow scales with sqrt(actual/rated pressure).
    pub pressure_compensation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMode {
    Sequential,
    SemiSequential,
    Batch,
    Staged,
}

impl Default for FuelConfig {
    fn default() -> Self {
        Self {
            stoich_afr: 14.7,
            fuel_density_g_cc: 0.745,
            lower_heating_value_mj_kg: 43.5,
            pressure_reference_kpa: 300.0,
            load_strategy: LoadStrategy::SpeedDensity,
            injection_mode: InjectionMode::Sequential,
            pressure_compensation: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensorSetup {
    pub map_min_kpa: f32,
    pub map_max_kpa: f32,
    pub map_calibrated: bool,
    pub clt_calibrated: bool,
    pub iat_calibrated: bool,
    pub wideband_banks: u8,
    pub lambda_can_ids: Vec<u32>,
    pub oil_pressure_configured: bool,
    pub fuel_pressure_configured: bool,
}

impl Default for SensorSetup {
    fn default() -> Self {
        Self {
            map_min_kpa: 10.0,
            map_max_kpa: 300.0,
            map_calibrated: false,
            clt_calibrated: false,
            iat_calibrated: false,
            wideband_banks: 1,
            lambda_can_ids: vec![0x1F0],
            oil_pressure_configured: false,
            fuel_pressure_configured: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IoAssignment {
    pub function: String,
    pub pin: String,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct IoConfig {
    pub assignments: Vec<IoAssignment>,
    pub notes: String,
}

/// Derived sequential assignment for one cylinder (D1-D3). Never stored in a calibration.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CylinderAssignment {
    /// 1-based cylinder number
    pub cylinder: u8,
    /// 1-based index into firing_order
    pub firing_position: u8,
    /// compression TDC in 0..720; firing_order[0] is 0.0
    pub tdc_deg: f32,
    /// simulated injector channel; equals cylinder
    pub injector_channel: u8,
    /// simulated coil channel; equals cylinder
    pub coil_channel: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    pub displacement_cc: f32,
    pub cylinders: u8,
    /// 1-based cylinder order in firing sequence, e.g. [1,3,4,2].
    pub firing_order: Vec<u8>,
    pub compression_ratio: f32,
    pub aspiration: Aspiration,
    pub rev_limit: u16,
    pub idle_rpm: u16,
    pub trigger: TriggerConfig,
    pub injectors: Vec<InjectorConfig>,
    /// 1-based bank assignment for each cylinder, in cylinder-number order.
    pub cylinder_banks: Vec<u8>,
}

impl EngineConfig {
    pub fn default_inline4() -> Self {
        Self {
            name: "OpenTech VR4 2.0L simulation".into(),
            displacement_cc: 1998.0,
            cylinders: 4,
            firing_order: vec![1, 3, 4, 2],
            compression_ratio: 9.5,
            aspiration: Aspiration::NaturallyAspirated,
            rev_limit: 7000,
            idle_rpm: 900,
            trigger: TriggerConfig::default(),
            injectors: vec![InjectorConfig {
                id: "SIM-440".into(),
                flow_cc_min: 440.0,
                rated_pressure_kpa: 300.0,
                dead_time_ms: 1.0,
                dead_time_curve: vec![(8.0, 1.8), (12.0, 1.2), (13.8, 1.0), (15.0, 0.9)],
                min_pw_ms: 0.8,
                max_duty: 0.90,
                cylinders: vec![1, 2, 3, 4],
            }],
            cylinder_banks: vec![1, 1, 1, 1],
        }
    }

    /// Synthetic simulator profile for sequencing tests; not an engine calibration.
    pub fn virtual_sequential(cylinders: u8) -> Option<Self> {
        let firing_order = default_firing_order(cylinders)?;
        let mut engine = Self::default_inline4();
        engine.name = format!("OpenTech virtual {cylinders}-cylinder simulation");
        engine.cylinders = cylinders;
        engine.firing_order = firing_order;
        engine.cylinder_banks = vec![1; cylinders as usize];
        engine.injectors[0].cylinders = (1..=cylinders).collect();
        Some(engine)
    }

    pub fn displacement_per_cylinder_cc(&self) -> f32 {
        if self.cylinders == 0 {
            0.0
        } else {
            self.displacement_cc / self.cylinders as f32
        }
    }

    pub fn cylinder_layout(&self) -> Option<Vec<CylinderAssignment>> {
        if !(1..=MAX_SEQUENTIAL_CYLINDERS).contains(&self.cylinders)
            || self.firing_order.len() != self.cylinders as usize
        {
            return None;
        }
        let mut seen = [false; MAX_SEQUENTIAL_CYLINDERS as usize + 1];
        for &c in &self.firing_order {
            if c == 0 || c > self.cylinders || seen[c as usize] {
                return None;
            }
            seen[c as usize] = true;
        }
        let spacing = 720.0 / self.cylinders as f32;
        Some(
            self.firing_order
                .iter()
                .enumerate()
                .map(|(i, &cylinder)| CylinderAssignment {
                    cylinder,
                    firing_position: i as u8 + 1,
                    tdc_deg: i as f32 * spacing,
                    injector_channel: cylinder,
                    coil_channel: cylinder,
                })
                .collect(),
        )
    }

    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        macro_rules! push {
            ($code:expr, $field:expr, $message:expr, $severity:expr $(,)?) => {
                issues.push(ValidationIssue {
                    code: $code.to_string(),
                    field: $field.to_string(),
                    message: $message.to_string(),
                    severity: $severity,
                })
            };
        }

        if !(1..=MAX_SEQUENTIAL_CYLINDERS).contains(&self.cylinders) {
            push!(
                "ENG_CYLINDERS",
                "cylinders",
                "Cylinder count must be 1..=8",
                Severity::Error,
            );
        }
        if !(200.0..=12_000.0).contains(&self.displacement_cc) {
            push!(
                "ENG_DISPLACEMENT",
                "displacement_cc",
                "Displacement must be 200..12000 cc",
                Severity::Error,
            );
        }
        if self.firing_order.len() != self.cylinders as usize {
            push!(
                "ENG_FIRING_LEN",
                "firing_order",
                "Firing order length must equal cylinder count",
                Severity::Error,
            );
        } else {
            let mut seen = vec![false; self.cylinders as usize + 1];
            for &c in &self.firing_order {
                if c == 0 || c > self.cylinders || seen[c as usize] {
                    push!(
                        "ENG_FIRING_INVALID",
                        "firing_order",
                        "Firing order must be a permutation of 1..=cylinders",
                        Severity::Error,
                    );
                    break;
                }
                seen[c as usize] = true;
            }
        }
        if !(4.0..=20.0).contains(&self.compression_ratio) {
            push!(
                "ENG_COMPRESSION",
                "compression_ratio",
                "Compression ratio must be 4..20",
                Severity::Error,
            );
        }
        if self.rev_limit < 1000 || self.rev_limit > 15_000 {
            push!(
                "ENG_REV_LIMIT",
                "rev_limit",
                "Rev limit must be 1000..15000",
                Severity::Error,
            );
        }
        if self.cylinder_banks.len() != self.cylinders as usize {
            push!(
                "ENG_BANKS",
                "cylinder_banks",
                "Bank assignment length must equal cylinder count",
                Severity::Error,
            );
        }
        for inj in &self.injectors {
            if !(40.0..=3000.0).contains(&inj.flow_cc_min) {
                push!(
                    "INJ_FLOW",
                    "injectors.flow_cc_min",
                    "Injector flow must be 40..3000 cc/min",
                    Severity::Error,
                );
            }
            if !(100.0..=1000.0).contains(&inj.rated_pressure_kpa) {
                push!(
                    "INJ_PRESSURE",
                    "injectors.rated_pressure_kpa",
                    "Rated pressure must be 100..1000 kPa",
                    Severity::Error,
                );
            }
            if !(0.1..=5.0).contains(&inj.dead_time_ms) {
                push!(
                    "INJ_DEADTIME",
                    "injectors.dead_time_ms",
                    "Dead time must be 0.1..5 ms",
                    Severity::Error,
                );
            }
            if !(0.1..=1.0).contains(&inj.max_duty) {
                push!(
                    "INJ_DUTY",
                    "injectors.max_duty",
                    "Max duty must be 0.1..1.0",
                    Severity::Error,
                );
            }
        }
        match self.trigger.pattern {
            TriggerPattern::MissingTooth { teeth, missing } => {
                if !(4..=120).contains(&teeth) || missing == 0 || missing >= teeth {
                    push!(
                        "TRIGGER_PATTERN",
                        "trigger.pattern",
                        "Missing-tooth wheel requires 4..=120 teeth and 1..teeth missing",
                        Severity::Error,
                    );
                }
            }
            TriggerPattern::DualWheel { .. } => push!(
                "TRIGGER_NOT_IMPLEMENTED",
                "trigger.pattern",
                "Dual-wheel decoder is not implemented in v1",
                Severity::Error,
            ),
        }
        if self.trigger.cam_required
            && !matches!(self.trigger.pattern, TriggerPattern::MissingTooth { .. })
        {
            push!(
                "TRIGGER_CAM",
                "trigger.cam_required",
                "Sequential operation requires a supported crank pattern with cam phase",
                Severity::Error,
            );
        }
        if !self.trigger.cam_required {
            push!(
                "ENG_SEQUENTIAL_CAM",
                "trigger.cam_required",
                "Sequential fuel and ignition require cam phase; set trigger.cam_required = true",
                Severity::Error,
            );
        }
        if self.injectors.len() != 1 {
            push!(
                "INJ_DEFINITIONS",
                "injectors",
                "Exactly one injector definition is supported and it must list every cylinder",
                Severity::Error,
            );
        }
        let n = self.cylinders.min(MAX_SEQUENTIAL_CYLINDERS);
        let mut counts = [0u8; MAX_SEQUENTIAL_CYLINDERS as usize + 1];
        let mut out_of_range = false;
        for inj in &self.injectors {
            for &c in &inj.cylinders {
                if c == 0 || c > n {
                    out_of_range = true;
                } else {
                    counts[c as usize] = counts[c as usize].saturating_add(1);
                }
            }
        }
        if out_of_range || (1..=n).any(|c| counts[c as usize] != 1) {
            push!(
                "INJ_COVERAGE",
                "injectors.cylinders",
                "Every cylinder 1..=cylinders must be listed exactly once across injector definitions",
                Severity::Error,
            );
        }
        issues
    }
}

pub fn default_firing_order(cylinders: u8) -> Option<Vec<u8>> {
    Some(match cylinders {
        1 => vec![1],
        2 => vec![1, 2],
        3 => vec![1, 2, 3],
        4 => vec![1, 3, 4, 2],
        5 => vec![1, 2, 4, 5, 3],
        6 => vec![1, 5, 3, 6, 2, 4],
        7 => vec![1, 3, 5, 7, 2, 4, 6],
        8 => vec![1, 8, 4, 3, 6, 5, 7, 2],
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub code: String,
    pub field: String,
    pub message: String,
    pub severity: Severity,
}

impl ValidationIssue {
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// Validate fuel configuration independently so the setup wizard can display
/// every problem rather than the first one.
pub fn validate_fuel_config(cfg: &FuelConfig) -> Vec<ValidationIssue> {
    let mut out = Vec::new();
    let mut add = |code: &str, msg: &str| {
        out.push(ValidationIssue {
            code: code.into(),
            field: "fuel".into(),
            message: msg.into(),
            severity: Severity::Error,
        });
    };
    if !(8.0..=20.0).contains(&cfg.stoich_afr) {
        add("FUEL_STOICH", "Stoichiometric AFR must be 8..20");
    }
    if !(0.65..=0.9).contains(&cfg.fuel_density_g_cc) {
        add("FUEL_DENSITY", "Fuel density must be 0.65..0.9 g/cc");
    }
    if !(30.0..=60.0).contains(&cfg.lower_heating_value_mj_kg) {
        add("FUEL_LHV", "Lower heating value must be 30..60 MJ/kg");
    }
    if !(100.0..=1000.0).contains(&cfg.pressure_reference_kpa) {
        add("FUEL_PRESSURE", "Reference pressure must be 100..1000 kPa");
    }
    if let LoadStrategy::Blended { alpha_percent } = cfg.load_strategy {
        if alpha_percent > 100 {
            add("FUEL_BLEND", "Alpha-N blend must be 0..100 percent");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_engine_is_valid() {
        let issues = EngineConfig::default_inline4().validate();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn duplicate_firing_order_rejected() {
        let mut e = EngineConfig::default_inline4();
        e.firing_order = vec![1, 2, 3, 3];
        assert!(e.validate().iter().any(|i| i.code == "ENG_FIRING_INVALID"));
    }

    #[test]
    fn dead_time_interpolates() {
        let inj = &EngineConfig::default_inline4().injectors[0];
        assert!((inj.dead_time_at(13.8) - 1.0).abs() < 1e-6);
        assert!((inj.dead_time_at(12.9) - 1.1).abs() < 1e-6);
        assert!((inj.dead_time_at(2.0) - 1.8).abs() < 1e-6);
    }

    #[test]
    fn invalid_fuel_rejected() {
        let f = FuelConfig {
            stoich_afr: 3.0,
            ..Default::default()
        };
        assert!(!validate_fuel_config(&f).is_empty());
    }

    #[test]
    fn default_firing_orders_match_decision_table() {
        let orders: [&[u8]; 8] = [
            &[1],
            &[1, 2],
            &[1, 2, 3],
            &[1, 3, 4, 2],
            &[1, 2, 4, 5, 3],
            &[1, 5, 3, 6, 2, 4],
            &[1, 3, 5, 7, 2, 4, 6],
            &[1, 8, 4, 3, 6, 5, 7, 2],
        ];
        for n in 1..=8 {
            assert_eq!(default_firing_order(n).unwrap(), orders[n as usize - 1]);
        }
        assert_eq!(default_firing_order(0), None);
        assert_eq!(default_firing_order(9), None);
    }

    #[test]
    fn virtual_sequential_profiles_validate_for_one_to_eight() {
        for n in 1..=8 {
            let issues = EngineConfig::virtual_sequential(n).unwrap().validate();
            assert!(issues.is_empty(), "n={n}: {issues:?}");
        }
        assert!(EngineConfig::virtual_sequential(0).is_none());
        assert!(EngineConfig::virtual_sequential(9).is_none());
    }

    #[test]
    fn layout_is_even_fire_with_one_channel_per_cylinder() {
        for n in 1..=8 {
            let engine = EngineConfig::virtual_sequential(n).unwrap();
            let layout = engine.cylinder_layout().unwrap();
            assert_eq!(layout.len(), n as usize);
            for (i, slot) in layout.iter().enumerate() {
                assert_eq!(slot.cylinder, engine.firing_order[i]);
                assert_eq!(slot.firing_position, i as u8 + 1);
                assert_eq!(slot.tdc_deg, i as f32 * (720.0 / n as f32));
                assert_eq!(slot.injector_channel, slot.cylinder);
                assert_eq!(slot.coil_channel, slot.cylinder);
            }
        }
    }

    #[test]
    fn default_inline4_layout_is_unchanged() {
        let pairs: Vec<_> = EngineConfig::default_inline4()
            .cylinder_layout()
            .unwrap()
            .iter()
            .map(|s| (s.cylinder, s.tdc_deg))
            .collect();
        assert_eq!(pairs, vec![(1, 0.0), (3, 180.0), (4, 360.0), (2, 540.0)]);
    }

    #[test]
    fn cylinder_count_above_eight_rejected() {
        let mut engine = EngineConfig::default_inline4();
        engine.cylinders = 9;
        engine.firing_order = (1..=9).collect();
        engine.cylinder_banks = vec![1; 9];
        engine.injectors[0].cylinders = (1..=9).collect();
        assert!(engine.validate().iter().any(|i| i.code == "ENG_CYLINDERS"));
        assert!(engine.cylinder_layout().is_none());
    }

    #[test]
    fn injector_coverage_enforced() {
        for cylinders in [vec![1, 2, 3], vec![1, 2, 3, 4, 4], vec![1, 2, 3, 5]] {
            let mut engine = EngineConfig::default_inline4();
            engine.injectors[0].cylinders = cylinders;
            assert!(engine.validate().iter().any(|i| i.code == "INJ_COVERAGE"));
        }
    }

    #[test]
    fn multiple_injector_definitions_rejected() {
        let mut engine = EngineConfig::default_inline4();
        let mut second = engine.injectors[0].clone();
        second.cylinders = vec![];
        engine.injectors.push(second);
        assert!(engine
            .validate()
            .iter()
            .any(|i| i.code == "INJ_DEFINITIONS"));
    }

    #[test]
    fn sequential_requires_cam_phase() {
        let mut engine = EngineConfig::default_inline4();
        engine.trigger.cam_required = false;
        assert!(engine
            .validate()
            .iter()
            .any(|i| i.code == "ENG_SEQUENTIAL_CAM"));
    }
}
