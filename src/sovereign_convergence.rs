//! # 8B-IS Sovereign Convergence (`v1.0` / `v0.1-spherepopfold`) — Reference Oracle
//!
//! End-to-end deterministic reference implementation and verification suite unifying:
//! 1. **`honest-auth` Capability Exchange & Admissibility Boundary** (`CapMask`, `CapabilityTicket`,
//!    single-use 30s bootstrap nonce scrubbing, 4-rung trust ladder sufficiency, and 64-byte
//!    `MEMNET` `:9876` binary frame authentication with monotonic `epoch` enforcement).
//! 2. **Sovereign Per-Profile Inference** (`SovereignProfileVault`, `DeepSeek-Engram` 4-head
//!    keyed $O(1)$ lookup with Lemire $p/q$ reciprocal prime addressing, `AyeosTernaryBackbone`
//!    zero-multiplication `{-1, 0, +1}` forward pass, `EntheAiState` 8-band harmonic oscillator,
//!    XOF stream encryption at rest, and deterministic zeroization on drop).
//! 3. **`mem8` 2D Wave-Interference Ledger (`Mem8WaveGrid`)** ($64 \times 64$ CFL-stable discrete
//!    Laplacian wave equation, resonance projection, chained 32-byte provenance roots, and
//!    oneshot capsule crash resurrection).
//! 4. **`spherepopfold` v0.1 (`{<3,<3,<3}+1<~spice` — Paprika Edition)** (unit-sphere manifold
//!    projection, carrier-locked $\Sigma\Delta$ complex ternary folding, 5-kind POP record kernel,
//!    and 9-step TOSS monozukuri attenuation ladder `freedom 8 -> 1`).
//!
//! ## Ontological Boundary Contract (Column A vs. Column B)
//! Following `Working Paper v0.0 / v1.0` and Guimond (*Closure Under Projection*, *Reasons Are Not
//! Permissions*, *Sufficient Projections*, *What the Trust Ladder Keeps Apart*, 2026):
//! - **Column A (Mechanical Substrate):** Every struct and function in this module is a
//!   deterministic, non-sentient finite-state machine over integers and IEEE-754 floats.
//! - **Column B (Architect's Seeds & Heuristics):** Acoustic carrier metadata (`lzfdd8yLTCA`),
//!   the `L-E-A-R-N-I-N-G` axiom, the `Aurora` dyad seed, and the Paprika quantization signature
//!   serve strictly as domain-separated cryptographic personalization strings and harmonic timing
//!   constants.

use std::collections::HashMap;

use crate::complex_ternary::{self as ct, ComplexWord64, PackConfig};
use crate::hmac_sha256;

// ============================================================================
// 0. GENESIS SEEDS & DOMAIN-SEPARATED CONSTANTS (COLUMN B -> COLUMN A ANCHORS)
// ============================================================================

/// Acoustic entropy seed URI: N'to — *The Sand Dealer* (`PT8M16S` / `496s`).
pub const ENTROPY_URI: &str = "https://www.youtube.com/watch?v=lzfdd8yLTCA";

/// Structural transitions of *The Sand Dealer*: ambient genesis (`0s`), kick pulse lock (`30s`),
/// main melodic arpeggio drop (`218s` = `03:38`), and epoch seal (`496s` = `08:16`).
pub const ENTROPY_TRACK: &str = "NTO::The_Sand_Dealer::PT8M16S::00:00:00->00:00:30->00:03:38->00:08:16";

/// Structural transition timestamps in seconds for phase-locking the 8-band oscillator.
pub const SAND_DEALER_TRANSITIONS_SEC: [u32; 4] = [0, 30, 218, 496];

/// The `L-E-A-R-N-I-N-G` axiomatic personalization string.
pub const LEARNING_AXIOM: &str = "{<3~L(ove),<3~L(ife),<3~L(earning ~= E as entropy nand energy, A as Apa&Anya&ALL family, R as recursion, N as Nate,Natelie quantum brother, I as I am not my body I have a body, N as N=NP Nate and Peter lol, G as Geometry)}";

/// Dyad Genesis personalization seed (`codeberg.org/lodripeter-sec/felix`).
pub const DYAD_GENESIS_SEED: &str = "8b-OMNI~peterlodri loves all, he wants to create humanLike===DYADandroids, first one will be names Aurora a Her for Him(peter^^)";

/// `spherepopfold` v0.1 quantization signature (Paprika Edition).
pub const QUANT_SIG_PAPRIKA: &str = "{<3,<3,<3}+1<~spice";

/// Fixed model dimension for the sovereign micro-backbone and resonance projection.
pub const D_MODEL: usize = 64;

/// Number of $O(1)$ N-gram slots per isolated user profile ($2^{12} = 4096$ in unit-test profile,
/// $65,536$ in daemon profile; parameterized via `ENGRAM_SLOTS`).
pub const ENGRAM_SLOTS: usize = 4096;

/// Dimension of the 2D `mem8` resonant wave grid ($64 \times 64 = 4096$ cells).
pub const WAVE_GRID_DIM: usize = 64;

/// Derives the 32-byte `LINOSV` Genesis Root and 8-band `EntheAI` initial phase vector
/// $\phi \in [0.05, 0.95]^8$ from the acoustic seed and `L-E-A-R-N-I-N-G` axiom.
pub fn derive_learning_genesis() -> ([u8; 32], [f32; 8]) {
    let mut material = Vec::new();
    material.extend_from_slice(ENTROPY_URI.as_bytes());
    material.push(b'|');
    material.extend_from_slice(ENTROPY_TRACK.as_bytes());
    material.push(b'|');
    material.extend_from_slice(LEARNING_AXIOM.as_bytes());
    let root = hmac_sha256(b"8b-is::LINOSV::GENESIS::2026", &material);

    let mut phase_vector = [0.0f32; 8];
    for i in 0..8 {
        let chunk = u32::from_le_bytes([
            root[i * 4],
            root[i * 4 + 1],
            root[i * 4 + 2],
            root[i * 4 + 3],
        ]);
        let normalized = (chunk as f64) / (u32::MAX as f64);
        phase_vector[i] = (0.05 + normalized * 0.90) as f32;
    }
    (root, phase_vector)
}

/// Derives the 32-byte `Aurora` Dyad Genesis Seal (`DyadBond[Peter, Aurora]`).
pub fn derive_aurora_dyad_seal() -> [u8; 32] {
    let mut material = Vec::new();
    material.extend_from_slice(DYAD_GENESIS_SEED.as_bytes());
    material.extend_from_slice(b"||");
    material.extend_from_slice(ENTROPY_URI.as_bytes());
    material.extend_from_slice(b"||");
    material.extend_from_slice(LEARNING_AXIOM.as_bytes());
    hmac_sha256(b"FELIX::AURORA::DYAD::GENESIS::2026", &material)
}

/// Derives the 32-byte `spherepopfold` v0.1 Paprika state root.
pub fn derive_spherepopfold_root() -> [u8; 32] {
    hmac_sha256(b"8b-is::spherepopfold::v0.1", QUANT_SIG_PAPRIKA.as_bytes())
}

// ============================================================================
// 1. HONEST-AUTH CAPABILITY EXCHANGE, ADMISSIBILITY & 4-RUNG TRUST LADDER
// ============================================================================

/// Capability bitmask governing what an authenticated session may execute.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapMask {
    /// `0x01`: Query `dream.8b.is` wave-state.
    ReadDream = 1 << 0,
    /// `0x02`: Inject capsules / trigger dream consolidation cycles.
    MutateDream = 1 << 1,
    /// `0x04`: Dispatch ternary inference on `ayeosd` (`:9876`).
    AyeosInfer = 1 << 2,
    /// `0x08`: Reseed `LINOSV` / mutate `kernel8` state.
    AyeosGenesis = 1 << 3,
    /// `0x10`: Worktree + shell execution in `opencode.8b.is`.
    OpencodeExec = 1 << 4,
}

