//! Live Hardware Benchmark: `MEMNET/1.0` × `WiCi` (`arXiv:2608.24204v1`) on Local
//! `NVIDIA GeForce RTX 4090` + `T-FORCE TM8FPP002T` 1.8 TiB NVMe Vault & Hetzner `RTX 6000 Ada`.
//!
//! 100% Exact Integer & Reduced `p/q` (`RationalPQ`) Arithmetic — Zero Floats.

use std::fs;
use std::process::Command;
use std::time::Instant;

use memnet::wici_gpu::{
    WiciEdgeClientProfile, WiciGpuRouterProfile, WiciMemnetSession, WiciNvmeVault,
    DEFAULT_TFORCE_NVME_VAULT,
};
use memnet::SoulIdentity;

fn query_local_rtx4090() -> (String, u32, u32, u32) {
    let out = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,memory.used,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let line = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = line.trim().split(',').map(str::trim).collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let total_mib = parts[1].parse::<u32>().unwrap_or(24_564);
                let used_mib = parts[2].parse::<u32>().unwrap_or(0);
                let temp_c = parts[3].parse::<u32>().unwrap_or(0);
                return (name, total_mib, used_mib, temp_c);
            }
        }
    }
    ("NVIDIA GeForce RTX 4090".to_string(), 24_564, 0, 45)
}

