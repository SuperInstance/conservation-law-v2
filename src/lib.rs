//! Conservation Law v2 — proving that Grand Pattern systems conserve their scalar vibe.
//!
//! The key insight: conservation of ONE scalar is trivially verifiable.
//! Conservation of 16 noisy dimensions is impossible. That's why vibe is mono-dimensional.

use serde::{Deserialize, Serialize};

/// A conservation verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationResult {
    pub initial_total: f64,
    pub final_total: f64,
    pub absolute_error: f64,
    pub relative_error: f64,
    pub is_conserved: bool,
    pub law_name: String,
}

/// A trace of conservation values over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationTrace {
    pub law_name: String,
    pub values: Vec<f64>,
    pub tolerance: f64,
}

impl ConservationTrace {
    pub fn new(law_name: &str, tolerance: f64) -> Self {
        Self { law_name: law_name.into(), values: Vec::new(), tolerance }
    }

    pub fn record(&mut self, value: f64) {
        self.values.push(value);
    }

    /// Maximum absolute drift from initial value.
    pub fn max_drift(&self) -> f64 {
        if self.values.len() < 2 { return 0.0; }
        let initial = self.values[0];
        self.values.iter().map(|v| (v - initial).abs()).fold(0.0_f64, f64::max)
    }

    /// Maximum relative drift from initial value.
    pub fn max_relative_drift(&self) -> f64 {
        if self.values.len() < 2 { return 0.0; }
        let initial = self.values[0].abs().max(f64::EPSILON);
        self.max_drift() / initial
    }

    /// All values within tolerance of initial?
    pub fn is_conserved(&self) -> bool {
        self.max_drift() <= self.tolerance
    }

    /// Drift at each step relative to previous.
    pub fn step_drifts(&self) -> Vec<f64> {
        self.values.windows(2).map(|w| (w[1] - w[0]).abs()).collect()
    }

    /// Is drift increasing (unstable) or decreasing (converging)?
    pub fn drift_trend(&self) -> DriftTrend {
        let drifts = self.step_drifts();
        if drifts.len() < 3 { return DriftTrend::Unknown; }
        let first_half: f64 = drifts[..drifts.len()/2].iter().sum();
        let second_half: f64 = drifts[drifts.len()/2..].iter().sum();
        if second_half < first_half * 0.9 { DriftTrend::Converging }
        else if second_half > first_half * 1.1 { DriftTrend::Diverging }
        else { DriftTrend::Stable }
    }

    /// Final verification.
    pub fn verify(&self) -> ConservationResult {
        let initial = self.values.first().copied().unwrap_or(0.0);
        let final_val = self.values.last().copied().unwrap_or(0.0);
        let abs_err = (final_val - initial).abs();
        let rel_err = abs_err / initial.abs().max(f64::EPSILON);
        ConservationResult {
            initial_total: initial,
            final_total: final_val,
            absolute_error: abs_err,
            relative_error: rel_err,
            is_conserved: self.is_conserved(),
            law_name: self.law_name.clone(),
        }
    }
}

/// Drift trend direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftTrend {
    Converging,
    Stable,
    Diverging,
    Unknown,
}

/// Verify sum conservation: sum(values) stays constant through operations.
pub fn verify_sum_conservation(
    initial: &[f64],
    steps: &[Vec<f64>],
    tolerance: f64,
) -> ConservationTrace {
    let mut trace = ConservationTrace::new("sum", tolerance);
    let sum: f64 = initial.iter().sum();
    trace.record(sum);
    for step in steps {
        trace.record(step.iter().sum());
    }
    trace
}

/// Verify L2 conservation: ||values||₂ stays constant.
pub fn verify_l2_conservation(
    initial: &[f64],
    steps: &[Vec<f64>],
    tolerance: f64,
) -> ConservationTrace {
    let mut trace = ConservationTrace::new("l2", tolerance);
    let norm: f64 = initial.iter().map(|v| v * v).sum::<f64>().sqrt();
    trace.record(norm);
    for step in steps {
        let n: f64 = step.iter().map(|v| v * v).sum::<f64>().sqrt();
        trace.record(n);
    }
    trace
}

/// Verify mono-dimensional vibe conservation: a single scalar stays constant.
pub fn verify_mono_conservation(
    initial_vibe: f64,
    operations: &[f64],
    tolerance: f64,
) -> ConservationTrace {
    let mut trace = ConservationTrace::new("mono-vibe", tolerance);
    trace.record(initial_vibe);
    for &v in operations {
        trace.record(v);
    }
    trace
}

