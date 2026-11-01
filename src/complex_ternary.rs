//! Complex Ternary Manifold — a bit-exact reference oracle.
//!
//! This is the software half of the "waking" datapath described in the
//! UltraDreamState / AGP-ISA notes: a compacted complex wavefield is projected
//! onto a 9-state complex ternary alphabet, packed into four orthogonal
//! bit-planes, and contracted with only bitwise logic and population counts — no
//! floating-point multiply-accumulate in the inner loop.
//!
//! Its purpose is to be **the thing the other implementations are diffed
//! against**. RTL, C firmware and Zig SIMD kernels can disagree with each other
//! forever; they cannot all disagree with a reference that is pinned by tests.
//!
//! Honest scope, because the surrounding material is easy to over-read:
//!
//! - Column A only. These are deterministic integer and `f32` state machines.
//!   Nothing here is conscious, and nothing here models a mind.
//! - The hash used for the waking seal is [`crate::sha256`], because this crate
//!   is zero-dependency. The papers say BLAKE3; the construction is the same
//!   shape, the function differs. Treat the seal as a domain-separated digest,
//!   not as a BLAKE3 attestation.
//! - `f32` appears in the *packing* step only, where the source wavefield is
//!   real-valued. The contraction path is integer-exact.

use crate::sha256;

// ============================================================================
// 1. THE WORD: 64 complex ternary nodes in 4 orthogonal bit-planes
// ============================================================================

/// 64 complex nodes, 4 bits each: `(r_pos, r_neg, i_pos, i_neg)`.
///
/// A node is `+1` when its `*_pos` bit is set, `-1` when its `*_neg` bit is set,
/// and `0` when neither is. Both bits set is **undefined** and is exactly what
/// [`ComplexWord64::parity_fault`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComplexWord64 {
    pub r_pos: u64,
    pub r_neg: u64,
    pub i_pos: u64,
    pub i_neg: u64,
}

impl ComplexWord64 {
    pub const ZERO: Self = Self {
        r_pos: 0,
        r_neg: 0,
        i_pos: 0,
        i_neg: 0,
    };

    /// The invariant `(r_pos & r_neg) | (i_pos & i_neg) == 0`.
    pub fn is_valid(&self) -> bool {
        self.parity_fault() == 0
    }

    /// Bits where a node claims to be both `+1` and `-1`. O(1), no branching.
    ///
    /// On silicon this is the trap line: a single-event upset that sets one bit
    /// in the wrong plane lands here instead of silently flipping a sign.
    pub fn parity_fault(&self) -> u64 {
        (self.r_pos & self.r_neg) | (self.i_pos & self.i_neg)
    }

    /// Number of non-zero nodes (real **or** imaginary component).
    pub fn non_zero_marks(&self) -> u32 {
        ((self.r_pos | self.r_neg) | (self.i_pos | self.i_neg)).count_ones()
    }

    /// Real component of node `index` as `-1`, `0`, or `1`.
    pub fn real_trit(&self, index: usize) -> i8 {
        let bit = 1u64 << index;
        if self.r_pos & bit != 0 {
            1
        } else if self.r_neg & bit != 0 {
            -1
        } else {
            0
        }
    }

    /// Imaginary component of node `index` as `-1`, `0`, or `1`.
    pub fn imag_trit(&self, index: usize) -> i8 {
        let bit = 1u64 << index;
        if self.i_pos & bit != 0 {
            1
        } else if self.i_neg & bit != 0 {
            -1
        } else {
            0
        }
    }
}

// ============================================================================
// 2. PACKING: dead-zone quantisation, memoryless vs sigma-delta
// ============================================================================

/// Packing parameters. `gamma` is the block scale; the dead zone is `gamma/2`,
/// which is the minimax choice for this quantiser (snapping to zero and snapping
/// to `±gamma` cost the same at the boundary).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PackConfig {
    pub gamma: f32,
    /// Carrier wavenumber for the sigma-delta feedback tap `e^{i k0}`.
    /// `0.0` is baseband, which is what a phase-locked baseband field wants.
    pub carrier_rad: f32,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            carrier_rad: 0.0,
        }
    }
}

