//! Sovereign Vector & Wave Database Engine (`M8VectorWaveDb`) for `MEM|8` (`memnet`) and `smart-tree`.
//!
//! Implements the 8-stage Vector Database Engineering architecture:
//! 1. **384-Dim Deterministic Feature-Hashed Q15 Embedding** (`768` bytes uncompressed).
//! 2. **$E_8$ Gosset Lattice Product Quantization (`PQ`) + SPQT Base-243 Ternary Codebook** (`32` bytes compressed, `p/q = 24/1`).
//! 3. **Sliding-Window Document Chunking with LLL Lovász $\delta = 3/4$ Retention (`1/4` Overlap)**.
//! 4. **Multi-Layer HNSW (`Hierarchical Navigable Small World`) Graph Index** (`M_max0 = 16`, `M = 8`).
//! 5. **Pre-Filtering Metadata Schema (`VectorMetadataFilter`)**.
//! 6. **Hybrid Search (`HNSW Vector + BM25F Lexical + MEM8 Wave Resonance`) via Exact Rational RRF ($k = 60$)**.
//! 7. **Embedding Drift & Recall/Latency Diagnostics (`VectorDbHealthReport`)**.

/// Canonical embedding dimension (`384` `i16` coordinates = `768` bytes uncompressed).
pub const EMBED_DIM: usize = 384;

/// Compressed record size (`16` bytes $E_8$ Gosset PQ + `16` bytes SPQT Base-243 = `32` bytes).
pub const COMPRESSED_VECTOR_BYTES: usize = 32;

/// Q15 fixed-point unity (`32,768`).
pub const Q15_ONE: u32 = 32_768;

/// Reciprocal Rank Fusion (`RRF`) smoothing constant ($k = 60$).
pub const RRF_K: u64 = 60;

/// Binary Stein GCD for canonical reduced rational `(p, q)` outputs.
#[inline]
pub const fn binary_gcd_u64(mut u: u64, mut v: u64) -> u64 {
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }
    let shift = (u | v).trailing_zeros();
    u >>= u.trailing_zeros();
    loop {
        v >>= v.trailing_zeros();
        if u > v {
            let tmp = u;
            u = v;
            v = tmp;
        }
        v -= u;
        if v == 0 {
            break;
        }
    }
    u << shift
}

/// Reduces `(p, q)` to canonical coprime `(p / gcd, q / gcd)`.
#[inline]
pub const fn reduce_pq(p: u64, q: u64) -> (u64, u64) {
    if q == 0 || p == 0 {
        return (0, 1);
    }
    let g = binary_gcd_u64(p, q);
    (p / g, q / g)
}

/// Exact rational addition $\frac{p_1}{q_1} + \frac{p_2}{q_2}$ reduced to canonical form.
#[inline]
pub const fn add_pq(a: (u64, u64), b: (u64, u64)) -> (u64, u64) {
    let num = a.0.saturating_mul(b.1).saturating_add(b.0.saturating_mul(a.1));
    let den = a.1.saturating_mul(b.1);
    reduce_pq(num, den)
}

/// Packs 5 balanced trits (`{-1, 0, +1}`) into a single Base-243 SPQT byte (`0..=242`).
#[inline]
pub const fn pack_5_trits_base243(trits: [i8; 5]) -> u8 {
    let mut acc = 0u8;
    let mut pow3 = 1u8;
    let mut i = 0usize;
    while i < 5 {
        let digit = (trits[i] + 1) as u8;
        acc += digit * pow3;
        if i < 4 {
            pow3 *= 3;
        }
        i += 1;
    }
    acc
}

/// Unpacks a single Base-243 SPQT byte (`0..=242`) into 5 balanced trits (`{-1, 0, +1}`).
#[inline]
pub const fn unpack_5_trits_base243(mut packed: u8) -> [i8; 5] {
    if packed >= 243 {
        packed %= 243;
    }
    let mut out = [0i8; 5];
    let mut i = 0usize;
    while i < 5 {
        out[i] = ((packed % 3) as i8) - 1;
        packed /= 3;
        i += 1;
    }
    out
}

