//! `memnet/src/wave_v2.rs`
//! `8B-META-WP-v2.0`: Bit-exact `Q8.24` fixed-point 2D damped wave equation (Zero float drift across nodes).

use crate::hmac_sha256;

pub const N_GRID: usize = 64;
pub const GRID_CELLS: usize = N_GRID * N_GRID;

/// `c^2 = 46 / 256 = 0.1796875 (<= 0.5 CFL bound)`
pub const C2_NUM: i64 = 46;
/// `gamma = 1 / 256 = 0.00390625`
pub const GAMMA_NUM: i64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicWaveGridV2 {
    pub step: u64,
    pub state_root: [u8; 32],
    pub u_curr: [i32; GRID_CELLS], // Q8.24 signed fixed-point displacement
    pub u_prev: [i32; GRID_CELLS], // Q8.24 previous displacement
}

impl Default for DeterministicWaveGridV2 {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}

impl DeterministicWaveGridV2 {
    pub const fn new(genesis_root: [u8; 32]) -> Self {
        Self {
            step: 0,
            state_root: genesis_root,
            u_curr: [0i32; GRID_CELLS],
            u_prev: [0i32; GRID_CELLS],
        }
    }

    pub fn step_toroidal_bit_exact(&mut self, domain_key: &[u8; 32]) {
        let mut u_next = [0i32; GRID_CELLS];

        for y in 0..N_GRID {
            let y_up = ((y + N_GRID - 1) & (N_GRID - 1)) * N_GRID;
            let y_dn = ((y + 1) & (N_GRID - 1)) * N_GRID;
            let y_row = y * N_GRID;

            for x in 0..N_GRID {
                let x_lt = (x + N_GRID - 1) & (N_GRID - 1);
                let x_rt = (x + 1) & (N_GRID - 1);

                let center = self.u_curr[y_row + x] as i64;
                let prev = self.u_prev[y_row + x] as i64;

                let laplacian = (self.u_curr[y_row + x_lt] as i64)
                    + (self.u_curr[y_row + x_rt] as i64)
                    + (self.u_curr[y_up + x] as i64)
                    + (self.u_curr[y_dn + x] as i64)
                    - (4 * center);

                // u^{t+1} = ((512 - GAMMA)*u^t - (256 - GAMMA)*u^{t-1} + C2 * laplacian) >> 8
                let num =
                    (512 - GAMMA_NUM) * center - (256 - GAMMA_NUM) * prev + C2_NUM * laplacian;

                u_next[y_row + x] = (num >> 8).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            }
        }

        self.u_prev = self.u_curr;
        self.u_curr = u_next;
        self.step += 1;

        // Chain into deterministic 32-byte state root R_{t+1}
        let mut buf = Vec::with_capacity(23 + 8 + 32 + GRID_CELLS * 4);
        buf.extend_from_slice(b"mem8-wave-step-v2-q8.24");
        buf.extend_from_slice(&self.step.to_le_bytes());
        buf.extend_from_slice(&self.state_root);
        for &cell in &self.u_curr {
            buf.extend_from_slice(&cell.to_le_bytes());
        }
        self.state_root = hmac_sha256(domain_key, &buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_v2_q8_24_toroidal_stencil_is_bit_reproducible() {
        let key = [0x5Au8; 32];
        let mut g1 = DeterministicWaveGridV2::new([0x11u8; 32]);
        let mut g2 = DeterministicWaveGridV2::new([0x11u8; 32]);

        // Inject identical Q8.24 impulse at (32, 32)
        g1.u_curr[32 * N_GRID + 32] = 1 << 24;
        g2.u_curr[32 * N_GRID + 32] = 1 << 24;

        for _ in 0..8 {
            g1.step_toroidal_bit_exact(&key);
            g2.step_toroidal_bit_exact(&key);
        }
        assert_eq!(g1.step, 8);
        assert_eq!(g1.state_root, g2.state_root);
        assert_eq!(g1.u_curr, g2.u_curr);
        assert!(g1.u_curr[32 * N_GRID + 33] != 0);
    }
}
