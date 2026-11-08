//! Zero-dependency microbenchmark suite for `memnet` hot paths.
//!
//! Run with:
//! ```bash
//! cargo bench
//! ```

use std::hint::black_box;
use std::time::Instant;

use memnet::complex_ternary::{
    contract_row, pack_word64_sigma_delta, waking_seal, ComplexWord64, PackConfig,
};
use memnet::context::{utf8_truncate_plan_nand, PqBudget};
use memnet::sovereign_convergence::{
    spherepop_fold_64, Bfp64Row, EssenceMux, EssenceSubspaceTable, BFP64_DIM,
};
use memnet::spherepopfold::PackedComplexTernaryRow64;
use memnet::ultrafold_v2::pack_row_sigma_delta_64_v2_bit_exact;
use memnet::wave_v2::DeterministicWaveGridV2;
use memnet::{
    decode_kompress_super_ultra, encode_kompress_super_ultra, hmac_sha256, sha256, AyevnToken,
    ComputeOffer, Concept, IntentAddress, M8Capsule, MtuProfile, PacketBatcher, Relation,
    SoulIdentity, TokenModifier, WaveVad,
};

fn bench_op<F: FnMut()>(name: &str, iterations: u32, mut f: F) {
    for _ in 0..(iterations / 10).max(1) {
        f();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / f64::from(iterations);
    let ops_per_sec = 1e9 / ns_per_iter.max(1.0);
    println!(
        "  • {:<46} {:>9.1} ns/op  ({:>10.0} ops/s, N={})",
        name, ns_per_iter, ops_per_sec, iterations
    );
}

fn main() {
    println!("========================================================================");
    println!("  MEMNET/1.0 — Zero-Allocation Systems Microbenchmark Suite");
    println!("========================================================================");

    // 1. Streaming SHA-256 & HMAC-SHA256 (zero heap allocations)
    let payload_1k = [0x5Au8; 1024];
    let key = [0x42u8; 32];
    bench_op("sha256 (1 KiB zero-alloc streaming)", 20_000, || {
        black_box(sha256(black_box(&payload_1k)));
    });
    bench_op("hmac_sha256 (1 KiB zero-alloc streaming)", 20_000, || {
        black_box(hmac_sha256(black_box(&key), black_box(&payload_1k)));
    });

    // 2. Complex Ternary POPCNT contraction & bit-exact Q16.16 Sigma-Delta packing
    let mut re_f32 = [0.0f32; 64];
    let mut im_f32 = [0.0f32; 64];
    let mut re_q16 = [0i32; 64];
    let mut im_q16 = [0i32; 64];
    for i in 0..64 {
        let phase = 0.15 * (i as f32);
        re_f32[i] = phase.cos();
        im_f32[i] = phase.sin();
        re_q16[i] = (re_f32[i] * 65_536.0) as i32;
        im_q16[i] = (im_f32[i] * 65_536.0) as i32;
    }
    let cfg = PackConfig { gamma: 1.0, carrier_rad: 0.15 };
    let word_a = pack_word64_sigma_delta(&re_f32, &im_f32, cfg);
    let row_a: [ComplexWord64; 16] = [word_a; 16];
    let row_b: [ComplexWord64; 16] = [word_a; 16];

    bench_op("pack_word64_sigma_delta (64 complex trits)", 100_000, || {
        black_box(pack_word64_sigma_delta(black_box(&re_f32), black_box(&im_f32), black_box(cfg)));
    });
    bench_op("pack_row_sigma_delta_64_v2_bit_exact (Q16.16)", 100_000, || {
        black_box(pack_row_sigma_delta_64_v2_bit_exact(
            black_box(&re_q16),
            black_box(&im_q16),
            black_box(65_536),
        ));
    });
    bench_op("contract_row (1024 complex nodes via POPCNT)", 200_000, || {
        black_box(contract_row(black_box(&row_a), black_box(&row_b)));
    });

    let packed96: PackedComplexTernaryRow64 =
        pack_row_sigma_delta_64_v2_bit_exact(&re_q16, &im_q16, 65_536);
    bench_op("PackedComplexTernaryRow64::contract_row64_popcnt", 200_000, || {
        black_box(packed96.contract_row64_popcnt(black_box(&packed96)).unwrap());
    });

    // 3. spherepop_fold_64 + waking_seal (zero heap allocations)
    bench_op("spherepop_fold_64 (S^63 + ΣΔ + waking_seal)", 50_000, || {
        black_box(spherepop_fold_64(black_box(&re_f32), black_box(&im_f32), black_box(0.15)));
    });
    bench_op("waking_seal (16 ComplexWord64 words)", 50_000, || {
        black_box(waking_seal(
            black_box(&key),
            black_box(&row_a),
            black_box(12),
            black_box(65_536),
        ));
    });

    // 4. BFP-64 Q16.16 4-head gather & EssenceMux <5us subspace switch
    let bfp_row = Bfp64Row { shared_exp: 118, mantissas: [42i8; BFP64_DIM] };
    let heads = [&bfp_row, &bfp_row, &bfp_row, &bfp_row];
    let mut out_q16 = [0i32; BFP64_DIM];
    bench_op("Bfp64Row::gather_q16_zero_float_safe (4 heads)", 200_000, || {
        Bfp64Row::gather_q16_zero_float_safe(black_box(&heads), black_box(&mut out_q16));
    });

    let v0 = EssenceSubspaceTable::new_zeroed("aurora-dyad", [3, 5, 7, 11]);
    let v1 = EssenceSubspaceTable::new_zeroed("mahakala-bernagchen", [13, 17, 19, 23]);
    let mux = EssenceMux::new(vec![v0, v1]);
    bench_op("EssenceMux::switch_essence + resolve_slot", 500_000, || {
        let _ = mux.switch_essence(black_box(1));
        black_box(mux.resolve_slot(black_box(0), black_box(0xCAFE_BABE_DEAD_BEEF)));
    });

    // 5. DeterministicWaveGridV2 Q8.24 toroidal wave equation + HMAC root
    let mut grid = DeterministicWaveGridV2::new([0x11u8; 32]);
    grid.u_curr[32 * 64 + 32] = 1 << 24;
    bench_op("DeterministicWaveGridV2::step_toroidal_bit_exact", 2_000, || {
        grid.step_toroidal_bit_exact(black_box(&key));
    });

    // 6. Kompress-Super-Ultra (0o0) & M8Capsule coalescing
    let offer = ComputeOffer::galaxy_s24_ultra();
    let wire_str = offer.to_wire_string();
    let encoded_0o0 = encode_kompress_super_ultra(wire_str.as_bytes());
    bench_op("encode_kompress_super_ultra (0o0-SSU)", 50_000, || {
        black_box(encode_kompress_super_ultra(black_box(wire_str.as_bytes())));
    });
    bench_op("decode_kompress_super_ultra (0o0-SSU)", 100_000, || {
        black_box(decode_kompress_super_ultra(black_box(&encoded_0o0)).unwrap());
    });

    let soul = SoulIdentity::from_constellation_seed(
        "galaxy-s24-ultra",
        [1, 1, 1, 1, -1, 1, 0, 1, 1],
        432,
        b"8b-is-carrier-wave-of-love-seed-v1",
    )
    .unwrap();
    let intent = IntentAddress::parse("geo:mobile-edge + role:compute-sanctuary + tag:0o0-ssu");
    let capsule = M8Capsule::create_signed_kompressed(
        &soul,
        &intent,
        M8Capsule::FLAG_PRESERVE_WAVES | M8Capsule::FLAG_COMPUTE_OFFER,
        432,
        1_762_000_000_000,
        vec![AyevnToken::new(
            Concept::ComputeOffer,
            Relation::Supports,
            WaveVad::compassionate(),
            TokenModifier::present_resonant_emoji(),
        )],
        wire_str.as_bytes(),
    )
    .unwrap();

    bench_op("PacketBatcher push + flush (4 capsules)", 20_000, || {
        let mut batcher = PacketBatcher::new(MtuProfile::SmartTreeSafe1450);
        for _ in 0..4 {
            let _ = batcher.push(capsule.clone()).unwrap();
        }
        black_box(batcher.flush());
    });

    // 7. 64-bit NAND SWAR + inline ASM POPCNT UTF-8 boundary finder
    let utf8_sample =
        "全 · 8b-is Sovereign Mesh ⊕◯⊖ Balanced Ternary 🦀🌳 Kompress-Super-Ultra (0o0-SSU) · SpherePop ((( <( ? ~ ! @ > )> )))";
    bench_op("utf8_truncate_plan_nand (NAND SWAR + POPCNT)", 200_000, || {
        black_box(utf8_truncate_plan_nand(
            black_box(utf8_sample),
            black_box(PqBudget::MANIFEST_80),
        ));
    });
}