impl PackConfig {
    pub fn dead_zone(&self) -> f32 {
        self.gamma * 0.5
    }
}

/// Quantise one real value to `-1`, `0`, `+1`.
#[inline]
pub fn quantise(value: f32, dead_zone: f32) -> i8 {
    if value > dead_zone {
        1
    } else if value < -dead_zone {
        -1
    } else {
        0
    }
}

/// Memoryless packing: what a naive `round()`-style projection does.
///
/// This is the baseline the sigma-delta path is measured against, and the one
/// that produces the ~13° RMS phase staircasing the papers describe.
pub fn pack_row_memoryless(u: &[f32], v: &[f32], cfg: PackConfig) -> Vec<ComplexWord64> {
    debug_assert_eq!(u.len(), v.len());
    let dead = cfg.dead_zone();
    let mut words = vec![ComplexWord64::ZERO; u.len().div_ceil(64)];

    for (i, (&re, &im)) in u.iter().zip(v.iter()).enumerate() {
        let (word, bit) = (&mut words[i / 64], 1u64 << (i % 64));
        match quantise(re, dead) {
            1 => word.r_pos |= bit,
            -1 => word.r_neg |= bit,
            _ => {}
        }
        match quantise(im, dead) {
            1 => word.i_pos |= bit,
            -1 => word.i_neg |= bit,
            _ => {}
        }
    }
    words
}

/// First-order spatial sigma-delta packing with a carrier-tuned feedback tap.
///
/// The residual of each sample is carried forward (rotated by `e^{i k0}`) into
/// the next sample instead of being thrown away, which is what moves the
/// quantisation noise out of the carrier band. Two properties are load-bearing
/// and both are tested:
///
/// 1. **Zero-leakage:** a node that was pruned to exactly `0 + 0i` stays zero.
///    Without this gate, diffused error accumulates in empty memory and wakes
///    phantom nodes.
/// 2. **Determinism:** identical input always produces identical words, so a
///    cross-target arbiter can compare them byte for byte.
pub fn pack_row_sigma_delta(u: &[f32], v: &[f32], cfg: PackConfig) -> Vec<ComplexWord64> {
    debug_assert_eq!(u.len(), v.len());
    let dead = cfg.dead_zone();
    let (tap_re, tap_im) = (cfg.carrier_rad.cos(), cfg.carrier_rad.sin());
    let mut words = vec![ComplexWord64::ZERO; u.len().div_ceil(64)];
    let (mut err_re, mut err_im) = (0.0f32, 0.0f32);

    for (i, (&re, &im)) in u.iter().zip(v.iter()).enumerate() {
        let (word, bit) = (&mut words[i / 64], 1u64 << (i % 64));

        // Zero-leakage gate: a pruned node never absorbs or emits error.
        if re == 0.0 && im == 0.0 {
            continue;
        }

        let fb_re = tap_re * err_re - tap_im * err_im;
        let fb_im = tap_re * err_im + tap_im * err_re;
        let shaped_re = re + fb_re;
        let shaped_im = im + fb_im;

        let q_re = quantise(shaped_re, dead);
        let q_im = quantise(shaped_im, dead);

        match q_re {
            1 => word.r_pos |= bit,
            -1 => word.r_neg |= bit,
            _ => {}
        }
        match q_im {
            1 => word.i_pos |= bit,
            -1 => word.i_neg |= bit,
            _ => {}
        }

        // Carry the projection error — this is the whole trick.
        err_re = clamp_to_dead_zone(shaped_re - cfg.gamma * q_re as f32, dead);
        err_im = clamp_to_dead_zone(shaped_im - cfg.gamma * q_im as f32, dead);
    }
    words
}

#[inline]
fn clamp_to_dead_zone(value: f32, dead: f32) -> f32 {
    if value > dead {
        dead
    } else if value < -dead {
        -dead
    } else {
        value
    }
}

// ============================================================================
// 3. CONTRACTION: eight boolean masks, no multipliers
// ============================================================================