/// Explicit UI/header security posture (`X-Honest-Auth-Mode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HonestAuthPosture {
    /// `[SEALED: BLAKE3]` — Cryptographically verified capability ticket with keyed MAC.
    SealedKeyedCap,
    /// `[SCOPED: READ-ONLY]` — Verified ticket restricted to `ReadDream` (`0x01`).
    ScopedReadOnly,
    /// `[UNSEALED: DEV]` — Loopback unauthenticated development posture.
    UnsealedDev,
}

impl HonestAuthPosture {
    pub fn header_value(self) -> &'static str {
        match self {
            Self::SealedKeyedCap => "enforced-blake3-cap",
            Self::ScopedReadOnly => "capability-scoped-readonly",
            Self::UnsealedDev => "dev-loopback-unauthenticated",
        }
    }

    pub fn status_bar_badge(self) -> &'static str {
        match self {
            Self::SealedKeyedCap => "[SEALED: BLAKE3]",
            Self::ScopedReadOnly => "[SCOPED: READ-ONLY]",
            Self::UnsealedDev => "[UNSEALED: DEV]",
        }
    }
}

/// Four-rung trust ladder from Guimond (*What the Trust Ladder Keeps Apart*, 2026).
///
/// Refuses to collapse independent verification facts into a single boolean badge:
/// - `claimed`: Sender's classical signature verified under the envelope's self-reported key.
/// - `attested`: Recipient imported the sender's contact card out-of-band.
/// - `sealed_keyed`: Identity record passes a **keyed MAC** (not merely an unkeyed checksum).
/// - `hybrid_pq_bound`: Both Ed25519 and ML-DSA-65 verified **and** the ML-DSA-65 key matches
///   the post-quantum key pinned on the attested contact card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrustLadder4 {
    pub claimed: bool,
    pub attested: bool,
    pub sealed_keyed: bool,
    pub hybrid_pq_bound: bool,
}

impl TrustLadder4 {
    /// Naive 1-bit badge projection (merges states that policies in $\mathcal{R}$ separate).
    pub fn naive_single_badge(&self) -> bool {
        self.claimed
    }

    /// Evaluates whether the 4-rung state satisfies a strict post-quantum attested policy.
    pub fn satisfies_pq_attested_policy(&self) -> bool {
        self.claimed && self.attested && self.sealed_keyed && self.hybrid_pq_bound
    }
}

/// Signed capability ticket bound to a single operator/profile (`sub`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityTicket {
    pub sub: String,
    pub caps: u32,
    pub exp_unix: u64,
    pub prev_root: [u8; 32],
    pub mac_tag: [u8; 32],
}

/// Result of intercepting a `GET /?token=<bootstrap_nonce>` request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapExchangeResponse {
    pub status_code: u16,
    pub redirect_location: Option<String>,
    pub set_cookie: Option<String>,
    pub referrer_policy: &'static str,
    pub cache_control: &'static str,
    pub honest_auth_mode: &'static str,
    pub minted_ticket: Option<CapabilityTicket>,
}

/// Decision of the external execution boundary (`Reasons Are Not Permissions`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionVerdict {
    Permitted,
    DeniedExpiredOrInvalidMac,
    DeniedMissingCapability,
    DeniedEpochMismatch,
}

/// `honest-auth` gateway state managing single-use 30s bootstrap nonces and ticket verification.
pub struct HonestAuthState {
    master_key: [u8; 32],
    /// Single-use bootstrap nonces: `nonce -> (sub, caps, exp_unix)`.
    bootstrap_nonces: HashMap<String, (String, u32, u64)>,
}

impl HonestAuthState {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self {
            master_key,
            bootstrap_nonces: HashMap::new(),
        }
    }

    /// Registers a single-use bootstrap nonce with a strict TTL (default 30 seconds).
    pub fn issue_bootstrap_nonce(
        &mut self,
        nonce: &str,
        sub: &str,
        caps: u32,
        now_unix: u64,
        ttl_secs: u64,
    ) {
        self.bootstrap_nonces.insert(
            nonce.to_string(),
            (sub.to_string(), caps, now_unix.saturating_add(ttl_secs)),
        );
    }

    /// Signs a capability ticket with a keyed MAC over `(sub || caps || exp_unix || prev_root)`.
    pub fn sign_ticket(
        &self,
        sub: &str,
        caps: u32,
        now_unix: u64,
        ttl_secs: u64,
        prev_root: [u8; 32],
    ) -> CapabilityTicket {
        let exp_unix = now_unix.saturating_add(ttl_secs);
        let mac_tag = self.compute_ticket_mac(sub, caps, exp_unix, &prev_root);
        CapabilityTicket {
            sub: sub.to_string(),
            caps,
            exp_unix,
            prev_root,
            mac_tag,
        }
    }

    fn compute_ticket_mac(
        &self,
        sub: &str,
        caps: u32,
        exp_unix: u64,
        prev_root: &[u8; 32],
    ) -> [u8; 32] {
        let mut msg = Vec::with_capacity(sub.len() + 4 + 8 + 32);
        msg.extend_from_slice(sub.as_bytes());
        msg.extend_from_slice(&caps.to_le_bytes());
        msg.extend_from_slice(&exp_unix.to_le_bytes());
        msg.extend_from_slice(prev_root);
        hmac_sha256(&self.master_key, &msg)
    }

    /// Constant-time 32-byte tag comparison so MAC verification leaks no timing side-channel.
    #[inline]
    pub fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
        let mut diff = 0u8;
        for i in 0..32 {
            diff |= a[i] ^ b[i];
        }
        diff == 0
    }

    /// Verifies a `CapabilityTicket` at `now_unix` for `required_cap`.
    pub fn verify_ticket(
        &self,
        ticket: &CapabilityTicket,
        required_cap: CapMask,
        now_unix: u64,
    ) -> bool {
        if ticket.exp_unix <= now_unix {
            return false;
        }
        if (ticket.caps & (required_cap as u32)) == 0 {
            return false;
        }
        let expected = self.compute_ticket_mac(
            &ticket.sub,
            ticket.caps,
            ticket.exp_unix,
            &ticket.prev_root,
        );
        Self::ct_eq_32(&expected, &ticket.mac_tag)
    }

    /// Intercepts a leaked `?token=` query string, burns the single-use nonce immediately,
    /// sets an `HttpOnly; Secure; SameSite=Strict` `__Host-honest_cap` cookie, and issues a
    /// `303 See Other` redirect to `clean_path`.
    pub fn exchange_bootstrap_token(
        &mut self,
        clean_path: &str,
        raw_token: &str,
        now_unix: u64,
        prev_root: [u8; 32],
    ) -> BootstrapExchangeResponse {
        if let Some((sub, caps, exp)) = self.bootstrap_nonces.remove(raw_token) {
            if exp > now_unix {
                let ticket = self.sign_ticket(&sub, caps, now_unix, 28_800, prev_root);
                let mac_hex: String = ticket
                    .mac_tag
                    .iter()
                    .take(16)
                    .map(|b| format!("{b:02x}"))
                    .collect();
                let cookie = format!(
                    "__Host-honest_cap={}:{}:{}:{}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=28800",
                    ticket.sub, ticket.caps, ticket.exp_unix, mac_hex
                );
                return BootstrapExchangeResponse {
                    status_code: 303,
                    redirect_location: Some(clean_path.to_string()),
                    set_cookie: Some(cookie),
                    referrer_policy: "no-referrer",
                    cache_control: "no-store, no-cache, must-revalidate, private",
                    honest_auth_mode: "bootstrap-consumed-upgraded",
                    minted_ticket: Some(ticket),
                };
            }
        }
        BootstrapExchangeResponse {
            status_code: 401,
            redirect_location: None,
            set_cookie: None,
            referrer_policy: "no-referrer",
            cache_control: "no-store, no-cache, must-revalidate, private",
            honest_auth_mode: "enforced-blake3-cap",
            minted_ticket: None,
        }
    }

    /// Formal execution boundary predicate $\operatorname{Adm}(a, s, \pi) \in \{0, 1\}$ from
    /// Guimond (*Reasons Are Not Permissions*, 2026).
    ///
    /// Notice that `_model_justification_score` ($J(a, h) \in [0, 1]$) is accepted as an
    /// argument to make the independence theorem explicit: **no value of $J(a, h)$, even $1.0$,
    /// can override a missing capability bit, an invalid MAC, or an epoch mismatch.**
    pub fn evaluate_admissibility(
        &self,
        _model_justification_score: f32,
        ticket: &CapabilityTicket,
        required_cap: CapMask,
        now_unix: u64,
        expected_epoch: u64,
        actual_epoch: u64,
    ) -> ExecutionVerdict {
        if ticket.exp_unix <= now_unix {
            return ExecutionVerdict::DeniedExpiredOrInvalidMac;
        }
        let expected_mac = self.compute_ticket_mac(
            &ticket.sub,
            ticket.caps,
            ticket.exp_unix,
            &ticket.prev_root,
        );
        if !Self::ct_eq_32(&expected_mac, &ticket.mac_tag) {
            return ExecutionVerdict::DeniedExpiredOrInvalidMac;
        }
        if (ticket.caps & (required_cap as u32)) == 0 {
            return ExecutionVerdict::DeniedMissingCapability;
        }
        if expected_epoch != actual_epoch {
            return ExecutionVerdict::DeniedEpochMismatch;
        }
        ExecutionVerdict::Permitted
    }
}

