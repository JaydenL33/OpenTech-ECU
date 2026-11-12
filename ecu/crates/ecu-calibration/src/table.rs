use std::string::String;
use std::vec::Vec;

use ecu_types::{Provenance, Severity, SourceKind, ValidationIssue};
use serde::{Deserialize, Serialize};

use crate::CalibrationError;

pub const MAX_RANK: usize = 4;
pub const MAX_AXIS_POINTS: usize = 32;
pub const MIN_AXIS_POINTS: usize = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    pub name: String,
    pub unit: String,
    pub breakpoints: Vec<f32>,
}

impl Axis {
    pub fn new(name: &str, unit: &str, breakpoints: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            unit: unit.into(),
            breakpoints,
        }
    }

    pub fn validate(&self) -> Vec<TableIssue> {
        let mut out = Vec::new();
        if !(MIN_AXIS_POINTS..=MAX_AXIS_POINTS).contains(&self.breakpoints.len()) {
            out.push(TableIssue::error(
                "axis_points",
                &format!(
                    "axis '{}' must have {MIN_AXIS_POINTS}..={MAX_AXIS_POINTS} points",
                    self.name
                ),
            ));
            return out;
        }
        for w in self.breakpoints.windows(2) {
            if !w[0].is_finite() || !w[1].is_finite() || w[1] <= w[0] {
                out.push(TableIssue::error(
                    "axis_order",
                    &format!(
                        "axis '{}' must be finite and strictly increasing",
                        self.name
                    ),
                ));
                break;
            }
        }
        out
    }

    /// Return (lower index, upper index, fraction) for a coordinate. Values
    /// outside the axis are clamped, never extrapolated.
    pub fn position(&self, value: f32) -> (usize, usize, f32) {
        let n = self.breakpoints.len();
        let v = if value.is_finite() {
            value
        } else {
            self.breakpoints[0]
        };
        if v <= self.breakpoints[0] {
            return (0, 0, 0.0);
        }
        if v >= self.breakpoints[n - 1] {
            return (n - 1, n - 1, 0.0);
        }
        // Binary search for the segment. Axis sizes are bounded at 32, so even
        // the linear fallback is deterministic and cheap.
        let mut lo = 0usize;
        for i in 0..n - 1 {
            if v >= self.breakpoints[i] && v <= self.breakpoints[i + 1] {
                lo = i;
                break;
            }
        }
        let span = self.breakpoints[lo + 1] - self.breakpoints[lo];
        let t = if span > 0.0 {
            (v - self.breakpoints[lo]) / span
        } else {
            0.0
        };
        (lo, lo + 1, t.clamp(0.0, 1.0))
    }

    pub fn nearest_index(&self, value: f32) -> usize {
        let mut best = 0usize;
        let mut best_d = f32::INFINITY;
        for (i, b) in self.breakpoints.iter().enumerate() {
            let d = (value - b).abs();
            if d < best_d {
                best = i;
                best_d = d;
            }
        }
        best
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub description: String,
    pub axes: Vec<Axis>,
    /// Row-major, first axis slowest: index = (((i0*n1)+i1)*n2+i2)...
    pub values: Vec<f32>,
    pub provenance: Provenance,
    pub revision: u32,
    pub value_min: f32,
    pub value_max: f32,
    pub decimals: u8,
}

impl Table {
    pub fn new(
        id: &str,
        name: &str,
        unit: &str,
        axes: Vec<Axis>,
        values: Vec<f32>,
        provenance: Provenance,
    ) -> Result<Self, CalibrationError> {
        let expected = values_len(&axes);
        if expected == 0 || values.len() != expected {
            return Err(CalibrationError::InvalidValues(format!(
                "table '{id}' expected {expected} values, got {}",
                values.len()
            )));
        }
        let table = Self {
            id: id.into(),
            name: name.into(),
            unit: unit.into(),
            description: String::new(),
            axes,
            values,
            provenance,
            revision: 1,
            value_min: 0.0,
            value_max: 150.0,
            decimals: 2,
        };
        let issues = table.issues();
        if let Some(e) = issues.iter().find(|i| i.severity == Severity::Error) {
            return Err(CalibrationError::InvalidValues(e.message.clone()));
        }
        Ok(table)
    }

    pub fn with_bounds(mut self, min: f32, max: f32, decimals: u8) -> Self {
        self.value_min = min;
        self.value_max = max;
        self.decimals = decimals;
        self
    }