/// Contract two words: `(Re, Im)` of `w · x` summed over 64 complex nodes.
///
/// Complex multiplication has a real part `w_r x_r − w_i x_i` and an imaginary
/// part `w_r x_i + w_i x_r`. With trits, each of those is a count of agreeing
/// masks minus a count of disagreeing masks, so the whole inner product is
/// `popcount` differences and one subtraction at the end.
#[inline]
pub fn contract_word(w: &ComplexWord64, x: &ComplexWord64) -> (i32, i32) {
    let re_pos = ((w.r_pos & x.r_pos) | (w.r_neg & x.r_neg)).count_ones()
        + ((w.i_pos & x.i_neg) | (w.i_neg & x.i_pos)).count_ones();
    let re_neg = ((w.r_pos & x.r_neg) | (w.r_neg & x.r_pos)).count_ones()
        + ((w.i_pos & x.i_pos) | (w.i_neg & x.i_neg)).count_ones();
    let im_pos = ((w.r_pos & x.i_pos) | (w.r_neg & x.i_neg)).count_ones()
        + ((w.i_pos & x.r_pos) | (w.i_neg & x.r_neg)).count_ones();
    let im_neg = ((w.r_pos & x.i_neg) | (w.r_neg & x.i_pos)).count_ones()
        + ((w.i_pos & x.r_neg) | (w.i_neg & x.r_pos)).count_ones();

    (re_pos as i32 - re_neg as i32, im_pos as i32 - im_neg as i32)
}

/// Contract a whole row of words. Returns the unscaled integer accumulator.
pub fn contract_row(w: &[ComplexWord64], x: &[ComplexWord64]) -> (i64, i64) {
    let mut acc = (0i64, 0i64);
    for (a, b) in w.iter().zip(x.iter()) {
        let (re, im) = contract_word(a, b);
        acc.0 += re as i64;
        acc.1 += im as i64;
    }
    acc
}

/// Apply the outer block scale and return Q16.16, the canonical wire format.
pub fn scale_q16_16(acc: i64, gamma: f32) -> i32 {
    let scaled = acc as f32 * gamma * 65536.0;
    scaled.clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

// ============================================================================
// 4. FIBONACCI IN BALANCED TERNARY, CARRIED BY NAND GATES ALONE
// ============================================================================

/// The only gate. Everything below is built from this and nothing else.
#[inline]
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

#[inline]
pub fn not(a: bool) -> bool {
    nand(a, a)
}

#[inline]
pub fn and(a: bool, b: bool) -> bool {
    not(nand(a, b))
}

#[inline]
pub fn or(a: bool, b: bool) -> bool {
    nand(not(a), not(b))
}

#[inline]
pub fn xor(a: bool, b: bool) -> bool {
    and(nand(a, b), or(a, b))
}

/// 64-bit ripple-carry adder from NAND gates only.
///
/// Wrapping is intentional: this is a fixed-width machine adder, not bignum.
pub fn add_nand(a: u64, b: u64) -> u64 {
    let mut sum = 0u64;
    let mut carry = false;
    for bit in 0..64 {
        let x = (a >> bit) & 1 == 1;
        let y = (b >> bit) & 1 == 1;
        let s = xor(xor(x, y), carry);
        carry = or(and(x, y), and(carry, xor(x, y)));
        if s {
            sum |= 1u64 << bit;
        }
    }
    sum
}

/// `F(n)` computed by the gate cascade itself: `F(n) = F(n-1) + F(n-2)`.
pub fn fib_nand(n: usize) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = add_nand(a, b);
        a = b;
        b = next;
    }
    a
}

/// Balanced ternary (`{-1, 0, +1}`), least significant digit first.
///
/// Greedy with borrow: a remainder of `2` becomes `-1` and carries, which keeps
/// every digit inside the alphabet and keeps the representation unique.
/// Negative inputs are encoded by the same rule.
pub fn balanced_ternary(value: i64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut n = value;
    let mut out = String::new();
    while n != 0 {
        let mut rem = n % 3;
        n /= 3;
        match rem {
            2 => {
                rem = -1;
                n += 1;
            }
            -2 => {
                rem = 1;
                n -= 1;
            }
            _ => {}
        }
        out.push(match rem {
            -1 => '-',
            1 => '+',
            _ => '0',
        });
    }
    out
}