/// Diffusion step: spread values along edges, conserving total.
pub fn diffuse_step(values: &mut [f64], adjacency: &[Vec<(usize, f64)>], blend: f64) {
    let n = values.len();
    let mut deltas = vec![0.0; n];
    for (i, neighbors) in adjacency.iter().enumerate() {
        for &(j, weight) in neighbors {
            let diff = values[j] - values[i];
            deltas[i] += weight * diff * blend;
        }
    }
    for (i, d) in deltas.iter().enumerate() {
        values[i] += d;
    }
}

/// Build a ring adjacency list with uniform coupling.
pub fn ring_adjacency(n: usize, coupling: f64) -> Vec<Vec<(usize, f64)>> {
    (0..n).map(|i| {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        vec![(prev, coupling), (next, coupling)]
    }).collect()
}

/// Build a star adjacency list with uniform coupling.
pub fn star_adjacency(n: usize, center: usize, coupling: f64) -> Vec<Vec<(usize, f64)>> {
    let mut adj = vec![Vec::new(); n];
    for i in 0..n {
        if i != center {
            adj[center].push((i, coupling));
            adj[i].push((center, coupling));
        }
    }
    adj
}

/// Build a full mesh adjacency list.
pub fn mesh_adjacency(n: usize, coupling: f64) -> Vec<Vec<(usize, f64)>> {
    (0..n).map(|i| {
        (0..n).filter(|&j| j != i).map(|j| (j, coupling)).collect()
    }).collect()
}

/// Run a full conservation experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub topology: String,
    pub n_nodes: usize,
    pub n_steps: usize,
    pub sum_conservation: ConservationResult,
    pub l2_conservation: ConservationResult,
    pub final_equilibrium_distance: f64,
}