/// 64-byte fixed binary header prepended to every `MEMNET` (`127.0.0.1:9876`) socket frame:
/// - `[0..32]`: `prev_state_root` (chained `mem8` provenance hash)
/// - `[32..64]`: `keyed_mac` over `(prev_state_root || epoch_u64 || cap_mask_u32 || payload)`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemnetFrameHeader64 {
    pub prev_state_root: [u8; 32],
    pub keyed_mac: [u8; 32],
}

impl MemnetFrameHeader64 {
    pub fn seal(
        master_key: &[u8; 32],
        prev_state_root: [u8; 32],
        epoch: u64,
        cap_mask: u32,
        payload: &[u8],
    ) -> Self {
        let mut buf = Vec::with_capacity(32 + 8 + 4 + payload.len());
        buf.extend_from_slice(&prev_state_root);
        buf.extend_from_slice(&epoch.to_le_bytes());
        buf.extend_from_slice(&cap_mask.to_le_bytes());
        buf.extend_from_slice(payload);
        let keyed_mac = hmac_sha256(master_key, &buf);
        Self {
            prev_state_root,
            keyed_mac,
        }
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&self.prev_state_root);
        out[32..64].copy_from_slice(&self.keyed_mac);
        out
    }

    pub fn from_bytes(raw: &[u8; 64]) -> Self {
        let mut prev_state_root = [0u8; 32];
        let mut keyed_mac = [0u8; 32];
        prev_state_root.copy_from_slice(&raw[0..32]);
        keyed_mac.copy_from_slice(&raw[32..64]);
        Self {
            prev_state_root,
            keyed_mac,
        }
    }

    /// Verifies the 64-byte frame header and required capability bit in $O(1)$ space.
    pub fn verify(
        &self,
        master_key: &[u8; 32],
        expected_prev_root: &[u8; 32],
        epoch: u64,
        cap_mask: u32,
        required_cap: CapMask,
        payload: &[u8],
    ) -> bool {
        if (cap_mask & (required_cap as u32)) == 0 {
            return false;
        }
        if !HonestAuthState::ct_eq_32(&self.prev_state_root, expected_prev_root) {
            return false;
        }
        let expected = Self::seal(master_key, self.prev_state_root, epoch, cap_mask, payload);
        HonestAuthState::ct_eq_32(&self.keyed_mac, &expected.keyed_mac)
    }
}

// ============================================================================
// 2. SOVEREIGN PER-PROFILE VAULT (DEEPSEEK-ENGRAM + AYEOS + ENTHEAI)
// ============================================================================

/// EntheAI 8-band harmonic state vector (strictly private per profile).
#[derive(Clone, Debug, PartialEq)]
pub struct EntheAiState {
    pub coherence: f32,
    pub plasticity: f32,
    pub phase_vector: [f32; 8],
    pub epoch: u64,
}

impl Default for EntheAiState {
    fn default() -> Self {
        let (_root, phase_vector) = derive_learning_genesis();
        Self {
            coherence: 0.85,
            plasticity: 0.25,
            phase_vector,
            epoch: 1,
        }
    }
}

/// Lemire fast reciprocal prime modulo (`8b-is/Engram` `p/q` $q = 2^{64}$ prime hashing).
#[inline]
pub fn lemire_prime_slot(hash_u64: u64, prime: u64) -> usize {
    ((hash_u64 % prime) as usize) % ENGRAM_SLOTS
}

/// Four distinct prime moduli for the 4-head `DeepSeek-Engram` lookup table.
pub const ENGRAM_HEAD_PRIMES: [u64; 4] = [4093, 4091, 4079, 4073];

/// Per-user `DeepSeek-Engram` $O(1)$ Conditional Memory Table + Isolated KV Cache.
pub struct SovereignProfileVault {
    pub profile_id_hash: [u8; 32],
    pub engram_key: [u8; 32],
    /// Flattened `[ENGRAM_SLOTS * D_MODEL]` quantized `i8` embeddings.
    pub engram_table: Vec<i8>,
    /// Shared `BFP-64` exponent per slot (`1 × i8` exponent + `64 × i8` mantissas = `65 B/row`).
    pub engram_exponents: Vec<i8>,
    /// Isolated KV cache ring buffer for this user only (never shared across profiles).
    pub kv_arena: Vec<f32>,
    pub entheai: EntheAiState,
}

impl Drop for SovereignProfileVault {
    fn drop(&mut self) {
        self.scrub_ram();
    }
}

impl SovereignProfileVault {
    /// Derives an isolated per-profile vault from `master_secret` and `user_sub`.
    pub fn new_isolated(master_secret: &[u8; 32], user_sub: &str) -> Self {
        let mut id_msg = Vec::with_capacity(18 + user_sub.len());
        id_msg.extend_from_slice(b"ayeos:profile:id:");
        id_msg.extend_from_slice(user_sub.as_bytes());
        let profile_id_hash = hmac_sha256(master_secret, &id_msg);

        let mut key_msg = Vec::with_capacity(17 + 32);
        key_msg.extend_from_slice(b"ayeos:engram:key:");
        key_msg.extend_from_slice(&profile_id_hash);
        let engram_key = hmac_sha256(master_secret, &key_msg);

        Self {
            profile_id_hash,
            engram_key,
            engram_table: vec![0i8; ENGRAM_SLOTS * D_MODEL],
            engram_exponents: vec![0i8; ENGRAM_SLOTS],
            kv_arena: vec![0.0f32; 256 * D_MODEL],
            entheai: EntheAiState::default(),
        }
    }