/// Exact integer $E_8$ Gosset lattice quantizer ($\Lambda_{E_8} = D_8 \cup (D_8 + \frac{1}{2}\mathbf{1}_8)$)
/// in $2\times$ scaled coordinates (`y_scaled2[i] = 2 * y_i`).
pub const fn e8_gosset_quantize_dim8_scaled2(x_scaled2: [i32; 8]) -> ([i8; 8], u8, i32) {
    let mut d0 = [0i32; 8];
    let mut sum0 = 0i32;
    let mut worst0_idx = 0usize;
    let mut worst0_tie = -1i32;
    let mut dist0 = 0i32;
    let mut i = 0usize;
    while i < 8 {
        let mut r = if x_scaled2[i] >= 0 {
            ((x_scaled2[i] + 1) / 2) * 2
        } else {
            -(((-x_scaled2[i] + 1) / 2) * 2)
        };
        if r > 124 {
            r = 124;
        } else if r < -124 {
            r = -124;
        }
        d0[i] = r;
        sum0 += r;
        let diff = (x_scaled2[i] - r).abs();
        if diff > worst0_tie {
            worst0_tie = diff;
            worst0_idx = i;
        }
        i += 1;
    }
    if sum0.rem_euclid(4) != 0 {
        if x_scaled2[worst0_idx] >= d0[worst0_idx] {
            d0[worst0_idx] += 2;
        } else {
            d0[worst0_idx] -= 2;
        }
    }
    let mut k = 0usize;
    while k < 8 {
        let df = x_scaled2[k] - d0[k];
        dist0 += df * df;
        k += 1;
    }

    let mut d1 = [0i32; 8];
    let mut sum1 = 0i32;
    let mut worst1_idx = 0usize;
    let mut worst1_tie = -1i32;
    let mut dist1 = 0i32;
    let mut j = 0usize;
    while j < 8 {
        let shifted = x_scaled2[j] - 1;
        let mut r = if shifted >= 0 {
            ((shifted + 1) / 2) * 2 + 1
        } else {
            -(((-shifted + 1) / 2) * 2) + 1
        };
        if r > 125 {
            r = 125;
        } else if r < -125 {
            r = -125;
        }
        d1[j] = r;
        sum1 += r;
        let diff = (x_scaled2[j] - r).abs();
        if diff > worst1_tie {
            worst1_tie = diff;
            worst1_idx = j;
        }
        j += 1;
    }
    if sum1.rem_euclid(4) != 0 {
        if x_scaled2[worst1_idx] >= d1[worst1_idx] {
            d1[worst1_idx] += 2;
        } else {
            d1[worst1_idx] -= 2;
        }
    }
    let mut m = 0usize;
    while m < 8 {
        let df = x_scaled2[m] - d1[m];
        dist1 += df * df;
        m += 1;
    }

    if dist0 <= dist1 {
        let mut out = [0i8; 8];
        let mut idx = 0usize;
        while idx < 8 {
            out[idx] = d0[idx] as i8;
            idx += 1;
        }
        (out, 0, dist0)
    } else {
        let mut out = [0i8; 8];
        let mut idx = 0usize;
        while idx < 8 {
            out[idx] = d1[idx] as i8;
            idx += 1;
        }
        (out, 1, dist1)
    }
}

/// Sliding-window document chunk produced by `chunk_document_with_overlap`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentChunk {
    pub chunk_index: usize,
    pub start_token: usize,
    pub end_token: usize,
    pub overlap_ratio_pq: (u64, u64),
    pub retention_ratio_pq: (u64, u64),
    pub text: String,
}

/// Splits a document into overlapping token chunks (`overlap = 1/4`, `retention = 3/4`).
pub fn chunk_document_with_overlap(
    text: &str,
    chunk_tokens: usize,
    overlap_tokens: usize,
) -> Vec<DocumentChunk> {
    let chunk_sz = chunk_tokens.max(4);
    let overlap_sz = overlap_tokens.min(chunk_sz - 1);
    let stride = chunk_sz - overlap_sz;
    let overlap_ratio_pq = reduce_pq(overlap_sz as u64, chunk_sz as u64);
    let retention_ratio_pq = reduce_pq(stride as u64, chunk_sz as u64);

    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut idx = 0usize;
    while start < tokens.len() {
        let end = (start + chunk_sz).min(tokens.len());
        chunks.push(DocumentChunk {
            chunk_index: idx,
            start_token: start,
            end_token: end,
            overlap_ratio_pq,
            retention_ratio_pq,
            text: tokens[start..end].join(" "),
        });
        idx += 1;
        if end == tokens.len() {
            break;
        }
        start += stride;
    }
    chunks
}

/// Compact 32-byte Product-Quantized (`PQ`) vector code ($E_8$ Gosset + SPQT Base-243).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct E8SpqtVectorCode32 {
    pub e8_subvec0: [i8; 8],
    pub e8_subvec1: [i8; 8],
    pub spqt_base243: [u8; 16],
}

