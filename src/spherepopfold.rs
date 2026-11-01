//! `memnet/src/spherepopfold.rs`
//! Section 6.1 (`8B-META-WP-v1.1` / `v2.0`): Carrier-Locked Sigma-Delta (ΣΔ)
//! 96-Byte Complex Ternary Row & Hardware Parity-Interlocked `POPCNT` Contraction.

#[repr(C, align(32))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedComplexTernaryRow64 {
    pub re_pos: u64,          // +1 real trit mask (8 B)
    pub re_neg: u64,          // -1 real trit mask (8 B)
    pub im_pos: u64,          // +1 imag trit mask (8 B)
    pub im_neg: u64,          // -1 imag trit mask (8 B)
    pub act_planes: [u64; 4], // Auxiliary two's-complement activation bit-planes (32 B)
    pub scale_q16: i32,       // Q16.16 radial normalization scale factor (4 B)
    pub parity_interlock: u8, // XOR-folded hardware parity byte (1 B)
    pub _pad: [u8; 27],       // Explicit alignment padding to exact 96-byte cache-line multiple
}

const _: () = assert!(std::mem::size_of::<PackedComplexTernaryRow64>() == 96);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpherepopError {
    ParityInterlockViolation { forbidden_mask: u64 },
    ChecksumMismatch { expected: u8, actual: u8 },
}

impl PackedComplexTernaryRow64 {
    /// Computes the 8-bit hardware parity byte across the four ternary bit-planes.
    #[inline(always)]
    pub const fn compute_parity_byte(re_pos: u64, re_neg: u64, im_pos: u64, im_neg: u64) -> u8 {
        let folded = re_pos
            ^ re_neg.rotate_left(13)
            ^ im_pos.rotate_left(29)
            ^ im_neg.rotate_left(47);
        (folded.count_ones() as u8) ^ 0xA5
    }

    /// Enforces Equation 17: `(re_+ & re_-) | (im_+ & im_-) == 0` plus parity checksum.
    #[inline(always)]
    pub fn verify_interlock(&self) -> Result<(), SpherepopError> {
        let forbidden = (self.re_pos & self.re_neg) | (self.im_pos & self.im_neg);
        if forbidden != 0 {
            return Err(SpherepopError::ParityInterlockViolation {
                forbidden_mask: forbidden,
            });
        }
        let expected =
            Self::compute_parity_byte(self.re_pos, self.re_neg, self.im_pos, self.im_neg);
        if self.parity_interlock != expected {
            return Err(SpherepopError::ChecksumMismatch {
                expected,
                actual: self.parity_interlock,
            });
        }
        Ok(())
    }

    /// Multiplication-free complex ternary inner product `<row_a, conj(row_b)>`
    /// using purely bitwise AND and hardware `POPCNT` (`count_ones()`).
    #[inline(always)]
    pub fn contract_row64_popcnt(&self, rhs: &Self) -> Result<(i32, i32), SpherepopError> {
        self.verify_interlock()?;
        rhs.verify_interlock()?;

        // Real part of <a, conj(b)>: (a_re * b_re) + (a_im * b_im)
        let re_plus = ((self.re_pos & rhs.re_pos).count_ones()
            + (self.re_neg & rhs.re_neg).count_ones()
            + (self.im_pos & rhs.im_pos).count_ones()
            + (self.im_neg & rhs.im_neg).count_ones()) as i32;

        let re_minus = ((self.re_pos & rhs.re_neg).count_ones()
            + (self.re_neg & rhs.re_pos).count_ones()
            + (self.im_pos & rhs.im_neg).count_ones()
            + (self.im_neg & rhs.im_pos).count_ones()) as i32;

        // Imaginary part of <a, conj(b)>: (a_im * b_re) - (a_re * b_im)
        let im_plus = ((self.im_pos & rhs.re_pos).count_ones()
            + (self.im_neg & rhs.re_neg).count_ones()
            + (self.re_neg & rhs.im_pos).count_ones()
            + (self.re_pos & rhs.im_neg).count_ones()) as i32;

        let im_minus = ((self.im_pos & rhs.re_neg).count_ones()
            + (self.im_neg & rhs.re_pos).count_ones()
            + (self.re_pos & rhs.im_pos).count_ones()
            + (self.re_neg & rhs.im_neg).count_ones()) as i32;

        Ok((re_plus - re_minus, im_plus - im_minus))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherepop_parity_interlock_catches_bitflip() {
        let re_p = 0b1010_0101u64;
        let re_n = 0b0101_1010u64; // Disjoint from re_p
        let im_p = 0b1100_0011u64;
        let im_n = 0b0011_1100u64; // Disjoint from im_p
        let parity = PackedComplexTernaryRow64::compute_parity_byte(re_p, re_n, im_p, im_n);

        let mut row = PackedComplexTernaryRow64 {
            re_pos: re_p,
            re_neg: re_n,
            im_pos: im_p,
            im_neg: im_n,
            act_planes: [0; 4],
            scale_q16: 65536,
            parity_interlock: parity,
            _pad: [0; 27],
        };
        assert!(row.verify_interlock().is_ok());

        // Self-contraction <row, conj(row)> has strictly positive real energy and zero imag part
        let (energy_re, energy_im) = row.contract_row64_popcnt(&row).expect("valid contraction");
        assert_eq!(energy_re, 16);
        assert_eq!(energy_im, 0);

        // Inject single-bit fault creating forbidden (+1, -1) overlap on bit 0
        row.re_pos |= 1;
        row.re_neg |= 1;
        assert_eq!(
            row.verify_interlock(),
            Err(SpherepopError::ParityInterlockViolation { forbidden_mask: 1 })
        );
    }
}