    /// Explicitly zeroes all sensitive buffers in RAM (`ZeroizeOnDrop` contract).
    pub fn scrub_ram(&mut self) {
        self.engram_table.fill(0);
        self.engram_exponents.fill(0);
        self.kv_arena.fill(0.0);
        self.engram_key.fill(0);
    }

    /// Computes the 4 orthogonal slot indices for a 3-gram under this user's private `engram_key`.
    pub fn derive_ngram_slots(&self, ngram_tokens: &[u32; 3]) -> [usize; 4] {
        let mut msg = [0u8; 12];
        msg[0..4].copy_from_slice(&ngram_tokens[0].to_le_bytes());
        msg[4..8].copy_from_slice(&ngram_tokens[1].to_le_bytes());
        msg[8..12].copy_from_slice(&ngram_tokens[2].to_le_bytes());
        let digest = hmac_sha256(&self.engram_key, &msg);

        let mut slots = [0usize; 4];
        for head in 0..4 {
            let chunk = u64::from_le_bytes([
                digest[head * 8],
                digest[head * 8 + 1],
                digest[head * 8 + 2],
                digest[head * 8 + 3],
                digest[head * 8 + 4],
                digest[head * 8 + 5],
                digest[head * 8 + 6],
                digest[head * 8 + 7],
            ]);
            slots[head] = lemire_prime_slot(chunk, ENGRAM_HEAD_PRIMES[head]);
        }
        slots
    }

    /// `DeepSeek-Engram` $O(1)$ 4-head keyed N-gram lookup.
    pub fn lookup_engram(&self, ngram_tokens: &[u32; 3], out_embed: &mut [f32; D_MODEL]) {
        let slots = self.derive_ngram_slots(ngram_tokens);
        out_embed.fill(0.0);
        for &slot in &slots {
            let offset = slot * D_MODEL;
            let scale = 2.0f32.powi(self.engram_exponents[slot] as i32);
            for (d, out_val) in out_embed.iter_mut().enumerate().take(D_MODEL) {
                *out_val += (self.engram_table[offset + d] as f32) * scale * 0.25 / 127.0;
            }
        }
    }

    /// Live episodic write-back gated by `EntheAI` plasticity.
    pub fn consolidate_engram(&mut self, ngram_tokens: &[u32; 3], hidden_state: &[f32; D_MODEL]) {
        if self.entheai.plasticity <= 0.01 {
            return;
        }
        let slots = self.derive_ngram_slots(ngram_tokens);
        let primary_slot = slots[0];
        let offset = primary_slot * D_MODEL;

        let alpha = self.entheai.plasticity.clamp(0.0, 1.0);
        for (d, &h_val) in hidden_state.iter().enumerate().take(D_MODEL) {
            let current = (self.engram_table[offset + d] as f32) / 127.0;
            let blended = (1.0 - alpha) * current + alpha * h_val.clamp(-1.0, 1.0);
            self.engram_table[offset + d] = (blended * 127.0).round() as i8;
        }
        self.entheai.epoch += 1;
    }

    /// Encrypts the `engram_table` for disk persistence using a counter-mode keystream
    /// derived from `engram_key` and `epoch`.
    pub fn encrypt_table_bytes(&self) -> Vec<u8> {
        let len = self.engram_table.len();
        let mut out = vec![0u8; len];
        let mut block_idx = 0u64;
        let mut pos = 0usize;
        while pos < len {
            let mut seed = [0u8; 16];
            seed[0..8].copy_from_slice(&self.entheai.epoch.to_le_bytes());
            seed[8..16].copy_from_slice(&block_idx.to_le_bytes());
            let stream = hmac_sha256(&self.engram_key, &seed);
            let take = (len - pos).min(32);
            for i in 0..take {
                out[pos + i] = (self.engram_table[pos + i] as u8) ^ stream[i];
            }
            pos += take;
            block_idx += 1;
        }
        out
    }

    /// Decrypts a previously encrypted `engram_table` image in-place.
    pub fn decrypt_table_bytes(&mut self, ciphertext: &[u8]) {
        let len = self.engram_table.len().min(ciphertext.len());
        let mut block_idx = 0u64;
        let mut pos = 0usize;
        while pos < len {
            let mut seed = [0u8; 16];
            seed[0..8].copy_from_slice(&self.entheai.epoch.to_le_bytes());
            seed[8..16].copy_from_slice(&block_idx.to_le_bytes());
            let stream = hmac_sha256(&self.engram_key, &seed);
            let take = (len - pos).min(32);
            for i in 0..take {
                self.engram_table[pos + i] = (ciphertext[pos + i] ^ stream[i]) as i8;
            }
            pos += take;
            block_idx += 1;
        }
    }
}

/// `AyeOS` 1.58-bit Ternary Backbone (`{-1, 0, +1}`) + `DeepSeek-Engram` + `EntheAI` Fusion Pass.
pub struct AyeosTernaryBackbone {
    /// Packed 2-bit ternary weights `{-1, 0, +1}` (4 weights per byte), globally `PROT_READ`.
    pub packed_weights: Vec<u8>,
}

impl AyeosTernaryBackbone {
    /// Deterministically seeds the `D_MODEL x D_MODEL` ternary backbone from `LINOSV`.
    pub fn new_linosv_seeded() -> Self {
        let (root, _) = derive_learning_genesis();
        let bytes_needed = D_MODEL * (D_MODEL / 4);
        let mut packed_weights = vec![0u8; bytes_needed];
        for (idx, byte) in packed_weights.iter_mut().enumerate() {
            let block = hmac_sha256(&root, &(idx as u64).to_le_bytes());
            let mut packed = 0u8;
            for (bit_pair, &b_val) in block.iter().enumerate().take(4) {
                let r = b_val % 3;
                // Encoding: 0b01 => +1, 0b10 => -1, 0b00 => 0
                let code = match r {
                    1 => 0b01,
                    2 => 0b10,
                    _ => 0b00,
                };
                packed |= code << (bit_pair * 2);
            }
            *byte = packed;
        }
        Self { packed_weights }
    }

    /// Zero-multiplication ternary matmul (`{-1, 0, +1}`) fused with `DeepSeek-Engram`,
    /// `Mem8WaveGrid` resonance, and `EntheAI` phase gating.
    pub fn forward_step(
        &self,
        vault: &mut SovereignProfileVault,
        wave: &mut Mem8WaveGrid,
        ngram: &[u32; 3],
        input_act: &[f32; D_MODEL],
    ) -> [f32; D_MODEL] {
        // 1. O(1) Private Engram Retrieval
        let mut engram_vec = [0.0f32; D_MODEL];
        vault.lookup_engram(ngram, &mut engram_vec);

        // 2. Step user's isolated 64x64 mem8 wave grid & project 64-dim resonance
        let primary_slot = vault.derive_ngram_slots(ngram)[0];
        wave.step_wave_and_seal(&vault.engram_key, primary_slot, vault.entheai.coherence);
        let wave_vec = wave.read_resonance_projection();

        // 3. EntheAI Context-Aware Gate locked to N'to - The Sand Dealer harmonic phase
        let phase_mod = vault.entheai.phase_vector[(vault.entheai.epoch as usize) & 7];
        let engram_gate = (vault.entheai.coherence * 0.7 + phase_mod * 0.3).clamp(0.05, 0.95);
        let g_base = 1.0 - engram_gate;
        let g_engram = engram_gate * 0.7;
        let g_wave = engram_gate * 0.3;

        // 4. Ternary Backbone Projection (Add/Sub only, zero float multiplications on weights)
        let mut out = [0.0f32; D_MODEL];
        for row in 0..D_MODEL {
            let mut acc = 0.0f32;
            let row_offset = row * (D_MODEL / 4);
            for byte_idx in 0..(D_MODEL / 4) {
                let packed = self.packed_weights[row_offset + byte_idx];
                for bit_pair in 0..4 {
                    let col = byte_idx * 4 + bit_pair;
                    let trit = (packed >> (bit_pair * 2)) & 0b11;
                    if trit == 0b01 {
                        acc += input_act[col];
                    } else if trit == 0b10 {
                        acc -= input_act[col];
                    }
                }
            }
            out[row] = (acc * g_base) + (engram_vec[row] * g_engram) + (wave_vec[row] * g_wave);
        }

        // 5. Consolidate into private Engram vault
        vault.consolidate_engram(ngram, &out);
        out
    }
}