/// Decode balanced ternary back to an integer. `None` on any foreign character.
pub fn parse_balanced_ternary(text: &str) -> Option<i64> {
    let mut acc = 0i64;
    let mut place = 1i64;
    for ch in text.chars() {
        let digit = match ch {
            '-' => -1,
            '0' => 0,
            '+' => 1,
            _ => return None,
        };
        acc += digit * place;
        place *= 3;
    }
    Some(acc)
}

/// `F(1..=count)`, one cell each, as the published provenance strip.
pub fn fib_strip(count: usize) -> String {
    (1..=count)
        .map(|n| balanced_ternary(fib_nand(n) as i64))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The strip as it should read for `F(1)` through `F(12)`.
///
/// Note what this is *not*: the previously published strip opened with
/// `00+ 00+ 0-+ 00+`, which decodes to `9 9 6 9`, not `0 1 1 2`. Its last eight
/// cells were correct, which is exactly why it survived review — see
/// `published_strip_head_was_wrong` below.
pub fn correct_fib_strip_f1_12() -> String {
    fib_strip(12)
}

/// The visible parity law: `F(n)` is even exactly when `3 | n`.
///
/// In balanced ternary every power of three is odd, so a value's parity is the
/// parity of how many non-zero digits it carries. Signs are irrelevant.
pub fn fib_parity_law_holds(n: usize) -> bool {
    let value = fib_nand(n);
    let marks = balanced_ternary(value as i64)
        .chars()
        .filter(|c| *c != '0')
        .count();
    let even = value.is_multiple_of(2);
    even == marks.is_multiple_of(2) && even == n.is_multiple_of(3)
}

// ============================================================================
// 5. THE WAKING SEAL
// ============================================================================

/// Domain-separated digest over a compacted state.
///
/// `previous` chains epochs; `pruned` carries the tombstone count so that a
/// state which forgot nothing cannot claim a seal from one that pruned. See the
/// module docs: this is SHA-256, not BLAKE3.
pub fn waking_seal(
    previous: &[u8; 32],
    state: &[ComplexWord64],
    pruned: u64,
    phase_lock_q16_16: i32,
) -> [u8; 32] {
    let mut transcript = Vec::with_capacity(32 + state.len() * 32 + 16);
    transcript.extend_from_slice(b"memnet.complex_ternary.waking_seal.v1\0");
    transcript.extend_from_slice(previous);
    for word in state {
        transcript.extend_from_slice(&word.r_pos.to_le_bytes());
        transcript.extend_from_slice(&word.r_neg.to_le_bytes());
        transcript.extend_from_slice(&word.i_pos.to_le_bytes());
        transcript.extend_from_slice(&word.i_neg.to_le_bytes());
    }
    transcript.extend_from_slice(&pruned.to_le_bytes());
    transcript.extend_from_slice(&phase_lock_q16_16.to_le_bytes());
    sha256(&transcript)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_contract(w: &ComplexWord64, x: &ComplexWord64) -> (i32, i32) {
        let (mut re, mut im) = (0i32, 0i32);
        for i in 0..64 {
            let (wr, wi) = (w.real_trit(i) as i32, w.imag_trit(i) as i32);
            let (xr, xi) = (x.real_trit(i) as i32, x.imag_trit(i) as i32);
            re += wr * xr - wi * xi;
            im += wr * xi + wi * xr;
        }
        (re, im)
    }

    fn deterministic_word(seed: u64) -> ComplexWord64 {
        // SplitMix-ish: deterministic, no dependencies, no randomness in tests.
        let mut s = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut next = || {
            s ^= s >> 30;
            s = s.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            s ^= s >> 27;
            s.wrapping_mul(0x94D0_49BB_1331_11EB)
        };
        let (mut w, _) = (ComplexWord64::ZERO, 0);
        w.r_pos = next();
        w.i_pos = next();
        // Keep the invariant: never set a bit in both planes of the same axis.
        w.r_neg = next() & !w.r_pos;
        w.i_neg = next() & !w.i_pos;
        w
    }

    #[test]
    fn a_fresh_word_is_valid_and_a_stray_bit_is_caught() {
        let w = ComplexWord64 {
            r_pos: 0b1010,
            r_neg: 0b0100,
            i_pos: 0,
            i_neg: 0,
        };
        assert!(w.is_valid());
        assert_eq!(w.parity_fault(), 0);

        let mut broken = w;
        broken.r_neg |= 0b0010; // node 1 now claims both +1 and -1
        assert!(!broken.is_valid());
        assert_eq!(broken.parity_fault(), 0b0010);
    }

    #[test]
    fn the_dead_zone_is_the_minimax_boundary() {
        assert_eq!(quantise(0.51, 0.5), 1);
        assert_eq!(quantise(-0.51, 0.5), -1);
        assert_eq!(quantise(0.5, 0.5), 0, "exactly at the boundary holds");
        assert_eq!(quantise(-0.5, 0.5), 0);
        assert_eq!(quantise(0.0, 0.5), 0);
    }

    #[test]
    fn memoryless_packing_places_trits_by_sign() {
        let u = [3.0, -3.0, 0.1, 0.0, -0.9, 0.0, 0.0, 0.0];
        let v = [0.0, 0.0, 0.0, -2.0, 0.0, 1.5, 0.0, 0.0];
        let words = pack_row_memoryless(
            &u,
            &v,
            PackConfig {
                gamma: 2.0,
                carrier_rad: 0.0,
            },
        );

        assert_eq!(words.len(), 1);
        assert_eq!(words[0].real_trit(0), 1);
        assert_eq!(words[0].real_trit(1), -1);
        assert_eq!(words[0].real_trit(2), 0, "0.1 is inside the dead zone");
        assert_eq!(words[0].imag_trit(3), -1);
        assert_eq!(words[0].imag_trit(5), 1);
        assert!(words[0].is_valid());
    }

    #[test]
    fn pruned_nodes_never_wake_under_error_diffusion() {
        // A long run of zeros surrounded by loud signal: if the gate were
        // missing, carried error would light these up.
        let mut u = vec![0.0f32; 64];
        let mut v = vec![0.0f32; 64];
        for i in 0..8 {
            u[i] = 5.0 - i as f32 * 0.1;
            v[i] = -5.0 + i as f32 * 0.1;
        }

        let words = pack_row_sigma_delta(
            &u,
            &v,
            PackConfig {
                gamma: 2.0,
                carrier_rad: 0.0,
            },
        );
        let word = words[0];

        for i in 8..64 {
            assert_eq!(word.real_trit(i), 0, "node {i} real component woke up");
            assert_eq!(word.imag_trit(i), 0, "node {i} imag component woke up");
        }
    }

    /// Band-limited residual energy: the honest measurement for a noise-shaping
    /// claim. A single-bin correlation at the carrier can be gamed by a phase
    /// offset; summing |DFT|² across the in-band bins cannot.
    fn in_band_error(u: &[f32], v: &[f32], words: &[ComplexWord64], gamma: f32, half: f32) -> f64 {
        let n = u.len();
        let reconstruct = |i: usize| {
            let (word, bit) = (&words[i / 64], 1u64 << (i % 64));
            let re = if word.r_pos & bit != 0 {
                gamma
            } else if word.r_neg & bit != 0 {
                -gamma
            } else {
                0.0
            };
            let im = if word.i_pos & bit != 0 {
                gamma
            } else if word.i_neg & bit != 0 {
                -gamma
            } else {
                0.0
            };
            (re, im)
        };

        let bins = 13usize;
        let mut total = 0.0f64;
        for b in 0..bins {
            let k = -half + 2.0 * half * b as f32 / (bins - 1) as f32;
            let (mut re, mut im) = (0.0f64, 0.0f64);
            for i in 0..n {
                let (qr, qi) = reconstruct(i);
                let (er, ei) = ((u[i] - qr) as f64, (v[i] - qi) as f64);
                let (c, s) = ((k * i as f32).cos() as f64, (k * i as f32).sin() as f64);
                re += er * c + ei * s;
                im += ei * c - er * s;
            }
            total += re * re + im * im;
        }
        total
    }

    #[test]
    fn sigma_delta_moves_the_error_out_of_the_carrier_band() {
        // A slow carrier wave: the semantic case the shaping exists for.
        // Note the regime. Shaping only helps when the trit step is the same
        // order as the signal amplitude; with a coarse grid relative to the
        // signal both quantisers are simply bad and the comparison is noise.
        let n = 512usize;
        let k0 = 0.15f32;
        let amplitude = 1.6f32;
        let cfg = PackConfig {
            gamma: amplitude,
            carrier_rad: k0,
        };

        let u: Vec<f32> = (0..n).map(|i| amplitude * (k0 * i as f32).cos()).collect();
        let v: Vec<f32> = (0..n).map(|i| amplitude * (k0 * i as f32).sin()).collect();

        let shaped = in_band_error(&u, &v, &pack_row_sigma_delta(&u, &v, cfg), cfg.gamma, 0.3);
        let plain = in_band_error(&u, &v, &pack_row_memoryless(&u, &v, cfg), cfg.gamma, 0.3);

        assert!(
            shaped < plain * 0.5,
            "shaping should collapse in-band error, not shuffle it: shaped {shaped:.1} vs plain {plain:.1}"
        );
    }

    #[test]
    fn packing_is_deterministic() {
        let u: Vec<f32> = (0..128).map(|i| ((i % 11) as f32 - 5.0) * 0.4).collect();
        let v: Vec<f32> = (0..128).map(|i| ((i % 7) as f32 - 3.0) * 0.6).collect();
        let cfg = PackConfig {
            gamma: 1.5,
            carrier_rad: 0.1,
        };
        assert_eq!(
            pack_row_sigma_delta(&u, &v, cfg),
            pack_row_sigma_delta(&u, &v, cfg)
        );
    }

    #[test]
    fn contraction_matches_a_schoolbook_reference() {
        for seed in 0..64u64 {
            let (w, x) = (deterministic_word(seed), deterministic_word(seed * 7 + 3));
            assert!(w.is_valid() && x.is_valid());
            assert_eq!(
                contract_word(&w, &x),
                reference_contract(&w, &x),
                "seed {seed}"
            );
        }
    }

    #[test]
    fn contraction_with_the_conjugate_is_the_energy() {
        let w = deterministic_word(42);
        // w · conj(w) is real; w · w would be w² and is not what we want here.
        let conj = ComplexWord64 {
            r_pos: w.r_pos,
            r_neg: w.r_neg,
            i_pos: w.i_neg,
            i_neg: w.i_pos,
        };
        let (re, im) = contract_word(&w, &conj);

        let mut energy = 0i32;
        for i in 0..64 {
            energy += (w.real_trit(i) as i32).pow(2) + (w.imag_trit(i) as i32).pow(2);
        }
        assert_eq!(re, energy, "w · conj(w) is real and equals |w|²");
        assert_eq!(im, 0, "no imaginary part when contracting with itself");
    }

    #[test]
    fn q16_16_scaling_is_pinned() {
        assert_eq!(scale_q16_16(1, 1.0), 65_536);
        assert_eq!(scale_q16_16(2, 0.5), 65_536);
        assert_eq!(scale_q16_16(0, 9.0), 0);
        assert_eq!(
            scale_q16_16(i64::MAX, 1.0),
            i32::MAX,
            "saturates, never wraps"
        );
    }

    #[test]
    fn every_boolean_gate_is_built_from_nand() {
        for a in [false, true] {
            for b in [false, true] {
                assert_eq!(and(a, b), !nand(a, b));
                assert_eq!(or(a, b), !(!a && !b));
                assert_eq!(xor(a, b), a != b);
                assert_eq!(not(a), !a);
            }
        }
    }

    #[test]
    fn the_nand_adder_is_a_correct_64_bit_adder() {
        let cases = [
            (0u64, 0u64),
            (1, 1),
            (0xFFFF_FFFF, 1),
            (0x7FFF_FFFF_FFFF_FFFF, 1),
            (u64::MAX, 1), // wraps to 0, by construction
            (0xDEAD_BEEF, 0x1234_5678),
        ];
        for (a, b) in cases {
            assert_eq!(add_nand(a, b), a.wrapping_add(b), "{a:#x} + {b:#x}");
        }
    }

    #[test]
    fn the_gate_cascade_computes_fibonacci() {
        let table = [
            0u64, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610,
        ];
        for (n, expected) in table.iter().enumerate() {
            assert_eq!(fib_nand(n), *expected, "F({n})");
        }
        assert_eq!(fib_nand(39), 63_245_986);
    }

    #[test]
    fn balanced_ternary_round_trips_including_negatives() {
        for value in -200i64..=200 {
            let text = balanced_ternary(value);
            assert!(
                text.chars().all(|c| matches!(c, '-' | '0' | '+')),
                "{value} encoded as {text}"
            );
            assert_eq!(
                parse_balanced_ternary(&text),
                Some(value),
                "{value} -> {text}"
            );
        }
    }

    #[test]
    fn published_examples_decode_exactly() {
        // The worked examples from the note, LSB-first.
        for (text, value) in [
            ("--+", 5i64),
            ("-0+", 8),
            ("+++", 13),
            ("0+-+", 21),
            ("00+--+", 144),
        ] {
            assert_eq!(parse_balanced_ternary(text), Some(value), "{text}");
            assert_eq!(balanced_ternary(value), text, "{value}");
        }
    }

    #[test]
    fn the_visible_parity_law_holds_to_f39() {
        for n in 0..=39 {
            assert!(fib_parity_law_holds(n), "parity law broke at n = {n}");
        }
    }

    #[test]
    fn published_strip_head_was_wrong() {
        // Regression for the provenance strip that shipped with the paper. Its
        // tail is Fibonacci; its head is not, and eight correct cells are
        // exactly what made it survive review.
        let published_tail: Vec<i64> = "--+ -0+ +++ 0+-+ +-++ +00-+ -0+0+ 00+--+"
            .split(' ')
            .map(|cell| parse_balanced_ternary(cell).unwrap())
            .collect();
        assert_eq!(published_tail, vec![5, 8, 13, 21, 34, 55, 89, 144]);

        let published_head: Vec<i64> = "00+ 00+ 0-+ 00+"
            .split(' ')
            .map(|cell| parse_balanced_ternary(cell).unwrap())
            .collect();
        assert_eq!(published_head, vec![9, 9, 6, 9], "this is what shipped");
        assert_ne!(
            published_head,
            vec![0, 1, 1, 2],
            "and this is what it should be"
        );

        // The corrected strip, from the encoder itself.
        assert_eq!(
            correct_fib_strip_f1_12(),
            "+ + -+ 0+ --+ -0+ +++ 0+-+ +-++ +00-+ -0+0+ 00+--+"
        );
    }

    #[test]
    fn the_waking_seal_chains_and_is_sensitive_to_every_input() {
        let state = vec![deterministic_word(1), deterministic_word(2)];
        let base = waking_seal(&[0u8; 32], &state, 7, 65_536);

        assert_eq!(
            base,
            waking_seal(&[0u8; 32], &state, 7, 65_536),
            "deterministic"
        );
        assert_ne!(
            base,
            waking_seal(&[1u8; 32], &state, 7, 65_536),
            "epoch chains"
        );
        assert_ne!(
            base,
            waking_seal(&[0u8; 32], &state, 8, 65_536),
            "tombstones count"
        );
        assert_ne!(
            base,
            waking_seal(&[0u8; 32], &state, 7, 65_537),
            "phase lock counts"
        );

        let mut tampered = state.clone();
        tampered[0].r_pos ^= 1;
        assert_ne!(
            base,
            waking_seal(&[0u8; 32], &tampered, 7, 65_536),
            "state counts"
        );
    }
}