    pub fn with_description(mut self, s: &str) -> Self {
        self.description = s.into();
        self
    }

    pub fn rank(&self) -> usize {
        self.axes.len()
    }

    pub fn dims(&self) -> Vec<usize> {
        self.axes.iter().map(|a| a.breakpoints.len()).collect()
    }

    pub fn issues(&self) -> Vec<TableIssue> {
        let mut out = Vec::new();
        if self.axes.is_empty() || self.axes.len() > MAX_RANK {
            out.push(TableIssue::error("rank", "table rank must be 1..=4"));
            return out;
        }
        for a in &self.axes {
            out.extend(a.validate());
        }
        if self.values.len() != values_len(&self.axes) {
            out.push(TableIssue::error(
                "values_len",
                "value count does not match axis dimensions",
            ));
            return out;
        }
        for v in &self.values {
            if !v.is_finite() {
                out.push(TableIssue::error(
                    "value_finite",
                    "table contains non-finite value",
                ));
                break;
            }
            if *v < self.value_min - 1e-4 || *v > self.value_max + 1e-4 {
                out.push(TableIssue::error(
                    "value_bounds",
                    &format!(
                        "value {v} outside declared bounds {}..{}",
                        self.value_min, self.value_max
                    ),
                ));
                break;
            }
        }
        for (i, axis) in self.axes.iter().enumerate() {
            if axis.breakpoints.len() > MAX_AXIS_POINTS {
                out.push(TableIssue::error(
                    "axis_points",
                    &format!("axis {i} exceeds {MAX_AXIS_POINTS} points"),
                ));
            }
        }
        out
    }

    pub fn validate(&self) -> Vec<ValidationIssue> {
        self.issues()
            .into_iter()
            .map(|i| ValidationIssue {
                code: i.code,
                field: i.field,
                message: i.message,
                severity: i.severity,
            })
            .collect()
    }

    fn flat_index(&self, indices: &[usize]) -> Option<usize> {
        if indices.len() != self.axes.len() {
            return None;
        }
        let mut flat = 0usize;
        for (i, idx) in indices.iter().enumerate() {
            let n = self.axes[i].breakpoints.len();
            if *idx >= n {
                return None;
            }
            flat = flat * n + idx;
        }
        Some(flat)
    }

    pub fn value_at(&self, indices: &[usize]) -> Option<f32> {
        self.flat_index(indices)
            .and_then(|i| self.values.get(i).copied())
    }

    pub fn set_value_at(&mut self, indices: &[usize], value: f32) -> Result<f32, CalibrationError> {
        if !value.is_finite() || value < self.value_min || value > self.value_max {
            return Err(CalibrationError::InvalidValues(format!(
                "value {value} outside {}..{}",
                self.value_min, self.value_max
            )));
        }
        let idx = self
            .flat_index(indices)
            .ok_or_else(|| CalibrationError::InvalidValues("index rank mismatch".into()))?;
        let old = self.values[idx];
        self.values[idx] = value;
        Ok(old)
    }

    /// Multilinear interpolation. Coordinate length must equal table rank.
    pub fn interpolate(&self, coords: &[f32]) -> Result<f32, CalibrationError> {
        if coords.len() != self.axes.len() {
            return Err(CalibrationError::InvalidValues(
                "coordinate rank does not match table".into(),
            ));
        }
        let mut positions = Vec::with_capacity(self.axes.len());
        for (i, axis) in self.axes.iter().enumerate() {
            positions.push(axis.position(coords[i]));
        }
        let corners = 1usize << self.axes.len();
        let mut sum = 0.0f32;
        let mut weights = 0.0f32;
        for corner in 0..corners {
            let mut idx = Vec::with_capacity(self.axes.len());
            let mut w = 1.0f32;
            for (d, (lo, hi, t)) in positions.iter().enumerate() {
                if (corner >> d) & 1 == 1 {
                    idx.push(*hi);
                    w *= *t;
                } else {
                    idx.push(*lo);
                    w *= 1.0 - *t;
                }
            }
            if let Some(v) = self.value_at(&idx) {
                sum += w * v;
                weights += w;
            }
        }
        if weights <= 0.0 {
            return Err(CalibrationError::InvalidValues(
                "interpolation weights invalid".into(),
            ));
        }
        Ok(sum / weights)
    }