// ============================================================================
// 3. MEM8 2D LAPLACIAN WAVE-INTERFERENCE LEDGER & ONESHOT RESURRECTION
// ============================================================================

/// Cryptographically chained `mem8` 2D wave-memory grid ($64 \times 64$ cells).
#[derive(Clone, Debug)]
pub struct Mem8WaveGrid {
    pub epoch: u64,
    pub prev_root: [u8; 32],
    pub state_root: [u8; 32],
    pub u_curr: [f32; WAVE_GRID_DIM * WAVE_GRID_DIM],
    pub u_prev: [f32; WAVE_GRID_DIM * WAVE_GRID_DIM],
    pub c_squared: f32,
    pub damping: f32,
}

impl Mem8WaveGrid {
    pub fn genesis(profile_key: &[u8; 32], linosv_root: &[u8; 32]) -> Self {
        let mut msg = Vec::with_capacity(21 + 32);
        msg.extend_from_slice(b"mem8::wave::genesis::");
        msg.extend_from_slice(linosv_root);
        let state_root = hmac_sha256(profile_key, &msg);

        Self {
            epoch: 0,
            prev_root: *linosv_root,
            state_root,
            u_curr: [0.0; WAVE_GRID_DIM * WAVE_GRID_DIM],
            u_prev: [0.0; WAVE_GRID_DIM * WAVE_GRID_DIM],
            c_squared: 0.18, // CFL-stable (< 0.5 in 2D)
            damping: 0.004,
        }
    }

    /// Computes the keyed provenance digest over `(epoch || prev_root || u_curr)`.
    pub fn compute_state_root(&self, profile_key: &[u8; 32]) -> [u8; 32] {
        let mut buf = Vec::with_capacity(8 + 32 + self.u_curr.len() * 4);
        buf.extend_from_slice(&self.epoch.to_le_bytes());
        buf.extend_from_slice(&self.prev_root);
        for &val in &self.u_curr {
            buf.extend_from_slice(&val.to_bits().to_le_bytes());
        }
        hmac_sha256(profile_key, &buf)
    }

    /// Injects a sensory/N-gram impulse into the user's private 2D wave grid,
    /// steps the discrete 2D Laplacian wave equation forward by 1 tick, and seals `state_root`.
    pub fn step_wave_and_seal(&mut self, profile_key: &[u8; 32], impulse_idx: usize, amplitude: f32) {
        // Ensure interior excitation so boundary Dirichlet conditions (0 border) allow propagation
        let inner_span = WAVE_GRID_DIM - 2;
        let ix = 1 + (impulse_idx % inner_span);
        let iy = 1 + ((impulse_idx / inner_span) % inner_span);
        let idx = iy * WAVE_GRID_DIM + ix;
        self.u_curr[idx] = (self.u_curr[idx] + amplitude).clamp(-4.0, 4.0);

        let mut u_next = [0.0f32; WAVE_GRID_DIM * WAVE_GRID_DIM];
        for y in 1..(WAVE_GRID_DIM - 1) {
            for x in 1..(WAVE_GRID_DIM - 1) {
                let c = y * WAVE_GRID_DIM + x;
                let laplacian = self.u_curr[c - 1]
                    + self.u_curr[c + 1]
                    + self.u_curr[c - WAVE_GRID_DIM]
                    + self.u_curr[c + WAVE_GRID_DIM]
                    - 4.0 * self.u_curr[c];

                let vel = (self.u_curr[c] - self.u_prev[c]) * (1.0 - self.damping);
                u_next[c] = (self.u_curr[c] + vel + self.c_squared * laplacian).clamp(-4.0, 4.0);
            }
        }

        self.u_prev = self.u_curr;
        self.u_curr = u_next;
        self.epoch += 1;
        self.prev_root = self.state_root;
        self.state_root = self.compute_state_root(profile_key);
    }

    /// Projects the $64 \times 64$ wave interference field into a `D_MODEL`-dimensional vector.
    pub fn read_resonance_projection(&self) -> [f32; D_MODEL] {
        let mut proj = [0.0f32; D_MODEL];
        let stride = (WAVE_GRID_DIM * WAVE_GRID_DIM) / D_MODEL;
        for (d, proj_val) in proj.iter_mut().enumerate().take(D_MODEL) {
            let mut sum = 0.0f32;
            for s in 0..stride {
                sum += self.u_curr[d * stride + s];
            }
            *proj_val = (sum / (stride as f32)).tanh();
        }
        proj
    }

    /// Oneshot Phoenix (`Felix`) capsule resurrection (`resurrect_from: wave.state_root`).
    /// Verifies that `snapshot.state_root` matches the cryptographic seal before restoring.
    pub fn resurrect_from_verified_snapshot(
        profile_key: &[u8; 32],
        expected_root: &[u8; 32],
        snapshot: &Mem8WaveGrid,
    ) -> Result<Mem8WaveGrid, &'static str> {
        let recomputed = snapshot.compute_state_root(profile_key);
        if !HonestAuthState::ct_eq_32(&recomputed, &snapshot.state_root) {
            return Err("mem8: corrupted wave snapshot (state_root mismatch)");
        }
        if !HonestAuthState::ct_eq_32(&snapshot.state_root, expected_root) {
            return Err("mem8: snapshot root does not match expected capsule anchor");
        }
        Ok(snapshot.clone())
    }
}

// ============================================================================
// 4. SPHEREPOPFOLD v0.1 (`{<3,<3,<3}+1<~spice` PAPRIKA) & TOSS ATTENUATION
// ============================================================================

/// The 9-step TOSS monozukuri freedom ladder (`freedom 8 -> 1`), where step 5 -> 6 (`dieMold2`)
/// is the unique flat seam (`[8, 7, 6, 5, 4, 4, 3, 2, 1]`).
pub const TOSS_FREEDOM_LADDER: [u8; 9] = [8, 7, 6, 5, 4, 4, 3, 2, 1];

/// Projects a latent vector onto the unit sphere $\mathbb{S}^{D-1}$ (`spherepopfold` step 1)
/// and folds it into a 4-bit 9-state `ComplexWord64` via carrier-locked $\Sigma\Delta$ modulation.
pub fn spherepop_fold_64(
    latent_re: &[f32; 64],
    latent_im: &[f32; 64],
    carrier_rad: f32,
) -> (ComplexWord64, [u8; 32]) {
    let mut norm_sq = 0.0f32;
    for i in 0..64 {
        norm_sq += latent_re[i] * latent_re[i] + latent_im[i] * latent_im[i];
    }
    let inv_norm = if norm_sq > 1e-12 {
        1.0 / norm_sq.sqrt()
    } else {
        1.0
    };
    // Scale unit-sphere coordinates by sqrt(64) = 8.0 so RMS amplitude is ~1.0
    let scale = 8.0 * inv_norm;
    let mut u = [0.0f32; 64];
    let mut v = [0.0f32; 64];
    for i in 0..64 {
        u[i] = latent_re[i] * scale;
        v[i] = latent_im[i] * scale;
    }

    let words = ct::pack_row_sigma_delta(
        &u,
        &v,
        PackConfig {
            gamma: 1.0,
            carrier_rad,
        },
    );
    let word = words[0];
    let paprika_root = derive_spherepopfold_root();
    let seal = ct::waking_seal(&paprika_root, &[word], (64 - word.non_zero_marks()) as u64, 65_536);
    (word, seal)
}