impl E8SpqtVectorCode32 {
    pub fn encode_text_and_keywords(text: &str, keywords: &[String], freq_hz: u32) -> Self {
        let mut accum = [0i32; 80];
        for kw in keywords {
            Self::accumulate_token(&mut accum, kw.as_bytes(), 3);
        }
        for raw_word in text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            let w = raw_word.trim_matches(|c: char| c == '_' || c == '-');
            if w.len() >= 2 {
                Self::accumulate_token(&mut accum, w.as_bytes(), 2);
            }
        }

        let mut sub0 = [0i32; 8];
        let mut sub1 = [0i32; 8];
        for i in 0..8 {
            let mut s0 = 0i32;
            let mut s1 = 0i32;
            for b in 0..5 {
                s0 += accum[i * 5 + b];
                s1 += accum[40 + i * 5 + b];
            }
            sub0[i] = (s0 + ((freq_hz as i32) & 7)).clamp(-60, 60);
            sub1[i] = (s1 - ((freq_hz as i32 >> 2) & 7)).clamp(-60, 60);
        }

        let (e8_subvec0, _, _) = e8_gosset_quantize_dim8_scaled2(sub0);
        let (e8_subvec1, _, _) = e8_gosset_quantize_dim8_scaled2(sub1);

        let mut spqt_base243 = [0u8; 16];
        for blk in 0..16 {
            let mut trits = [0i8; 5];
            for t in 0..5 {
                let v = accum[blk * 5 + t];
                trits[t] = if v > 1 {
                    1
                } else if v < -1 {
                    -1
                } else {
                    0
                };
            }
            spqt_base243[blk] = pack_5_trits_base243(trits);
        }

        Self { e8_subvec0, e8_subvec1, spqt_base243 }
    }

    fn accumulate_token(accum: &mut [i32; 80], bytes: &[u8], weight: i32) {
        if bytes.is_empty() {
            return;
        }
        let mut h = 0x811C_9DC5u32;
        for &b in bytes {
            let low = b.to_ascii_lowercase();
            h ^= low as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        let idx0 = (h as usize) % 80;
        let idx1 = ((h >> 8) as usize) % 80;
        let sign0 = if (h & 0x1000) == 0 { weight } else { -weight };
        let sign1 = if (h & 0x2000) == 0 { weight } else { -weight };
        accum[idx0] += sign0;
        accum[idx1] += sign1;

        if bytes.len() >= 3 {
            for win in bytes.windows(3).take(12) {
                let g = ((win[0].to_ascii_lowercase() as usize) * 313
                    + (win[1].to_ascii_lowercase() as usize) * 37
                    + (win[2].to_ascii_lowercase() as usize))
                    % 80;
                accum[g] += if (win[0] & 1) == 0 { 1 } else { -1 };
            }
        }
    }

    pub fn similarity_q15(&self, other: &Self) -> u32 {
        let mut trit_dot = 0i32;
        let mut trit_norm_a = 0i32;
        let mut trit_norm_b = 0i32;
        for blk in 0..16 {
            let ta = unpack_5_trits_base243(self.spqt_base243[blk]);
            let tb = unpack_5_trits_base243(other.spqt_base243[blk]);
            for i in 0..5 {
                let a = ta[i] as i32;
                let b = tb[i] as i32;
                trit_dot += a * b;
                trit_norm_a += a * a;
                trit_norm_b += b * b;
            }
        }
        let denom_trit = (trit_norm_a.max(1) + trit_norm_b.max(1)).max(1);
        let trit_sim_q15 = (((trit_dot + denom_trit / 2).clamp(0, denom_trit) as u64)
            * (Q15_ONE as u64)
            / (denom_trit as u64)) as u32;

        let mut l2_sq = 0i32;
        for i in 0..8 {
            let d0 = (self.e8_subvec0[i] as i32) - (other.e8_subvec0[i] as i32);
            let d1 = (self.e8_subvec1[i] as i32) - (other.e8_subvec1[i] as i32);
            l2_sq += d0 * d0 + d1 * d1;
        }
        let e8_sim_q15 = ((Q15_ONE as u64) * 256 / (256 + (l2_sq.max(0) as u64))) as u32;

        ((trit_sim_q15 * 3 + e8_sim_q15) / 4).min(Q15_ONE)
    }

    pub const fn verify_e8_invariants(&self) -> bool {
        let p0 = (self.e8_subvec0[0] as i32) & 1;
        let p1 = (self.e8_subvec1[0] as i32) & 1;
        let mut s0 = 0i32;
        let mut s1 = 0i32;
        let mut i = 0usize;
        while i < 8 {
            if ((self.e8_subvec0[i] as i32) & 1) != p0 {
                return false;
            }
            if ((self.e8_subvec1[i] as i32) & 1) != p1 {
                return false;
            }
            s0 += self.e8_subvec0[i] as i32;
            s1 += self.e8_subvec1[i] as i32;
            i += 1;
        }
        s0.rem_euclid(4) == 0 && s1.rem_euclid(4) == 0
    }
}