    /// Nearest grid cell for a coordinate. This is the learning key; it does not
    /// interpolate, because mixing observations from adjacent cells would hide
    /// real calibration error.
    pub fn cell_indices(&self, coords: &[f32]) -> Option<Vec<usize>> {
        if coords.len() != self.axes.len() {
            return None;
        }
        Some(
            self.axes
                .iter()
                .enumerate()
                .map(|(i, a)| a.nearest_index(coords[i]))
                .collect(),
        )
    }

    pub fn replace_values(&mut self, values: Vec<f32>) -> Result<(), CalibrationError> {
        if values.len() != values_len(&self.axes) {
            return Err(CalibrationError::InvalidValues(
                "value count mismatch".into(),
            ));
        }
        let mut clone = self.clone();
        clone.values = values;
        if let Some(e) = clone
            .issues()
            .into_iter()
            .find(|i| i.severity == Severity::Error)
        {
            return Err(CalibrationError::InvalidValues(e.message));
        }
        self.values = clone.values;
        Ok(())
    }

    /// Resample onto new axis breakpoints. New points are interpolated inside
    /// the original extent and clamped at the edges. This is an editing aid, not
    /// new evidence.
    pub fn resample_axis(
        &self,
        axis_index: usize,
        new_breakpoints: Vec<f32>,
    ) -> Result<Self, CalibrationError> {
        if axis_index >= self.axes.len() {
            return Err(CalibrationError::InvalidValues(
                "axis index out of range".into(),
            ));
        }
        let mut axes = self.axes.clone();
        axes[axis_index] = Axis {
            name: axes[axis_index].name.clone(),
            unit: axes[axis_index].unit.clone(),
            breakpoints: new_breakpoints,
        };
        for a in &axes {
            for issue in a.validate() {
                if issue.severity == Severity::Error {
                    return Err(CalibrationError::InvalidValues(issue.message));
                }
            }
        }
        let dims = values_dims(&axes);
        let total: usize = dims.iter().product();
        let mut values = Vec::with_capacity(total);
        let mut idx = vec![0usize; axes.len()];
        for _ in 0..total {
            let mut coords = Vec::with_capacity(axes.len());
            for (d, a) in axes.iter().enumerate() {
                coords.push(a.breakpoints[idx[d]]);
            }
            values.push(self.interpolate(&coords)?);
            // increment row-major index
            for d in (0..idx.len()).rev() {
                idx[d] += 1;
                if idx[d] < dims[d] {
                    break;
                }
                idx[d] = 0;
            }
        }
        let mut out = self.clone();
        out.axes = axes;
        out.values = values;
        out.revision += 1;
        Ok(out)
    }

    pub fn estimated(&self) -> bool {
        matches!(
            self.provenance.source,
            SourceKind::EngineeringEstimate
                | SourceKind::SyntheticSimulationOnly
                | SourceKind::Unknown
        ) && !self.provenance.engine_specific_verified
    }

    pub fn mark_edited(&mut self, note: &str) {
        self.revision += 1;
        self.provenance.notes = format!("{} Edited: {note}", self.provenance.notes)
            .trim()
            .into();
    }
}

fn values_dims(axes: &[Axis]) -> Vec<usize> {
    axes.iter().map(|a| a.breakpoints.len()).collect()
}