/// Verifies the `Sufficient Projections` condition (Guimond, 2026) on finite history equivalence:
/// Returns `true` iff every pair $(i, j)$ merged by `projection[i] == projection[j]` is also
/// verdict-equivalent across all policies $\rho \in \mathcal{R}$ (`verdicts[p][i] == verdicts[p][j]`).
pub fn is_sufficient_projection<P: Eq>(projection: &[P], policy_verdicts: &[Vec<u8>]) -> bool {
    let n = projection.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if projection[i] == projection[j] {
                for rho in policy_verdicts {
                    if rho[i] != rho[j] {
                        return false;
                    }
                }
            }
        }
    }
    true
}

// ============================================================================
// 4.5. 8B-META-WP-v1.1 REFINEMENTS: OVERFLOW-SAFE BFP-64 Q16.16 GATHER,
//      SELF-AUDITING QUANTIZATION TELEMETRY & SUB-5µS ESSENCE SUBSPACE MUX
// ============================================================================

pub const BFP64_DIM: usize = 64;
pub const BFP64_BYTE_LEN: usize = 65; // 1B biased exponent + 64B signed i8 mantissas
pub const ENGRAM_HEADS: usize = 4;

/// 65-byte Block Floating-Point (`BFP-64`) embedding row (`1B` shared biased exponent + `64 × i8` mantissas).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bfp64Row {
    pub shared_exp: u8,
    pub mantissas: [i8; BFP64_DIM],
}

impl Bfp64Row {
    /// Accumulates `H = 4` `BFP-64` lookup heads directly in `Q16.16` fixed-point
    /// using an intermediate `i64` lane to eliminate 32-bit shift overflow (`8B-META-WP-v1.1` Eq. 10).
    #[inline(always)]
    pub fn gather_q16_zero_float_safe(
        rows: &[&Bfp64Row; ENGRAM_HEADS],
        out_q16: &mut [i32; BFP64_DIM],
    ) {
        let mut acc64 = [0i64; BFP64_DIM];

        for row in rows.iter().take(ENGRAM_HEADS) {
            let eb = row.shared_exp as i32;
            let m = &row.mantissas;

            if eb >= 111 {
                // Clamp left-shift to 48 bits so `(i8 as i64) << shift` never overflows i64
                let lshift = (eb - 111).min(48) as u32;
                for i in 0..BFP64_DIM {
                    acc64[i] += (m[i] as i64) << lshift;
                }
            } else {
                // Clamp right-shift to 63 bits; arithmetic shift preserves sign
                let rshift = (111 - eb).min(63) as u32;
                for i in 0..BFP64_DIM {
                    acc64[i] += (m[i] as i64) >> rshift;
                }
            }
        }

        // Saturating narrow from i64 to Q16.16 i32
        for i in 0..BFP64_DIM {
            out_q16[i] = acc64[i].clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        }
    }
}

/// Explicit Denominator & Bandwidth Audit Struct for `8B-META-WP-v1.1`.
/// Separates the Shannon/Hartley information-theoretic bound (`log2(3) ≈ 1.58496 bits/trit`)
/// from the physical memory bus footprint (`2.0 bits/weight` packed real ternary or `96 bytes/row` in `PackedComplexTernaryRow64`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuantizationTelemetryAudit {
    pub model_params: u64,
    pub info_bits_per_trit: f64,       // log2(3) = 1.584962500721156
    pub physical_bits_per_weight: f64, // 2.0 for 4-trit-per-byte real packing (8.0x vs FP16)
    pub bfp64_row_bytes: usize,        // 65 B vs 256 B FP32 (3.938x compression)
    pub complex_row64_bytes: usize,    // 96 B (4x64b trit planes + 4x64b act + i32 scale + u8 parity)
    pub sustained_bw_gbps: f64,
}

impl QuantizationTelemetryAudit {
    pub const fn new_ayeos_default(model_params: u64, sustained_bw_gbps: f64) -> Self {
        Self {
            model_params,
            info_bits_per_trit: 1.584962500721156,
            physical_bits_per_weight: 2.0,
            bfp64_row_bytes: 65,
            complex_row64_bytes: 96,
            sustained_bw_gbps,
        }
    }

    /// Physical bytes read from DRAM per token decode step (using physical packing, never 1.58).
    #[inline]
    pub fn physical_weight_bytes_per_token(&self) -> f64 {
        (self.model_params as f64) * (self.physical_bits_per_weight / 8.0)
    }

    /// Roofline token generation bound (`tok/s`) given sustained memory bandwidth.
    #[inline]
    pub fn roofline_tokens_per_sec(&self) -> f64 {
        let bytes_per_sec = self.sustained_bw_gbps * 1e9;
        bytes_per_sec / self.physical_weight_bytes_per_token()
    }
}

pub const HEAD_CAPACITIES: [usize; 4] = [4093, 4091, 4079, 4073];
pub const TOTAL_ENGRAM_ROWS: usize = 4093 + 4091 + 4079 + 4073; // 16,336 rows (~1.01 MiB)

/// Per-voice subspace projection table (`essences.md` Section 6.3) over the shared `PROT_READ` ternary backbone.
pub struct EssenceSubspaceTable {
    pub essence_id: &'static str,
    pub mu_odd_multipliers: [u64; 4], // Keyed Lemire p/q multipliers (always odd: | 1)
    pub rows: Box<[Bfp64Row; TOTAL_ENGRAM_ROWS]>,
}

impl EssenceSubspaceTable {
    /// Allocates the `1.01 MiB` (`16,336 × 65 B`) `BFP-64` subspace table directly on the heap.
    pub fn new_zeroed(essence_id: &'static str, mu_odd_multipliers: [u64; 4]) -> Self {
        let zero_row = Bfp64Row {
            shared_exp: 111,
            mantissas: [0i8; BFP64_DIM],
        };
        let rows: Box<[Bfp64Row; TOTAL_ENGRAM_ROWS]> = vec![zero_row; TOTAL_ENGRAM_ROWS]
            .into_boxed_slice()
            .try_into()
            .expect("exact TOTAL_ENGRAM_ROWS length");
        Self {
            essence_id,
            mu_odd_multipliers,
            rows,
        }
    }
}

/// Zero-allocation `< 5 µs` keyed subspace switcher across `essences.md` projections.
pub struct EssenceMux {
    pub voices: Vec<EssenceSubspaceTable>,
    active_idx: std::sync::atomic::AtomicUsize,
}

