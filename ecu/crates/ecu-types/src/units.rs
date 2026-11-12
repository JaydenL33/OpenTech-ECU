use alloc::string::{String, ToString};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Physical ranges used to reject implausible data at the boundary.
/// These are sanity bounds, not engine-specific limits.
pub const RPM_RANGE: (f32, f32) = (0.0, 20_000.0);
pub const PRESSURE_KPA_RANGE: (f32, f32) = (0.0, 600.0);
pub const TEMP_C_RANGE: (f32, f32) = (-60.0, 300.0);
pub const LAMBDA_RANGE: (f32, f32) = (0.0, 2.0);
pub const VOLT_RANGE: (f32, f32) = (0.0, 24.0);
pub const PW_MS_RANGE: (f32, f32) = (0.0, 50.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuantityError {
    pub quantity: &'static str,
    pub reason: &'static str,
}

impl fmt::Display for QuantityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {}: {}", self.quantity, self.reason)
    }
}

macro_rules! quantity {
    ($name:ident, $unit:literal, $min:expr, $max:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub f32);

        impl $name {
            pub const MIN: f32 = $min;
            pub const MAX: f32 = $max;
            pub const UNIT: &'static str = $unit;

            pub fn new(value: f32) -> Result<Self, QuantityError> {
                if value.is_finite() && (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(QuantityError {
                        quantity: stringify!($name),
                        reason: "outside sanity bounds or not finite",
                    })
                }
            }

            /// Construct without validation. Only for values already proven
            /// valid or for display placeholders. Never use in control paths.
            pub const fn raw(value: f32) -> Self {
                Self(value)
            }

            #[inline]
            pub const fn get(self) -> f32 {
                self.0
            }

            pub fn is_sane(self) -> bool {
                self.0.is_finite() && (Self::MIN..=Self::MAX).contains(&self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.0, $unit)
            }
        }
    };
}

quantity!(Rpm, "rpm", RPM_RANGE.0, RPM_RANGE.1);
quantity!(
    PressureKpa,
    "kPa",
    PRESSURE_KPA_RANGE.0,
    PRESSURE_KPA_RANGE.1
);
quantity!(TempC, "degC", TEMP_C_RANGE.0, TEMP_C_RANGE.1);
quantity!(Lambda, "lambda", LAMBDA_RANGE.0, LAMBDA_RANGE.1);
quantity!(Volts, "V", VOLT_RANGE.0, VOLT_RANGE.1);
quantity!(PwMs, "ms", PW_MS_RANGE.0, PW_MS_RANGE.1);

impl Rpm {
    #[inline]
    pub fn radians_per_second(self) -> f32 {
        self.0 * core::f32::consts::TAU / 60.0
    }
}

impl TempC {
    #[inline]
    pub fn kelvin(self) -> f32 {
        self.0 + 273.15
    }
}

impl PressureKpa {
    #[inline]
    pub fn pascals(self) -> f32 {
        self.0 * 1000.0
    }
}

impl Lambda {
    /// Convert lambda to AFR using an explicitly supplied stoichiometric AFR.
    /// The stoichiometric ratio is fuel-specific (E85, methanol and gasoline
    /// differ), so it is never assumed here.
    #[inline]
    pub fn to_afr(self, stoich_afr: f32) -> f32 {
        self.0 * stoich_afr
    }
}

impl Volts {
    /// Battery voltage correction is a lookup, not a linear guess. This helper
    /// exists only for documentation/tests.
    #[inline]
    pub fn is_cranking_plausible(self) -> bool {
        (7.0..=15.0).contains(&self.0)
    }
}

/// Convert AFR to lambda with an explicit fuel stoichiometry.
pub fn afr_to_lambda(afr: f32, stoich_afr: f32) -> Result<f32, QuantityError> {
    if !afr.is_finite() || !stoich_afr.is_finite() || stoich_afr <= 8.0 || stoich_afr > 20.0 {
        return Err(QuantityError {
            quantity: "afr",
            reason: "non-finite or stoichiometry outside 8..20",
        });
    }
    Ok(afr / stoich_afr)
}

/// Format a float without locale dependence, used in logs and reports.
pub fn fmt_f32(v: f32, decimals: usize) -> String {
    let mut s = alloc::format!("{:.*}", decimals, v);
    if s == "-0.00" || s == "-0.0" || s == "-0" {
        s = s.trim_start_matches('-').to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nan_and_out_of_range() {
        assert!(Rpm::new(f32::NAN).is_err());
        assert!(Rpm::new(-1.0).is_err());
        assert!(Lambda::new(2.1).is_err());
        assert!(TempC::new(120.0).is_ok());
    }

    #[test]
    fn explicit_stoich_conversion() {
        let l = Lambda::new(0.85).unwrap();
        assert!((l.to_afr(14.7) - 12.495).abs() < 1e-4);
        assert!((afr_to_lambda(12.495, 14.7).unwrap() - 0.85).abs() < 1e-4);
        // E85 uses a different stoich ratio; the API forces the caller to say so.
        assert!((l.to_afr(9.8) - 8.33).abs() < 1e-4);
    }

    #[test]
    fn kelvin_and_pressure() {
        assert!((TempC::new(20.0).unwrap().kelvin() - 293.15).abs() < 1e-3);
        assert!((PressureKpa::new(101.3).unwrap().pascals() - 101_300.0).abs() < 0.1);
    }
}