fn values_len(axes: &[Axis]) -> usize {
    if axes.is_empty() {
        0
    } else {
        axes.iter().map(|a| a.breakpoints.len()).product()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableIssue {
    pub code: String,
    pub field: String,
    pub message: String,
    pub severity: Severity,
}

impl TableIssue {
    fn error(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            field: "table".into(),
            message: message.into(),
            severity: Severity::Error,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CellChange {
    pub indices: Vec<usize>,
    pub old: f32,
    pub new: f32,
    pub delta: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_id: String,
    pub changes: Vec<CellChange>,
    pub max_abs_delta: f32,
    pub mean_abs_delta: f32,
}

pub fn diff_tables(before: &Table, after: &Table) -> TableDiff {
    let mut changes = Vec::new();
    let n = before.values.len().min(after.values.len());
    let mut sum = 0.0f32;
    let mut max = 0.0f32;
    for i in 0..n {
        let delta = after.values[i] - before.values[i];
        if delta.abs() > 1e-6 {
            sum += delta.abs();
            max = max.max(delta.abs());
            changes.push(CellChange {
                indices: unflatten(before, i),
                old: before.values[i],
                new: after.values[i],
                delta,
            });
        }
    }
    let mean = if changes.is_empty() {
        0.0
    } else {
        sum / changes.len() as f32
    };
    TableDiff {
        table_id: before.id.clone(),
        changes,
        max_abs_delta: max,
        mean_abs_delta: mean,
    }
}

fn unflatten(table: &Table, mut flat: usize) -> Vec<usize> {
    let dims: Vec<usize> = table.axes.iter().map(|a| a.breakpoints.len()).collect();
    let mut idx = vec![0usize; dims.len()];
    for d in (0..dims.len()).rev() {
        idx[d] = flat % dims[d];
        flat /= dims[d];
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecu_types::Provenance;

    fn prov() -> Provenance {
        Provenance::synthetic_simulation("test")
    }

    fn table2() -> Table {
        Table::new(
            "ve",
            "VE",
            "%",
            vec![
                Axis::new("MAP", "kPa", vec![20.0, 60.0, 100.0]),
                Axis::new("RPM", "rpm", vec![1000.0, 3000.0, 5000.0]),
            ],
            vec![
                40.0, 50.0, 55.0, // MAP 20
                60.0, 75.0, 80.0, // MAP 60
                75.0, 90.0, 95.0, // MAP 100
            ],
            prov(),
        )
        .unwrap()
        .with_bounds(0.0, 120.0, 1)
    }

    #[test]
    fn bilinear_interpolation_is_exact_on_nodes_and_center() {
        let t = table2();
        assert!((t.interpolate(&[20.0, 1000.0]).unwrap() - 40.0).abs() < 1e-5);
        assert!((t.interpolate(&[60.0, 3000.0]).unwrap() - 75.0).abs() < 1e-5);
        // Center of the first cell: average of 40,50,60,75 = 56.25
        assert!((t.interpolate(&[40.0, 2000.0]).unwrap() - 56.25).abs() < 1e-5);
    }

    #[test]
    fn interpolation_clamps_and_never_extrapolates() {
        let t = table2();
        let low = t.interpolate(&[-100.0, 0.0]).unwrap();
        let high = t.interpolate(&[500.0, 9000.0]).unwrap();
        assert!((low - 40.0).abs() < 1e-5);
        assert!((high - 95.0).abs() < 1e-5);
    }

    #[test]
    fn nearest_cell_is_discrete() {
        let t = table2();
        assert_eq!(t.cell_indices(&[45.0, 2200.0]).unwrap(), vec![1, 1]);
        assert_eq!(t.cell_indices(&[21.0, 1100.0]).unwrap(), vec![0, 0]);
    }

    #[test]
    fn rank_mismatch_rejected() {
        let t = table2();
        assert!(t.interpolate(&[50.0]).is_err());
        assert!(t.cell_indices(&[50.0]).is_none());
    }

    #[test]
    fn invalid_values_rejected_and_old_table_unchanged() {
        let mut t = table2();
        let old = t.values.clone();
        assert!(t.replace_values(vec![1.0; 2]).is_err());
        assert!(t.replace_values(vec![200.0; 9]).is_err());
        assert_eq!(t.values, old);
    }

    #[test]
    fn axis_resampling_interpolates() {
        let t = table2();
        let r = t
            .resample_axis(1, vec![1000.0, 2000.0, 3000.0, 4000.0, 5000.0])
            .unwrap();
        assert_eq!(r.dims(), vec![3, 5]);
        assert!((r.interpolate(&[20.0, 2000.0]).unwrap() - 45.0).abs() < 1e-4);
    }

    #[test]
    fn rank4_interpolates() {
        let axes = vec![
            Axis::new("A", "u", vec![0.0, 10.0]),
            Axis::new("B", "u", vec![0.0, 10.0]),
            Axis::new("C", "u", vec![0.0, 10.0]),
            Axis::new("D", "u", vec![0.0, 10.0]),
        ];
        let mut values = Vec::new();
        for i in 0..16 {
            values.push(i as f32);
        }
        let t = Table::new("four", "4D", "u", axes, values, prov())
            .unwrap()
            .with_bounds(0.0, 100.0, 1);
        let center = t.interpolate(&[5.0, 5.0, 5.0, 5.0]).unwrap();
        let avg = t.values.iter().sum::<f32>() / 16.0;
        assert!((center - avg).abs() < 1e-4);
    }

    #[test]
    fn diff_reports_cells() {
        let a = table2();
        let mut b = a.clone();
        b.values[4] += 3.5;
        let d = diff_tables(&a, &b);
        assert_eq!(d.changes.len(), 1);
        assert!((d.max_abs_delta - 3.5).abs() < 1e-5);
        assert_eq!(d.changes[0].indices, vec![1, 1]);
    }
}