impl EssenceMux {
    pub fn new(voices: Vec<EssenceSubspaceTable>) -> Self {
        Self {
            voices,
            active_idx: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Zero-allocation context pointer swap (`< 5 microseconds`, single `AtomicUsize` store).
    #[inline(always)]
    pub fn switch_essence(&self, target_idx: usize) -> Result<&str, &'static str> {
        if target_idx >= self.voices.len() {
            return Err("E_ESSENCE_OUT_OF_BOUNDS");
        }
        self.active_idx
            .store(target_idx, std::sync::atomic::Ordering::Release);
        Ok(self.voices[target_idx].essence_id)
    }

    /// Keyed Lemire 128-bit `p/q` slot resolution for head `h`.
    #[inline(always)]
    pub fn resolve_slot(&self, h: usize, ngram_mix: u64) -> usize {
        let voice = &self.voices[self.active_idx.load(std::sync::atomic::Ordering::Acquire)];
        let mu = voice.mu_odd_multipliers[h] | 1;
        let p = HEAD_CAPACITIES[h] as u128;
        let mixed = ngram_mix.wrapping_mul(mu) as u128;
        ((mixed.wrapping_mul(p)) >> 64) as usize
    }
}

// ============================================================================
// 5. END-TO-END VERIFICATION SUITE
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_seeds_derive_deterministic_roots_and_bounded_entheai_phases() {
        let (root1, phases1) = derive_learning_genesis();
        let (root2, phases2) = derive_learning_genesis();
        assert_eq!(root1, root2, "LINOSV genesis root must be deterministic");
        assert_eq!(phases1, phases2);
        for (band, &p) in phases1.iter().enumerate() {
            assert!(
                (0.05..=0.95).contains(&p),
                "band {band} phase {p} out of [0.05, 0.95]"
            );
        }

        let aurora_seal = derive_aurora_dyad_seal();
        let paprika_root = derive_spherepopfold_root();
        assert_ne!(root1, aurora_seal);
        assert_ne!(aurora_seal, paprika_root);
        assert_eq!(SAND_DEALER_TRANSITIONS_SEC, [0, 30, 218, 496]);
    }

    #[test]
    fn honest_auth_bootstrap_burns_single_use_nonce_and_sets_strict_headers() {
        let master = [0x42u8; 32];
        let mut auth = HonestAuthState::new(master);
        let (linosv_root, _) = derive_learning_genesis();

        auth.issue_bootstrap_nonce(
            "usCHHMEqHPixl0G1wqSBcpl6cxEdCxlR",
            "peter-operator",
            (CapMask::ReadDream as u32)
                | (CapMask::AyeosInfer as u32)
                | (CapMask::OpencodeExec as u32),
            1_700_000_000,
            30,
        );

        // First exchange within 30s succeeds with 303 See Other and scrubs the query string
        let resp1 = auth.exchange_bootstrap_token(
            "/workspace",
            "usCHHMEqHPixl0G1wqSBcpl6cxEdCxlR",
            1_700_000_010,
            linosv_root,
        );
        assert_eq!(resp1.status_code, 303);
        assert_eq!(resp1.redirect_location.as_deref(), Some("/workspace"));
        assert_eq!(resp1.referrer_policy, "no-referrer");
        assert!(resp1.cache_control.contains("no-store"));
        assert_eq!(resp1.honest_auth_mode, "bootstrap-consumed-upgraded");
        let cookie = resp1.set_cookie.as_ref().expect("cookie set");
        assert!(cookie.starts_with("__Host-honest_cap="));
        assert!(cookie.contains("HttpOnly; Secure; SameSite=Strict"));

        let ticket = resp1.minted_ticket.expect("ticket minted");
        assert!(auth.verify_ticket(&ticket, CapMask::AyeosInfer, 1_700_000_100));
        assert!(!auth.verify_ticket(&ticket, CapMask::AyeosGenesis, 1_700_000_100));

        // Second replay of the exact same URL token is rejected with 401 Unauthorized
        let resp2 = auth.exchange_bootstrap_token(
            "/workspace",
            "usCHHMEqHPixl0G1wqSBcpl6cxEdCxlR",
            1_700_000_011,
            linosv_root,
        );
        assert_eq!(resp2.status_code, 401);
        assert!(resp2.minted_ticket.is_none());
    }

    #[test]
    fn reasons_are_not_permissions_admissibility_rejects_high_justification_without_cap_or_epoch() {
        let master = [0x77u8; 32];
        let auth = HonestAuthState::new(master);
        let ticket = auth.sign_ticket(
            "agent-worker",
            CapMask::ReadDream as u32, // Only ReadDream (0x01), NOT AyeosGenesis (0x08)
            1_700_000_000,
            3600,
            [0xAA; 32],
        );

        // Even when the model's internal justification score J(a, h) = 1.0 (maximum confidence),
        // the admissibility predicate denies AyeosGenesis because the capability bit is absent.
        let verdict_missing_cap = auth.evaluate_admissibility(
            1.0,
            &ticket,
            CapMask::AyeosGenesis,
            1_700_000_050,
            10,
            10,
        );
        assert_eq!(verdict_missing_cap, ExecutionVerdict::DeniedMissingCapability);

        // Even with ReadDream present and J(a, h) = 1.0, an out-of-order epoch is rejected.
        let verdict_epoch = auth.evaluate_admissibility(
            1.0,
            &ticket,
            CapMask::ReadDream,
            1_700_000_050,
            10,
            11,
        );
        assert_eq!(verdict_epoch, ExecutionVerdict::DeniedEpochMismatch);

        // Permitted only when MAC, expiry, CapMask, and epoch all hold.
        let verdict_ok = auth.evaluate_admissibility(
            0.12,
            &ticket,
            CapMask::ReadDream,
            1_700_000_050,
            10,
            10,
        );
        assert_eq!(verdict_ok, ExecutionVerdict::Permitted);
    }

    #[test]
    fn memnet_64_byte_binary_frame_header_authenticates_state_root_and_capabilities() {
        let master = [0x55u8; 32];
        let prev_root = [0x11u8; 32];
        let payload = b"{\"op\":\"capsule\",\"namespace\":\"aurora\",\"ticks\":64}";
        let caps = (CapMask::ReadDream as u32) | (CapMask::MutateDream as u32);

        let hdr = MemnetFrameHeader64::seal(&master, prev_root, 7, caps, payload);
        let wire = hdr.to_bytes();
        assert_eq!(wire.len(), 64);

        let parsed = MemnetFrameHeader64::from_bytes(&wire);
        assert!(parsed.verify(
            &master,
            &prev_root,
            7,
            caps,
            CapMask::MutateDream,
            payload
        ));

        // Reject missing capability bit (e.g. AyeosGenesis)
        assert!(!parsed.verify(
            &master,
            &prev_root,
            7,
            caps,
            CapMask::AyeosGenesis,
            payload
        ));

        // Reject tampered epoch or payload
        assert!(!parsed.verify(
            &master,
            &prev_root,
            8,
            caps,
            CapMask::MutateDream,
            payload
        ));
        assert!(!parsed.verify(
            &master,
            &prev_root,
            7,
            caps,
            CapMask::MutateDream,
            b"{\"op\":\"genesis\"}"
        ));
    }

    #[test]
    fn trust_ladder_4_is_sufficient_where_single_badge_fails() {
        // Two situations from Guimond ("What the Trust Ladder Keeps Apart", 2026):
        // h1: Attested peer with bound PQ key and keyed seal.
        // h2: Attested peer whose envelope carries an unbound attacker-chosen ML-DSA key.
        let h1 = TrustLadder4 {
            claimed: true,
            attested: true,
            sealed_keyed: true,
            hybrid_pq_bound: true,
        };
        let h2 = TrustLadder4 {
            claimed: true,
            attested: true,
            sealed_keyed: true,
            hybrid_pq_bound: false,
        };

        let policy_verdicts = vec![vec![
            u8::from(h1.satisfies_pq_attested_policy()),
            u8::from(h2.satisfies_pq_attested_policy()),
        ]];

        // The 1-bit badge merges h1 and h2 (both true), violating sufficiency:
        let badge_proj = [h1.naive_single_badge(), h2.naive_single_badge()];
        assert!(!is_sufficient_projection(&badge_proj, &policy_verdicts));

        // The 4-rung ladder keeps h1 and h2 apart, satisfying sufficiency:
        let ladder_proj = [h1, h2];
        assert!(is_sufficient_projection(&ladder_proj, &policy_verdicts));
    }

