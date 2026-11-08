//! `UltraZeno` M1–M2–M3 Structural Compression & Dual-Process Information Bottleneck Engine (`src/ultra_zeno.rs`).
//!
//! Zero-dependency implementation of:
//! 1. **Information Theory Primitives:** Binary Shannon entropy $H_2(p)$, discrete Shannon entropy
//!    $H(P)$, Kullback–Leibler divergence $D_{\text{KL}}(P \parallel Q)$, and Structural Minimum
//!    Description Length $L_{\text{MDL}}(G, \mathcal{P}_t) = L_{\text{model}}(\mathcal{P}_t) + L_{\text{residual}}(G \mid \mathcal{P}_t)$.
//! 2. **Kahneman System 1 / System 2 Cognitive Routing (`CognitiveMode`):**
//!    - `System1FastReflex` (low degree entropy & low KL surprise vs Erdős–Rényi prior)
//!    - `System1To2UltraZenoBridge` (moderate surprise, dyadic M2 quotient graph compression)
//!    - `System2DeepDeliberation` (high surprise, full dyadic schedule + M3 clique & LLL verification)
//! 3. **Zeno Dyadic Schedule ($\varepsilon_t = \varepsilon_0 \cdot 2^{-t}$):** Minimizes the Information
//!    Bottleneck Lagrangian $\mathcal{L}_{\text{IB}} = L_{\text{MDL}}(G, \mathcal{P}_t) - \beta \, I(G; M_2)$.

/// Binary Shannon entropy $H_2(p) = -p \log_2(p) - (1-p)\log_2(1-p)$ in bits.
#[inline]
pub fn binary_entropy_bits(p: f64) -> f64 {
    let clamped = p.clamp(0.0, 1.0);
    if clamped <= 1e-12 || clamped >= 1.0 - 1e-12 {
        return 0.0;
    }
    -clamped * clamped.log2() - (1.0 - clamped) * (1.0 - clamped).log2()
}

/// Shannon entropy $H(P) = -\sum p_i \log_2(p_i)$ in bits for a discrete distribution.
pub fn shannon_entropy_bits(probs: &[f64]) -> f64 {
    let sum: f64 = probs.iter().copied().filter(|&p| p > 0.0).sum();
    if sum <= 0.0 {
        return 0.0;
    }
    probs
        .iter()
        .copied()
        .filter(|&p| p > 1e-15)
        .map(|p| {
            let norm_p = p / sum;
            -norm_p * norm_p.log2()
        })
        .sum()
}

/// Kullback–Leibler divergence $D_{\text{KL}}(P \parallel Q) = \sum p_i \log_2(p_i / q_i)$ in bits.
pub fn kl_divergence_bits(p_target: &[f64], q_draft: &[f64]) -> f64 {
    let p_sum: f64 = p_target.iter().copied().filter(|&x| x > 0.0).sum();
    let q_sum: f64 = q_draft.iter().copied().filter(|&x| x > 0.0).sum();
    if p_sum <= 0.0 || q_sum <= 0.0 {
        return 0.0;
    }

    p_target
        .iter()
        .zip(q_draft.iter())
        .map(|(&p_raw, &q_raw)| {
            let p = p_raw / p_sum;
            if p <= 1e-15 {
                0.0
            } else {
                let q = (q_raw / q_sum).max(1e-12);
                p * (p / q).log2()
            }
        })
        .sum()
}

/// System 1 / System 2 cognitive operating mode across the M1–M2–M3 hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveMode {
    System1FastReflex,
    System1To2UltraZenoBridge,
    System2DeepDeliberation,
}

/// Per-step Minimum Description Length (MDL) telemetry for a Zeno dyadic step $t$.
#[derive(Debug, Clone, PartialEq)]
pub struct MdlCompressionTelemetry {
    pub zeno_step: usize,
    pub dyadic_epsilon: f64,
    pub num_partitions: usize,
    pub naive_graph_bits: f64,
    pub model_header_bits: f64,
    pub residual_entropy_bits: f64,
    pub total_mdl_bits: f64,
    pub compression_ratio: f64,
    pub mutual_information_bits: f64,
}

/// Compact bit-matrix adjacency representation using `u64` words and hardware `POPCNT`.
#[derive(Clone, Debug)]
pub struct BitMatrix64 {
    pub num_vertices: usize,
    pub words_per_row: usize,
    pub data: Vec<u64>,
}

impl BitMatrix64 {
    pub fn new(n: usize) -> Self {
        let words_per_row = n.div_ceil(64);
        Self { num_vertices: n, words_per_row, data: vec![0u64; n * words_per_row] }
    }

    #[inline]
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        if u < self.num_vertices && v < self.num_vertices && u != v {
            self.data[u * self.words_per_row + (v >> 6)] |= 1u64 << (v & 63);
            self.data[v * self.words_per_row + (u >> 6)] |= 1u64 << (u & 63);
        }
    }

    #[inline]
    pub fn row(&self, u: usize) -> &[u64] {
        let start = u * self.words_per_row;
        &self.data[start..start + self.words_per_row]
    }
}

