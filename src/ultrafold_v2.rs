//! `memnet/src/ultrafold_v2.rs`
//! `8B-META-WP-v2.0`: Bit-Exact Dyadic Sigma-Delta (ΣΔ) Complex Ternary Quantizer.
//! Guarantees identical `PackedComplexTernaryRow64` bit-planes across ARM64, x86_64, and FPGA RTL.

use crate::spherepopfold::PackedComplexTernaryRow64;

/// Dyadic `Q16.16` approximation of `cos(π/4) = sin(π/4) = 1/√2 ≈ 46341 / 65536`.
pub const INV_SQRT2_Q16: i64 = 46_341;
/// Optimal 9-state complex ternary threshold circle `r_opt ≈ 1.3066` in `Q16.16` (`85,629 / 65536`).
pub const R_OPT_SCALE_Q16: i64 = 85_629;

/// Folds 64 complex `Q16.16` wave samples `(re_q16, im_q16)` into a 96-byte
/// `PackedComplexTernaryRow64` using bit-exact integer carrier-shifted ΣΔ modulation.
#[inline]
pub fn pack_row_sigma_delta_64_v2_bit_exact(
    re_q16: &[i32; 64],
    im_q16: &[i32; 64],
    gamma_q16: i32,
) -> PackedComplexTernaryRow64 {
    let thresh = ((gamma_q16 as i64) * R_OPT_SCALE_Q16) >> 16;
    let half_thresh = (thresh >> 1).max(1);

    let mut re_pos: u64 = 0;
    let mut re_neg: u64 = 0;
    let mut im_pos: u64 = 0;
    let mut im_neg: u64 = 0;

    // Complex quantization error state e_{n-1} in Q16.16
    let mut err_re: i64 = 0;
    let mut err_im: i64 = 0;

    for n in 0..64 {
        // 1. Rotate previous error by e^{i * pi/4}:
        //    rot_re = (err_re - err_im) / sqrt(2)
        //    rot_im = (err_re + err_im) / sqrt(2)
        let rot_re = ((err_re - err_im) * INV_SQRT2_Q16) >> 16;
        let rot_im = ((err_re + err_im) * INV_SQRT2_Q16) >> 16;

        let u_re = (re_q16[n] as i64) + rot_re;
        let u_im = (im_q16[n] as i64) + rot_im;

        // 2. 9-state complex ternary quantizer Q_9(u_n / gamma) in {-1, 0, +1} + i{-1, 0, +1}
        let q_re = if u_re > half_thresh {
            re_pos |= 1u64 << n;
            thresh
        } else if u_re < -half_thresh {
            re_neg |= 1u64 << n;
            -thresh
        } else {
            0
        };

        let q_im = if u_im > half_thresh {
            im_pos |= 1u64 << n;
            thresh
        } else if u_im < -half_thresh {
            im_neg |= 1u64 << n;
            -thresh
        } else {
            0
        };

        // 3. Feedback quantization residual into carrier-locked ΣΔ integrator
        err_re = u_re - q_re;
        err_im = u_im - q_im;
    }

    let parity = PackedComplexTernaryRow64::compute_parity_byte(re_pos, re_neg, im_pos, im_neg);

    PackedComplexTernaryRow64 {
        re_pos,
        re_neg,
        im_pos,
        im_neg,
        act_planes: [
            re_pos ^ im_pos,
            re_neg ^ im_neg,
            re_pos.wrapping_add(im_neg),
            re_neg.wrapping_add(im_pos),
        ],
        scale_q16: gamma_q16,
        parity_interlock: parity,
        _pad: [0u8; 27],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultrafold_v2_dyadic_sigma_delta_is_bit_exact_and_interlock_valid() {
        let mut re_q16 = [0i32; 64];
        let mut im_q16 = [0i32; 64];
        for i in 0..64 {
            re_q16[i] = ((i as i32) - 32) * 2048;
            im_q16[i] = (32 - (i as i32)) * 2048;
        }
        let row1 = pack_row_sigma_delta_64_v2_bit_exact(&re_q16, &im_q16, 65_536);
        let row2 = pack_row_sigma_delta_64_v2_bit_exact(&re_q16, &im_q16, 65_536);
        assert_eq!(row1, row2);
        assert!(row1.verify_interlock().is_ok());
        let (re_e, im_e) = row1
            .contract_row64_popcnt(&row2)
            .expect("valid POPCNT contraction");
        assert!(re_e > 0);
        assert_eq!(im_e, 0);
    }
}