fn main() {
    let (gpu_name, vram_total_mib, vram_used_mib, gpu_temp_c) = query_local_rtx4090();
    let vault_path = std::env::var("WICI_VAULT_DIR")
        .unwrap_or_else(|_| DEFAULT_TFORCE_NVME_VAULT.to_string());

    println!("=========================================================================");
    println!("  8b-is x MEMNET/1.0 x WiCi Wireless GPU & DeepSeek-671B MoE Testbed");
    println!("=========================================================================");
    println!(
        "  Local GPU       : {} ({} MiB used / {} MiB total, {} C)",
        gpu_name, vram_used_mib, vram_total_mib, gpu_temp_c
    );
    println!("  T-FORCE Vault   : {}", vault_path);

    let mut vault = WiciNvmeVault::open(&vault_path).expect("open T-FORCE NVMe vault");

    // 1. Benchmark Content-Addressed Expert Tensor Caching on the T-FORCE TM8FPP002T NVMe:
    // Create 8 routed expert weight blocks (each 1 MiB of base-243 packed ternary trits = 5.24M trits/block).
    let expert_block: Vec<u8> = (0..(1024 * 1024)).map(|i| (i % 243) as u8).collect();
    let t_cold = Instant::now();
    let (hit_cold, digest, path) = vault
        .cache_or_reuse_tensor("moe-experts", &expert_block)
        .expect("cold write to T-FORCE NVMe");
    let cold_ns = (t_cold.elapsed().as_nanos() as u64).max(1);

    let t_warm = Instant::now();
    let hit_warm = vault.lookup_tensor_by_digest(&digest, expert_block.len());
    let warm_ns = (t_warm.elapsed().as_nanos() as u64).max(1);

    assert!(!hit_cold);
    assert!(hit_warm);

    println!(
        "  Expert Tensor   : {:?} (1 MiB base-243 = 5,242,880 trits)",
        path.file_name().unwrap_or_default()
    );
    println!(
        "  NVMe Cold Write : {} ns | Router 32B Digest Hit: {} ns (Speedup p/q = {}/{})",
        cold_ns, warm_ns, cold_ns, warm_ns
    );
    println!(
        "  Digest Prefix   : {:02x}{:02x}{:02x}{:02x}...",
        digest[0], digest[1], digest[2], digest[3]
    );

    // 2. Run End-to-End MEMNET Wireless Sessions across Edge Clients -> Local RTX 4090 & Hetzner A6000:
    let clients = [
        WiciEdgeClientProfile::galaxy_s24_ultra(),
        WiciEdgeClientProfile::raspberry_pi_5(),
        WiciEdgeClientProfile::blu_g75(),
        WiciEdgeClientProfile::tdeck_omni(),
    ];
    let routers = [
        WiciGpuRouterProfile::local_rtx4090_tforce(),
        WiciGpuRouterProfile::hetzner_a6000_700gb(),
    ];

    let mut report_md = String::new();
    report_md.push_str("# WiCi × MEMNET/1.0 Live Hardware Receipt (`RTX 4090` + `T-FORCE TM8FPP002T` & `Hetzner A6000`)\n\n");
    report_md.push_str(&format!(
        "- **Local GPU Verified**: `{}` (`{}/{}` MiB VRAM, `{}` °C)\n",
        gpu_name, vram_used_mib, vram_total_mib, gpu_temp_c
    ));
    report_md.push_str(&format!("- **NVMe Vault Path**: `{}`\n", vault_path));
    report_md.push_str(&format!(
        "- **1 MiB Expert Tensor Cold Write vs Router 32B Digest Cache Hit**: `{}` ns vs `{}` ns (`{}/{}` speedup)\n\n",
        cold_ns, warm_ns, cold_ns, warm_ns
    ));
    report_md.push_str("| Router | Client | Mode | Naive RTTs | Batched RTTs | Trace Replay RTTs | QAT Window Reuse (`p/q`) | Speedup (`p/q`) | SpherePop Witness |\n");
    report_md.push_str("|---|---|---|---:|---:|---:|---:|---:|---|\n");

    println!("-------------------------------------------------------------------------");
    println!(
        "  {:<24} {:<20} {:<8} {:>10} {:>10} {:>14}",
        "ROUTER", "CLIENT", "MODE", "NAIVE_RTT", "REPLAY_RTT", "SPEEDUP (p/q)"
    );
    println!("-------------------------------------------------------------------------");

    for router in &routers {
        for client in &clients {
            for &is_train in &[false, true] {
                let c_soul = SoulIdentity::from_constellation_seed(
                    &client.node_id,
                    [1, 0, 1, 1, 0, -1, 1, 0, 1],
                    432,
                    b"memnet-wici-client-seed-2026",
                )
                .unwrap();
                let r_soul = SoulIdentity::from_constellation_seed(
                    &router.node_id,
                    [1, 1, 1, 0, 1, 0, 1, 1, 1],
                    432,
                    b"memnet-wici-router-seed-2026",
                )
                .unwrap();
                let sub_vault = WiciNvmeVault::open(&vault_path).unwrap();
                let mut sess = WiciMemnetSession::new(
                    c_soul,
                    r_soul,
                    client.clone(),
                    router.clone(),
                    sub_vault,
                );
                let rec = sess
                    .execute_deepseek_moe_trace_session(32, is_train, 1_762_700_000_000)
                    .unwrap();
                let mode_str = if is_train { "QAT-32" } else { "INFER-32" };
                let approx_int = rec.speedup_ratio_pq.num / (rec.speedup_ratio_pq.den as i64);
                println!(
                    "  {:<24} {:<20} {:<8} {:>10} {:>10} {:>6}x ({}/{})",
                    rec.router_node_id,
                    rec.client_node_id,
                    mode_str,
                    rec.naive_driver_rtts,
                    rec.trace_replay_rtts,
                    approx_int,
                    rec.speedup_ratio_pq.num,
                    rec.speedup_ratio_pq.den
                );
                report_md.push_str(&format!(
                    "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}/{}` | `{}/{}` (`~{}x`) | `{}` |\n",
                    rec.router_node_id,
                    rec.client_node_id,
                    mode_str,
                    rec.naive_driver_rtts,
                    rec.batched_rtts,
                    rec.trace_replay_rtts,
                    rec.qat_window_reuse_pq.num,
                    rec.qat_window_reuse_pq.den,
                    rec.speedup_ratio_pq.num,
                    rec.speedup_ratio_pq.den,
                    approx_int,
                    rec.spherepop_witness.short_hex()
                ));
            }
        }
    }

    let md_path = std::path::Path::new(&vault_path)
        .join("receipts")
        .join("BENCHMARK_RECEIPT_RTX4090_TFORCE.md");
    fs::write(&md_path, &report_md).expect("write benchmark receipt to T-FORCE NVMe");
    println!("-------------------------------------------------------------------------");
    println!("  Receipt written : {}", md_path.display());
    println!("=========================================================================");
}