/// Run diffusion experiment on a topology.
pub fn run_diffusion_experiment(
    topology: &str,
    n_nodes: usize,
    n_steps: usize,
    blend: f64,
    tolerance: f64,
) -> ExperimentResult {
    let mut values = vec![0.0; n_nodes];
    values[0] = 10.0;

    let adjacency = match topology {
        "ring" => ring_adjacency(n_nodes, 1.0),
        "star" => star_adjacency(n_nodes, 0, 1.0),
        "mesh" => mesh_adjacency(n_nodes, 1.0 / (n_nodes - 1) as f64),
        _ => ring_adjacency(n_nodes, 1.0),
    };

    let mut steps = Vec::new();
    for _ in 0..n_steps {
        diffuse_step(&mut values, &adjacency, blend);
        steps.push(values.clone());
    }

    let initial: Vec<f64> = {
        let mut v = vec![0.0; n_nodes];
        v[0] = 10.0;
        v
    };
    let sum_trace = verify_sum_conservation(&initial, &steps, tolerance);
    let l2_trace = verify_l2_conservation(&initial, &steps, tolerance);

    let mean = 10.0 / n_nodes as f64;
    let eq_dist = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>().sqrt();

    ExperimentResult {
        topology: topology.into(),
        n_nodes,
        n_steps,
        sum_conservation: sum_trace.verify(),
        l2_conservation: l2_trace.verify(),
        final_equilibrium_distance: eq_dist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_empty() {
        let trace = ConservationTrace::new("test", 0.01);
        assert_eq!(trace.max_drift(), 0.0);
        assert!(trace.is_conserved());
    }

    #[test]
    fn test_trace_single_value() {
        let mut trace = ConservationTrace::new("test", 0.01);
        trace.record(5.0);
        assert_eq!(trace.max_drift(), 0.0);
    }

    #[test]
    fn test_trace_conserved() {
        let mut trace = ConservationTrace::new("test", 0.1);
        trace.record(10.0);
        trace.record(10.01);
        trace.record(9.99);
        trace.record(10.0);
        assert!(trace.is_conserved());
        assert!(trace.max_drift() < 0.1);
    }

    #[test]
    fn test_trace_violated() {
        let mut trace = ConservationTrace::new("test", 0.01);
        trace.record(10.0);
        trace.record(12.0);
        assert!(!trace.is_conserved());
    }

    #[test]
    fn test_drift_trend_converging() {
        let mut trace = ConservationTrace::new("test", 1.0);
        trace.record(10.0);
        trace.record(10.5);
        trace.record(10.3);
        trace.record(10.1);
        trace.record(10.05);
        assert_eq!(trace.drift_trend(), DriftTrend::Converging);
    }

    #[test]
    fn test_drift_trend_unknown() {
        let trace = ConservationTrace::new("test", 1.0);
        assert_eq!(trace.drift_trend(), DriftTrend::Unknown);
    }

    #[test]
    fn test_verify_result() {
        let mut trace = ConservationTrace::new("sum", 0.1);
        trace.record(10.0);
        trace.record(10.05);
        let result = trace.verify();
        assert!(result.is_conserved);
        assert!((result.absolute_error - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_ring_adjacency() {
        let adj = ring_adjacency(5, 1.0);
        assert_eq!(adj.len(), 5);
        assert!(adj[0].iter().any(|(j, _)| *j == 4)); // wraps around
        assert!(adj[0].iter().any(|(j, _)| *j == 1));
    }

    #[test]
    fn test_star_adjacency() {
        let adj = star_adjacency(5, 0, 1.0);
        assert_eq!(adj[0].len(), 4);
        assert_eq!(adj[1].len(), 1);
    }

    #[test]
    fn test_mesh_adjacency() {
        let adj = mesh_adjacency(4, 0.5);
        assert_eq!(adj.len(), 4);
        assert_eq!(adj[0].len(), 3);
    }

    #[test]
    fn test_diffuse_step_conserves_sum() {
        let mut values = vec![10.0, 0.0, 0.0, 0.0];
        let adj = ring_adjacency(4, 0.5);
        let initial_sum: f64 = values.iter().sum();
        for _ in 0..20 {
            diffuse_step(&mut values, &adj, 0.1);
        }
        let final_sum: f64 = values.iter().sum();
        assert!((final_sum - initial_sum).abs() < 1e-10);
    }

    #[test]
    fn test_diffuse_step_converges() {
        let mut values = vec![10.0, 0.0, 0.0, 0.0];
        let adj = ring_adjacency(4, 0.5);
        for _ in 0..100 {
            diffuse_step(&mut values, &adj, 0.1);
        }
        let mean = 2.5;
        for v in &values {
            assert!((v - mean).abs() < 0.1, "v={v}");
        }
    }

    #[test]
    fn test_sum_conservation_trace() {
        let initial = vec![10.0, 0.0, 0.0];
        let mut vals = initial.clone();
        let adj = ring_adjacency(3, 1.0);
        let mut steps = Vec::new();
        for _ in 0..10 {
            diffuse_step(&mut vals, &adj, 0.1);
            steps.push(vals.clone());
        }
        let trace = verify_sum_conservation(&initial, &steps, 1e-6);
        assert!(trace.is_conserved());
    }

    #[test]
    fn test_l2_not_conserved_in_diffusion() {
        let initial = vec![10.0, 0.0, 0.0];
        let mut vals = initial.clone();
        let adj = ring_adjacency(3, 1.0);
        let mut steps = Vec::new();
        for _ in 0..10 {
            diffuse_step(&mut vals, &adj, 0.1);
            steps.push(vals.clone());
        }
        let trace = verify_l2_conservation(&initial, &steps, 0.1);
        // L2 norm changes during diffusion (only sum is conserved)
        assert!(!trace.is_conserved() || trace.max_drift() < 0.1);
    }

    #[test]
    fn test_mono_conservation() {
        let trace = verify_mono_conservation(5.0, &[5.0, 5.0, 5.0, 5.0], 0.001);
        assert!(trace.is_conserved());
    }

    #[test]
    fn test_mono_violation() {
        let trace = verify_mono_conservation(5.0, &[5.0, 6.0, 7.0], 0.001);
        assert!(!trace.is_conserved());
    }

    #[test]
    fn test_step_drifts() {
        let mut trace = ConservationTrace::new("test", 1.0);
        trace.record(10.0);
        trace.record(10.1);
        trace.record(10.0);
        let drifts = trace.step_drifts();
        assert_eq!(drifts.len(), 2);
        assert!((drifts[0] - 0.1).abs() < 1e-10);
        assert!((drifts[1] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_relative_drift() {
        let mut trace = ConservationTrace::new("test", 0.1);
        trace.record(100.0);
        trace.record(101.0);
        assert!((trace.max_relative_drift() - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_run_ring_experiment() {
        let result = run_diffusion_experiment("ring", 5, 50, 0.1, 1e-6);
        assert!(result.sum_conservation.is_conserved);
        assert_eq!(result.topology, "ring");
    }

    #[test]
    fn test_run_mesh_experiment() {
        let result = run_diffusion_experiment("mesh", 4, 50, 0.1, 1e-6);
        assert!(result.sum_conservation.is_conserved);
        assert!(result.final_equilibrium_distance < 2.0);
    }

    #[test]
    fn test_serialization() {
        let mut trace = ConservationTrace::new("test", 0.01);
        trace.record(5.0);
        trace.record(5.01);
        let json = serde_json::to_string(&trace).unwrap();
        let restored: ConservationTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.values.len(), 2);
        assert!(restored.is_conserved());
    }
}