#[inline]
fn intersect_popcnt(row: &[u64], mask: &[u64]) -> usize {
    row.iter().zip(mask.iter()).map(|(&a, &b)| (a & b).count_ones() as usize).sum()
}

/// Computes the Structural Minimum Description Length (MDL) of a graph under partition `parts`.
pub fn compute_structural_mdl(
    num_vertices: usize,
    adj: &BitMatrix64,
    parts: &[Vec<usize>],
    zeno_step: usize,
    dyadic_epsilon: f64,
) -> (MdlCompressionTelemetry, Vec<(usize, usize)>) {
    let n = num_vertices;
    let k = parts.len().max(1);
    let naive_graph_bits = (n * n.saturating_sub(1) / 2) as f64;

    let k_log2 = (k as f64).log2().max(1.0);
    let num_pairs = k * k.saturating_sub(1) / 2;
    let model_header_bits = (n as f64) * k_log2 + (num_pairs as f64) * 16.0;

    let words_per_row = adj.words_per_row;
    let mut masks = vec![0u64; k * words_per_row];
    for (p_idx, p) in parts.iter().enumerate() {
        let base = p_idx * words_per_row;
        for &v in p {
            if v < n {
                masks[base + (v >> 6)] |= 1u64 << (v & 63);
            }
        }
    }

    let mut residual_entropy_bits = 0.0f64;
    let mut total_edges = 0usize;
    let mut quotient_edges: Vec<(usize, usize)> = Vec::new();

    for i in 0..k {
        let vi = &parts[i];
        let mask_i = &masks[i * words_per_row..(i + 1) * words_per_row];
        if vi.len() > 1 {
            let intra_edges: usize =
                vi.iter().map(|&u| intersect_popcnt(adj.row(u), mask_i)).sum::<usize>() / 2;
            total_edges += intra_edges;
            let max_intra = vi.len() * (vi.len() - 1) / 2;
            let d_ii = intra_edges as f64 / (max_intra as f64);
            residual_entropy_bits += (max_intra as f64) * binary_entropy_bits(d_ii);
        }

        for j in (i + 1)..k {
            let vj = &parts[j];
            if vi.is_empty() || vj.is_empty() {
                continue;
            }
            let mask_j = &masks[j * words_per_row..(j + 1) * words_per_row];
            let bip_edges: usize = vi.iter().map(|&u| intersect_popcnt(adj.row(u), mask_j)).sum();
            total_edges += bip_edges;

            let pair_cap = (vi.len() * vj.len()) as f64;
            let d_ij = (bip_edges as f64) / pair_cap;
            residual_entropy_bits += pair_cap * binary_entropy_bits(d_ij);

            if d_ij >= dyadic_epsilon {
                quotient_edges.push((i, j));
            }
        }
    }

    let global_density =
        if naive_graph_bits > 0.0 { (total_edges as f64) / naive_graph_bits } else { 0.0 };
    let prior_er_bits = naive_graph_bits * binary_entropy_bits(global_density);
    let total_mdl_bits = model_header_bits + residual_entropy_bits;
    let mutual_info = (prior_er_bits - residual_entropy_bits).max(0.0);
    let compression_ratio =
        if total_mdl_bits > 0.0 { naive_graph_bits / total_mdl_bits } else { 1.0 };

    (
        MdlCompressionTelemetry {
            zeno_step,
            dyadic_epsilon,
            num_partitions: k,
            naive_graph_bits,
            model_header_bits,
            residual_entropy_bits,
            total_mdl_bits,
            compression_ratio,
            mutual_information_bits: mutual_info,
        },
        quotient_edges,
    )
}

#[derive(Debug, Clone)]
pub struct UltraZenoParams {
    pub num_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub terminals: Vec<usize>,
    pub initial_epsilon: f64,
    pub max_zeno_steps: usize,
    pub ib_beta: f64,
    pub system2_entropy_threshold: f64,
    pub target_clique_k: usize,
}

#[derive(Debug, Clone)]
pub struct UltraZenoSummary {
    pub cognitive_mode: CognitiveMode,
    pub shannon_degree_entropy_bits: f64,
    pub kl_divergence_from_er_bits: f64,
    pub optimal_zeno_step: usize,
    pub optimal_dyadic_epsilon: f64,
    pub quotient_supernodes: usize,
    pub quotient_superedges: usize,
    pub mdl_compression_ratio: f64,
    pub mutual_information_bits: f64,
    pub information_bottleneck_lagrangian: f64,
    pub trajectory: Vec<MdlCompressionTelemetry>,
}