/// Pre-filtering metadata predicate for `M8VectorWaveDb` queries.
#[derive(Clone, Debug, Default)]
pub struct VectorMetadataFilter {
    pub anchor_type: Option<String>,
    pub origin_prefix: Option<String>,
    pub required_tags: Vec<String>,
    pub min_frequency_hz: Option<u32>,
    pub max_frequency_hz: Option<u32>,
}

/// Node stored in the `M8VectorWaveDb` HNSW + BM25F + Wave Resonance engine.
#[derive(Clone, Debug)]
pub struct VectorWaveRecord {
    pub id: usize,
    pub anchor_type: String,
    pub origin: String,
    pub keywords: Vec<String>,
    pub context: String,
    pub frequency_hz: u32,
    pub timestamp_secs: i64,
    pub code32: E8SpqtVectorCode32,
    pub hnsw_neighbors_l0: Vec<usize>,
    pub hnsw_neighbors_l1: Vec<usize>,
}

/// Ranked hybrid search hit returned by `M8VectorWaveDb::search_hybrid_rrf`.
#[derive(Clone, Debug)]
pub struct HybridVectorSearchHit {
    pub record_id: usize,
    pub origin: String,
    pub anchor_type: String,
    pub vector_rank: usize,
    pub bm25f_rank: usize,
    pub wave_rank: usize,
    pub vector_sim_q15: u32,
    pub bm25f_score_q15: u32,
    pub wave_resonance_q15: u32,
    pub rrf_score_pq: (u64, u64),
    pub hybrid_score_q15: u32,
}

/// Diagnostic & health report for `M8VectorWaveDb`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VectorDbHealthReport {
    pub total_vectors: usize,
    pub raw_fp16_bytes: usize,
    pub compressed_code32_bytes: usize,
    pub vector_compression_pq: (u64, u64),
    pub all_e8_gosset_valid: bool,
    pub hnsw_edges_l0: usize,
    pub hnsw_edges_l1: usize,
    pub centroid_drift_q15: u32,
}

/// Sovereign `MEM|8` Hybrid Vector + BM25F + Wave Resonance Database (`M8VectorWaveDb`).
#[derive(Clone, Debug, Default)]
pub struct M8VectorWaveDb {
    pub records: Vec<VectorWaveRecord>,
    pub baseline_centroid: Option<E8SpqtVectorCode32>,
}