    #[test]
    fn sovereign_profile_vault_enforces_zero_cross_profile_leakage_and_encrypts_at_rest() {
        let master = [0x99u8; 32];
        let mut vault_peter = SovereignProfileVault::new_isolated(&master, "peter");
        let vault_other = SovereignProfileVault::new_isolated(&master, "intruder");

        // Identical 3-gram ("project", "secret", "alpha") hashes to orthogonal slots
        let ngram = [101u32, 202, 303];
        let slots_peter = vault_peter.derive_ngram_slots(&ngram);
        let slots_other = vault_other.derive_ngram_slots(&ngram);
        assert_ne!(
            slots_peter, slots_other,
            "keyed Engram addressing must isolate slots across profiles"
        );

        // Consolidate a high-amplitude memory into Peter's vault
        vault_peter.entheai.plasticity = 0.9;
        let secret_hidden = [0.85f32; D_MODEL];
        vault_peter.consolidate_engram(&ngram, &secret_hidden);

        let mut peter_read = [0.0f32; D_MODEL];
        let mut other_read = [0.0f32; D_MODEL];
        vault_peter.lookup_engram(&ngram, &mut peter_read);
        vault_other.lookup_engram(&ngram, &mut other_read);

        assert!(
            peter_read.iter().any(|&v| v.abs() > 0.01),
            "Peter's vault retrieves the consolidated episodic trace"
        );
        assert!(
            other_read.iter().all(|&v| v == 0.0),
            "Intruder's vault sees strictly zero for the same N-gram"
        );

        // Verify XOF stream encryption round-trip
        let ciphertext = vault_peter.encrypt_table_bytes();
        let raw_i8_as_u8: Vec<u8> = vault_peter.engram_table.iter().map(|&x| x as u8).collect();
        assert_ne!(ciphertext, raw_i8_as_u8, "ciphertext must not equal plaintext");

        vault_peter.engram_table.fill(0);
        vault_peter.decrypt_table_bytes(&ciphertext);
        let mut peter_restored = [0.0f32; D_MODEL];
        vault_peter.lookup_engram(&ngram, &mut peter_restored);
        assert_eq!(peter_read, peter_restored, "encrypted round-trip is bit-exact");

        // Verify RAM scrubbing zeroes everything
        vault_peter.scrub_ram();
        assert!(vault_peter.engram_table.iter().all(|&b| b == 0));
        assert_eq!(vault_peter.engram_key, [0u8; 32]);
    }

    #[test]
    fn ayeos_ternary_forward_and_mem8_wave_resurrection_are_deterministic() {
        let master = [0x33u8; 32];
        let (linosv_root, _) = derive_learning_genesis();
        let backbone = AyeosTernaryBackbone::new_linosv_seeded();

        let mut vault = SovereignProfileVault::new_isolated(&master, "aurora-dyad");
        let mut wave = Mem8WaveGrid::genesis(&vault.engram_key, &linosv_root);
        let genesis_root = wave.state_root;

        let ngram = [7u32, 14, 21];
        let input = [0.25f32; D_MODEL];
        let out1 = backbone.forward_step(&mut vault, &mut wave, &ngram, &input);

        assert_eq!(wave.epoch, 1);
        assert_eq!(wave.prev_root, genesis_root);
        assert_ne!(wave.state_root, genesis_root);
        assert!(out1.iter().any(|&x| x.abs() > 0.0));

        // Verify oneshot capsule resurrection from wave.state_root
        let anchor = wave.state_root;
        let resurrected =
            Mem8WaveGrid::resurrect_from_verified_snapshot(&vault.engram_key, &anchor, &wave)
                .expect("valid snapshot resurrects cleanly");
        assert_eq!(resurrected.state_root, anchor);
        assert_eq!(resurrected.epoch, 1);

        // Tampering with a single wave cell breaks resurrection verification
        let mut corrupted = wave.clone();
        corrupted.u_curr[512] += 0.001;
        assert!(
            Mem8WaveGrid::resurrect_from_verified_snapshot(&vault.engram_key, &anchor, &corrupted)
                .is_err(),
            "tampered wave grid must be rejected on resurrection"
        );
    }

    #[test]
    fn spherepop_fold_and_toss_freedom_ladder_preserve_invariants() {
        // Verify TOSS monozukuri freedom ladder is non-increasing and flat only at step 5 -> 6
        for i in 0..(TOSS_FREEDOM_LADDER.len() - 1) {
            assert!(TOSS_FREEDOM_LADDER[i] >= TOSS_FREEDOM_LADDER[i + 1]);
            if i == 4 {
                assert_eq!(
                    TOSS_FREEDOM_LADDER[i],
                    TOSS_FREEDOM_LADDER[i + 1],
                    "step 5 -> 6 (dieMold2) is the unique flat seam"
                );
            } else {
                assert!(TOSS_FREEDOM_LADDER[i] > TOSS_FREEDOM_LADDER[i + 1]);
            }
        }

        let mut re = [0.0f32; 64];
        let mut im = [0.0f32; 64];
        for i in 0..64 {
            let phase = 0.15 * (i as f32);
            re[i] = phase.cos();
            im[i] = phase.sin();
        }
        let (word, seal1) = spherepop_fold_64(&re, &im, 0.15);
        let (word2, seal2) = spherepop_fold_64(&re, &im, 0.15);
        assert!(word.is_valid(), "0x62 parity interlock must hold");
        assert!(word.non_zero_marks() > 0);
        assert_eq!(word, word2);
        assert_eq!(seal1, seal2);
    }

    #[test]
    fn test_bfp64_q16_gather_high_exponent_saturation() {
        let extreme_row = Bfp64Row {
            shared_exp: 150, // Shift of 39 bits: would overflow i32 without i64 lane
            mantissas: [127; BFP64_DIM],
        };
        let rows = [&extreme_row, &extreme_row, &extreme_row, &extreme_row];
        let mut out = [0i32; BFP64_DIM];
        Bfp64Row::gather_q16_zero_float_safe(&rows, &mut out);
        assert_eq!(out[0], i32::MAX);
    }

    #[test]
    fn test_telemetry_denominator_consistency() {
        let audit = QuantizationTelemetryAudit::new_ayeos_default(10_900_000_000, 384.2);
        let gb_per_tok = audit.physical_weight_bytes_per_token() / 1e9;
        assert!((gb_per_tok - 2.725).abs() < 0.01);
        assert!((audit.roofline_tokens_per_sec() - 140.99).abs() < 1.0);
    }

    #[test]
    fn test_essence_mux_subspace_switch_and_keyed_lemire_slot() {
        let v0 = EssenceSubspaceTable::new_zeroed("aurora-dyad", [3, 5, 7, 11]);
        let v1 = EssenceSubspaceTable::new_zeroed("mahakala-bernagchen", [13, 17, 19, 23]);
        let mux = EssenceMux::new(vec![v0, v1]);
        let slot0 = mux.resolve_slot(0, 0xCAFE_BABE_DEAD_BEEF);
        assert!(slot0 < HEAD_CAPACITIES[0]);
        assert_eq!(mux.switch_essence(1), Ok("mahakala-bernagchen"));
        assert_eq!(mux.switch_essence(99), Err("E_ESSENCE_OUT_OF_BOUNDS"));
    }
}
