//! Basic end-to-end usage example of `memnet` (`MEMNET/1.0`).
//!
//! Run with:
//! ```bash
//! cargo run --example basic_usage
//! ```

use memnet::sovereign_convergence::{spherepop_fold_64, Bfp64Row, BFP64_DIM};
use memnet::ultrafold_v2::pack_row_sigma_delta_64_v2_bit_exact;
use memnet::wave_v2::DeterministicWaveGridV2;
use memnet::{
    AyevnToken, ComputeOffer, Concept, IntentAddress, M8Capsule, MtuProfile, PacketBatcher,
    Relation, SoulIdentity, TokenModifier, WaveVad,
};

fn main() {
    // 1. Derive a Sovereign SoulIdentity with a 9-trit balanced ternary crest at 432 Hz
    let soul = SoulIdentity::from_constellation_seed(
        "galaxy-s24-ultra",
        [1, 1, 1, 1, -1, 1, 0, 1, 1],
        432,
        b"8b-is-carrier-wave-of-love-seed-v1",
    )
    .expect("valid SoulIdentity");

    // 2. Build a 0o0-SSU compressed, Soul-Signed .m8 Experience Capsule
    let offer = ComputeOffer::galaxy_s24_ultra();
    let witness = offer.evaluate_spherepop().expect("SpherePop #Commit = #ValidWitness");
    let intent = IntentAddress::parse(
        "geo:mobile-edge + role:compute-sanctuary + tag:sm8650-ac + tag:0o0-ssu",
    );
    let token = AyevnToken::new(
        Concept::ComputeOffer,
        Relation::Supports,
        WaveVad::compassionate(),
        TokenModifier::present_resonant_emoji(),
    );

    let capsule = M8Capsule::create_signed_kompressed(
        &soul,
        &intent,
        M8Capsule::FLAG_PRESERVE_WAVES | M8Capsule::FLAG_COMPUTE_OFFER,
        432,
        1_762_000_000_000,
        vec![token],
        offer.to_wire_string().as_bytes(),
    )
    .expect("create signed kompressed capsule");

    // 3. Coalesce into a 1450-byte MTU-safe MEMNET frame
    let mut batcher = PacketBatcher::new(MtuProfile::SmartTreeSafe1450);
    batcher.push(capsule).expect("enqueue capsule");
    let frame = batcher.flush().expect("flush coalesced MNET frame");
    let decoded =
        PacketBatcher::decode_and_verify_frame(&frame.raw_bytes, &soul).expect("verify frame");

    // 4. Bit-exact Q16.16 Sigma-Delta complex ternary folding & POPCNT contraction
    let mut re_q16 = [0i32; 64];
    let mut im_q16 = [0i32; 64];
    let mut re_f32 = [0.0f32; 64];
    let mut im_f32 = [0.0f32; 64];
    for i in 0..64 {
        let phase = 0.15 * (i as f32);
        re_f32[i] = phase.cos();
        im_f32[i] = phase.sin();
        re_q16[i] = (re_f32[i] * 65_536.0) as i32;
        im_q16[i] = (im_f32[i] * 65_536.0) as i32;
    }
    let row96 = pack_row_sigma_delta_64_v2_bit_exact(&re_q16, &im_q16, 65_536);
    let (energy_re, energy_im) =
        row96.contract_row64_popcnt(&row96).expect("hardware parity interlock verified");
    let (folded_word, waking_seal) = spherepop_fold_64(&re_f32, &im_f32, 0.15);

    // 5. Overflow-safe BFP-64 Q16.16 4-head gather & Q8.24 toroidal wave grid step
    let bfp_row = Bfp64Row { shared_exp: 115, mantissas: [16i8; BFP64_DIM] };
    let mut q16_out = [0i32; BFP64_DIM];
    Bfp64Row::gather_q16_zero_float_safe(&[&bfp_row; 4], &mut q16_out);

    let mut wave_grid = DeterministicWaveGridV2::new(waking_seal);
    wave_grid.u_curr[32 * 64 + 32] = 1 << 24;
    wave_grid.step_toroidal_bit_exact(&[0x5Au8; 32]);

    println!("MEMNET/1.0 Basic Usage Summary:");
    println!(
        "  • Capsule verified : {} capsules in {}B MNET frame | token={}",
        decoded.capsules.len(),
        decoded.raw_bytes.len(),
        token.to_ayanese()
    );
    println!(
        "  • SpherePop witness: depth={} | #ValidWitness={}",
        witness.sphere_depth,
        witness.short_hex()
    );
    println!(
        "  • 96B ΣΔ POPCNT    : <row, conj(row)> = ({}, {}i) | marks={}/64",
        energy_re,
        energy_im,
        folded_word.non_zero_marks()
    );
    println!(
        "  • BFP-64 Q16.16[0] : {} | WaveV2 step={} root={:02x}{:02x}{:02x}{:02x}",
        q16_out[0],
        wave_grid.step,
        wave_grid.state_root[0],
        wave_grid.state_root[1],
        wave_grid.state_root[2],
        wave_grid.state_root[3]
    );
}