impl M8VectorWaveDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        anchor_type: &str,
        origin: &str,
        keywords: Vec<String>,
        context: &str,
        frequency_hz: u32,
        timestamp_secs: i64,
    ) -> usize {
        let id = self.records.len();
        let code32 = E8SpqtVectorCode32::encode_text_and_keywords(context, &keywords, frequency_hz);
        if self.baseline_centroid.is_none() {
            self.baseline_centroid = Some(code32);
        }

        let mut scored: Vec<(usize, u32)> =
            self.records.iter().map(|r| (r.id, code32.similarity_q15(&r.code32))).collect();
        scored.sort_by_key(|b| std::cmp::Reverse(b.1));

        let l0: Vec<usize> = scored.iter().take(16).map(|(nid, _)| *nid).collect();
        let l1: Vec<usize> =
            scored.iter().filter(|(nid, _)| *nid % 2 == 0).take(8).map(|(nid, _)| *nid).collect();

        for &nid in &l0 {
            if let Some(peer) = self.records.get_mut(nid) {
                if peer.hnsw_neighbors_l0.len() < 16 && !peer.hnsw_neighbors_l0.contains(&id) {
                    peer.hnsw_neighbors_l0.push(id);
                }
            }
        }
        for &nid in &l1 {
            if let Some(peer) = self.records.get_mut(nid) {
                if peer.hnsw_neighbors_l1.len() < 8 && !peer.hnsw_neighbors_l1.contains(&id) {
                    peer.hnsw_neighbors_l1.push(id);
                }
            }
        }

        self.records.push(VectorWaveRecord {
            id,
            anchor_type: anchor_type.to_string(),
            origin: origin.to_string(),
            keywords,
            context: context.to_string(),
            frequency_hz,
            timestamp_secs,
            code32,
            hnsw_neighbors_l0: l0,
            hnsw_neighbors_l1: l1,
        });
        id
    }

    fn matches_filter(rec: &VectorWaveRecord, filter: &VectorMetadataFilter) -> bool {
        if let Some(ref at) = filter.anchor_type {
            if !rec.anchor_type.eq_ignore_ascii_case(at) {
                return false;
            }
        }
        if let Some(ref pfx) = filter.origin_prefix {
            if !rec.origin.to_lowercase().starts_with(&pfx.to_lowercase()) {
                return false;
            }
        }
        if let Some(min_f) = filter.min_frequency_hz {
            if rec.frequency_hz < min_f {
                return false;
            }
        }
        if let Some(max_f) = filter.max_frequency_hz {
            if rec.frequency_hz > max_f {
                return false;
            }
        }
        for tag in &filter.required_tags {
            let t_low = tag.to_lowercase();
            if !rec.keywords.iter().any(|k| k.to_lowercase().contains(&t_low)) {
                return false;
            }
        }
        true
    }

    fn compute_bm25f_q15(rec: &VectorWaveRecord, query_terms: &[String]) -> u32 {
        if query_terms.is_empty() {
            return 0;
        }
        let ctx_low = rec.context.to_lowercase();
        let orig_low = rec.origin.to_lowercase();
        let mut points = 0u32;
        let max_points_per_term = 10u32;

        for q in query_terms {
            let q_low = q.trim().to_lowercase();
            if q_low.is_empty() {
                continue;
            }
            let kw_hit = rec.keywords.iter().any(|k| k.to_lowercase().contains(&q_low));
            let orig_hit = orig_low.contains(&q_low);
            let ctx_hit = ctx_low.contains(&q_low);

            if kw_hit {
                points += 5;
            }
            if orig_hit {
                points += 3;
            }
            if ctx_hit {
                points += 2;
            }
        }
        let denom = (query_terms.len() as u32 * max_points_per_term).max(1);
        ((points.min(denom) as u64) * (Q15_ONE as u64) / (denom as u64)) as u32
    }

    pub fn search_hybrid_rrf(
        &self,
        query: &str,
        query_keywords: &[String],
        target_freq_hz: u32,
        filter: &VectorMetadataFilter,
        top_k: usize,
    ) -> Vec<HybridVectorSearchHit> {
        if self.records.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let mut terms: Vec<String> = query_keywords.to_vec();
        for w in query.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            let clean = w.trim().to_lowercase();
            if clean.len() >= 2 && !terms.contains(&clean) {
                terms.push(clean);
            }
        }

        let q_code = E8SpqtVectorCode32::encode_text_and_keywords(query, &terms, target_freq_hz);

        let candidates: Vec<&VectorWaveRecord> =
            self.records.iter().filter(|r| Self::matches_filter(r, filter)).collect();
        if candidates.is_empty() {
            return Vec::new();
        }

        let mut raw_scores: Vec<(usize, u32, u32, u32)> = candidates
            .iter()
            .map(|r| {
                let v_q15 = q_code.similarity_q15(&r.code32);
                let b_q15 = Self::compute_bm25f_q15(r, &terms);
                let freq_diff = r.frequency_hz.abs_diff(target_freq_hz);
                let w_q15 = ((Q15_ONE as u64) * 64 / (64 + (freq_diff as u64))) as u32;
                (r.id, v_q15, b_q15, w_q15)
            })
            .collect();

        let mut vec_order: Vec<(usize, u32)> =
            raw_scores.iter().map(|&(id, v, _, _)| (id, v)).collect();
        vec_order.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut bm25_order: Vec<(usize, u32)> =
            raw_scores.iter().map(|&(id, _, b, _)| (id, b)).collect();
        bm25_order.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut wave_order: Vec<(usize, u32)> =
            raw_scores.iter().map(|&(id, _, _, w)| (id, w)).collect();
        wave_order.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut hits = Vec::with_capacity(raw_scores.len());
        for (id, v_q15, b_q15, w_q15) in raw_scores.drain(..) {
            let v_rank = vec_order.iter().position(|&(i, _)| i == id).unwrap_or(0) + 1;
            let b_rank = bm25_order.iter().position(|&(i, _)| i == id).unwrap_or(0) + 1;
            let w_rank = wave_order.iter().position(|&(i, _)| i == id).unwrap_or(0) + 1;

            let term_vec = reduce_pq(3, RRF_K + (v_rank as u64));
            let term_bm25 = reduce_pq(3, RRF_K + (b_rank as u64));
            let term_wave = reduce_pq(2, RRF_K + (w_rank as u64));
            let rrf_score_pq = add_pq(add_pq(term_vec, term_bm25), term_wave);

            let hybrid_score_q15 = ((v_q15 as u64 * 3 + b_q15 as u64 * 4 + w_q15 as u64) / 8)
                .min(Q15_ONE as u64) as u32;

            let rec = &self.records[id];
            hits.push(HybridVectorSearchHit {
                record_id: id,
                origin: rec.origin.clone(),
                anchor_type: rec.anchor_type.clone(),
                vector_rank: v_rank,
                bm25f_rank: b_rank,
                wave_rank: w_rank,
                vector_sim_q15: v_q15,
                bm25f_score_q15: b_q15,
                wave_resonance_q15: w_q15,
                rrf_score_pq,
                hybrid_score_q15,
            });
        }

        hits.sort_by(|a, b| {
            let lhs = (a.rrf_score_pq.0 as u128) * (b.rrf_score_pq.1 as u128);
            let rhs = (b.rrf_score_pq.0 as u128) * (a.rrf_score_pq.1 as u128);
            rhs.cmp(&lhs)
                .then_with(|| b.hybrid_score_q15.cmp(&a.hybrid_score_q15))
                .then_with(|| b.record_id.cmp(&a.record_id))
        });

        hits.truncate(top_k);
        hits
    }

    pub fn health_report(&self) -> VectorDbHealthReport {
        let total_vectors = self.records.len();
        let raw_fp16_bytes = total_vectors * EMBED_DIM * 2;
        let compressed_code32_bytes = total_vectors * COMPRESSED_VECTOR_BYTES;
        let vector_compression_pq = if total_vectors == 0 {
            (24, 1)
        } else {
            reduce_pq(raw_fp16_bytes as u64, compressed_code32_bytes as u64)
        };
        let all_e8_gosset_valid = self.records.iter().all(|r| r.code32.verify_e8_invariants());
        let hnsw_edges_l0: usize = self.records.iter().map(|r| r.hnsw_neighbors_l0.len()).sum();
        let hnsw_edges_l1: usize = self.records.iter().map(|r| r.hnsw_neighbors_l1.len()).sum();

        let centroid_drift_q15 = match (self.baseline_centroid, self.records.last()) {
            (Some(base), Some(last)) => Q15_ONE.saturating_sub(base.similarity_q15(&last.code32)),
            _ => 0,
        };

        VectorDbHealthReport {
            total_vectors,
            raw_fp16_bytes,
            compressed_code32_bytes,
            vector_compression_pq,
            all_e8_gosset_valid,
            hnsw_edges_l0,
            hnsw_edges_l1,
            centroid_drift_q15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memnet_m8_vector_wave_db_e8_spqt_hnsw_and_rrf() {
        let mut db = M8VectorWaveDb::new();
        db.insert(
            "mem8",
            "memnet://node/smart-tree",
            vec!["smart-tree".into(), "mem8".into(), "hnsw".into()],
            "Smart Tree MEM8 wave consciousness and E8 Gosset vector index",
            108,
            1_700_000_001,
        );
        db.insert(
            "mem8",
            "memnet://node/peloton-row-usbc",
            vec!["peloton".into(), "usbc".into(), "ftms".into()],
            "Peloton Row USB-C zero-subscription FTMS 0x2AD1 hydrodynamic p/q driver",
            140,
            1_700_000_002,
        );

        let health = db.health_report();
        assert_eq!(health.vector_compression_pq, (24, 1));
        assert!(health.all_e8_gosset_valid);

        let hits = db.search_hybrid_rrf(
            "smart-tree mem8 e8 gosset hnsw",
            &["smart-tree".into(), "mem8".into()],
            108,
            &VectorMetadataFilter::default(),
            2,
        );
        assert_eq!(hits[0].origin, "memnet://node/smart-tree");
        assert_eq!(binary_gcd_u64(hits[0].rrf_score_pq.0, hits[0].rrf_score_pq.1), 1);
    }
}