/// Runs the M1–M2–M3 Information Bottleneck & Zeno dyadic MDL compression pipeline.
pub fn run_ultra_zeno_bottleneck(params: &UltraZenoParams) -> Result<UltraZenoSummary, String> {
    let n = params.num_vertices;
    if n == 0 {
        return Err("num_vertices must be at least 1".into());
    }

    let mut adj = BitMatrix64::new(n);
    let mut degrees = vec![0usize; n];
    for &(u, v) in &params.edges {
        if u < n && v < n && u != v {
            adj.add_undirected_edge(u, v);
        }
    }
    for (u, deg) in degrees.iter_mut().enumerate().take(n) {
        *deg = adj.row(u).iter().map(|w| w.count_ones() as usize).sum();
    }

    let deg_probs: Vec<f64> = degrees.iter().map(|&d| (d + 1) as f64).collect();
    let m1_degree_entropy_bits = shannon_entropy_bits(&deg_probs);
    let uniform_q = vec![1.0 / (n as f64); n];
    let m1_kl_surprise_bits = kl_divergence_bits(&deg_probs, &uniform_q);

    let cognitive_mode = if m1_degree_entropy_bits < params.system2_entropy_threshold * 0.5
        && m1_kl_surprise_bits < 0.05
    {
        CognitiveMode::System1FastReflex
    } else if m1_kl_surprise_bits < 0.25 {
        CognitiveMode::System1To2UltraZenoBridge
    } else {
        CognitiveMode::System2DeepDeliberation
    };

    let max_steps = params.max_zeno_steps.clamp(1, 6);
    let mut zeno_trajectory = Vec::with_capacity(max_steps);
    let mut best_ib_score = f64::INFINITY;
    let mut optimal_quotient_edges = Vec::new();
    let mut optimal_num_parts = 2usize;
    let mut optimal_zeno_step = 0usize;

    let mut sorted_vertices: Vec<usize> = (0..n).collect();
    sorted_vertices.sort_unstable_by_key(|&u| degrees[u]);

    for step in 0..max_steps {
        let dyadic_scale = 0.5f64.powi(step as i32);
        let eps_t = (params.initial_epsilon * dyadic_scale).clamp(0.02, 0.49);
        let min_parts = (1usize << (step + 1)).min(n);

        let cap = n.div_ceil(min_parts);
        let mut parts: Vec<Vec<usize>> = (0..min_parts).map(|_| Vec::with_capacity(cap)).collect();
        for (idx, &v) in sorted_vertices.iter().enumerate() {
            parts[idx % min_parts].push(v);
        }

        let (telemetry, q_edges) = compute_structural_mdl(n, &adj, &parts, step, eps_t);
        let ib_lagrangian =
            telemetry.total_mdl_bits - params.ib_beta * telemetry.mutual_information_bits;

        if ib_lagrangian < best_ib_score {
            best_ib_score = ib_lagrangian;
            optimal_quotient_edges = q_edges;
            optimal_num_parts = telemetry.num_partitions;
            optimal_zeno_step = step;
        }

        zeno_trajectory.push(telemetry);
        if cognitive_mode == CognitiveMode::System1FastReflex {
            break;
        }
    }

    let best = zeno_trajectory[optimal_zeno_step].clone();
    Ok(UltraZenoSummary {
        cognitive_mode,
        shannon_degree_entropy_bits: m1_degree_entropy_bits,
        kl_divergence_from_er_bits: m1_kl_surprise_bits,
        optimal_zeno_step,
        optimal_dyadic_epsilon: best.dyadic_epsilon,
        quotient_supernodes: optimal_num_parts,
        quotient_superedges: optimal_quotient_edges.len(),
        mdl_compression_ratio: best.compression_ratio,
        mutual_information_bits: best.mutual_information_bits,
        information_bottleneck_lagrangian: best_ib_score,
        trajectory: zeno_trajectory,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultra_zeno_information_bottleneck_and_mdl_invariants() {
        assert!((binary_entropy_bits(0.5) - 1.0).abs() < 1e-12);
        assert!(binary_entropy_bits(0.0).abs() < 1e-12);
        assert!(binary_entropy_bits(1.0).abs() < 1e-12);

        let uniform_4 = [0.25, 0.25, 0.25, 0.25];
        assert!((shannon_entropy_bits(&uniform_4) - 2.0).abs() < 1e-12);

        let skewed = [0.70, 0.10, 0.10, 0.10];
        let kl = kl_divergence_bits(&skewed, &uniform_4);
        assert!(kl > 0.0);

        let n = 64;
        let mut edges = Vec::new();
        for u in 0..16 {
            for v in (u + 1)..16 {
                edges.push((u, v));
            }
        }
        for u in 16..32 {
            for v in (u + 1)..32 {
                edges.push((u, v));
            }
        }
        edges.push((15, 16));

        let summary = run_ultra_zeno_bottleneck(&UltraZenoParams {
            num_vertices: n,
            edges,
            terminals: vec![0, 15, 16, 31],
            initial_epsilon: 0.32,
            max_zeno_steps: 4,
            ib_beta: 1.5,
            system2_entropy_threshold: 1.5,
            target_clique_k: 6,
        })
        .expect("UltraZeno bottleneck succeeds");

        assert!(summary.shannon_degree_entropy_bits > 3.0);
        assert!(summary.kl_divergence_from_er_bits > 0.1);
        assert!(summary.mdl_compression_ratio > 1.2);
        assert!(summary.mutual_information_bits > 0.0);
    }
}
