# Creator mode — a memnet-native learning surface

The reader hub ([`../ultra-omni-zeno-education`](../ultra-omni-zeno-education/README.md))
hands a learner seven modules to *consume*. **Creator mode** inverts that: the
learner *authors* an **admissible fragment** (a learning card), and memnet
compresses it, folds it, interlaces it into the engram, and appends one
breadcrumb. Learning is authoring; the record is the proof.

Everything it needs already lives in `memnet`. This document names the join
points and the one new surface to add. Nothing is reimplemented: the fold, the
bottleneck, and the interlock are the oracle, and the oracle wins.

---

## 1. What already exists (do not rebuild)

| Concern | Symbol | File |
|---|---|---|
| Unit-sphere fold + seal | `spherepop_fold_64(&[f32;64], &[f32;64], f32) -> (ComplexWord64, [u8;32])` | [`src/sovereign_convergence.rs`](src/sovereign_convergence.rs#L844) |
| Parity-interlocked contraction | `PackedComplexTernaryRow64::{verify_interlock, contract_row64_popcnt}` | [`src/spherepopfold.rs`](src/spherepopfold.rs#L39) |
| Ternary alphabet | 9-state complex ternary `{-1,0,+1}+i{-1,0,+1}`, four bit-planes | [`src/complex_ternary.rs`](src/complex_ternary.rs) |
| System 1 / System 2 | `CognitiveMode::{System1FastReflex, System1To2UltraZenoBridge, System2DeepDeliberation}` | [`src/ultra_zeno.rs`](src/ultra_zeno.rs#L66) |
| M1 structural compression | `run_ultra_zeno_bottleneck(&UltraZenoParams) -> UltraZenoSummary` (M1 = degree entropy + KL surprise) | [`src/ultra_zeno.rs`](src/ultra_zeno.rs#L254) |
| Web surface | Astro site: `GuardianPlayground`, `LibraryExplorer`, `.m8` library | [`site/`](site/) |

Two fragments of the brief are **already implemented**, not new:

- **`system 1, system 2`** = `CognitiveMode`. The routing is a function of
  M1 surprise: low degree-entropy + low KL → `System1FastReflex`; high surprise
  → `System2DeepDeliberation` (full dyadic schedule + M3 clique/LLL).
- **`M1 == structural compression`** = the first rank of the UltraZeno
  bottleneck: `m1_degree_entropy_bits` + `m1_kl_surprise_bits` over the card's
  concept graph. It is exactly the "level-1 model" the brief calls M1.

---

## 2. Decoding the remaining brief

| Fragment | Meaning | Join point |
|---|---|---|
| `entropic seed` | deterministic seed → balanced trits `{-1,0,+1}` (reproducible, no RNG) | card identity + trit extraction |
| `clipping economy` | threshold-quantize a continuous signal to 3 centroids, 1.58 bits/trit | `complex_ternary` packing |
| `holonomy` | position-dependent bend `(k·i) mod 3`, a cyclic ring `-1→0→+1→-1` | the card's place in the loop |
| `R1,R2,R3 <~ STRUCTURE` | three representation ranks, reducible to the M1/M2/M3 ladder | §3 |
| `ADMISSABILITY gRAMMAR === engram*qT` | admissibility = memory × quantized ternary | §5 |
| `perfectly interlocking?` | no `+1`/`-1` overlap in a trit, parity checksum | `verify_interlock` |
| `tension OR self-reflective 3trit -1` | the `-1` trit = refusal = the trigger that opens self-reflection | §4 |
| `breadcrumbs lol` | append-only ledger; "history is identity" | §6 |
| `system design ultrabackyardloop` | build it one `reflect→improve→wire→polish→push→readme` lap | §7 |

---

## 3. The R1 → R2 → R3 ladder (over M1 → M2 → M3)

`<~` in the brief reads as "reduces to". The three *representation* ranks a
card passes through sit directly on the three *structural* ranks `ultra_zeno`
already compresses:

- **R1 — the gesture.** Raw text, or a VAD affect triple
  (`valence·arousal·dominance`, each `-1..+1`). Uncompressed. This is `System1`'s
  input.
- **R2 — the ternary.** Seed → balanced trits → bent by the holonomy `(k·i) mod 3`
  → clipped to three centroids → packed into `complex_ternary` bit-planes.
  This *is* M1: the structural compression, measured in bits by
  `run_ultra_zeno_bottleneck`'s M1 telemetry.
- **R3 — the fold.** The packed card becomes a `64`-dim latent vector, is
  carried onto the unit sphere `S^63`, folded by `spherepop_fold_64`, and sealed.
  This is M2/M3 territory: the quotient that survives compression, sealed so
  history is identity.

`R1 → R2` loses signal by design (clipping); `R2 → R3` loses nothing reversible
(the seal pins the fold). Compression is a commitment, not a cache.

---

## 4. Dual process as a gate, not a label

- **System 1 (fast)** — the author gives an immediate `{-1,0,+1}` verdict:
  `+1` affirm, `0` rest ("don't know" is real, not a rounding error), `-1`
  refuse.
- **System 2 (slow)** — `CognitiveMode` routes on M1 surprise. A `-1` (tension)
  is *not* a rejection of the learner; it is the OR branch that routes to
  `System2DeepDeliberation`, the self-reflection that keeps the loop alive.

Nothing is admitted on gesture alone: System 1 filters cheaply, System 2 decides
at cost, and the admissibility grammar (below) is the final gate.

---

## 5. The admissibility grammar (`engram × qT`)

A card is **admissible** iff its ternary form contracts cleanly against the
engram — the memory memnet already holds — with **zero forbidden overlap**:

```text
admissible(card)  ⟺  (re_pos & re_neg) | (im_pos & im_neg) == 0
                      ∧  parity_byte matches
```

This is CT-000 restated for cards: *a guaranteeing policy exists exactly when no
class blurs.* A blurred card (one that must be both `+1` and `-1` in the same
position to fit) is refused and named, never silently rewritten. The grammar is
the product of two things that already exist: the **engram** (the append-only
wave memory, `MEM|8`) and **qT** (the quantized ternary from `complex_ternary`).

---

## 6. The card lifecycle (the one new surface)

A `creator_mode` module in `memnet` (e.g. `src/creator_mode.rs`), composing the
oracle — no new math, only the lifecycle:

```text
author(seed, gesture, verdict)
  → extract trits(seed)                 # entropic seed
  → bend(k·i mod 3) · clip · pack       # R2 = M1 structural compression
  → spherepop_fold_64(…)                # R3 = fold + seal
  → verify_interlock(…)                 # "perfectly interlocking?"
  → contract vs engram                  # admissibility grammar
  → admissible ? append breadcrumb : refuse(-1)
```

Each accepted card appends one breadcrumb. The trail *is* the learner's identity
("history is identity"), so it is append-only and never rewritten.

Proposed surface, mirroring the existing `site/` structure:

```text
src/creator_mode.rs          # the lifecycle above (calls ultra_zeno, spherepopfold, complex_ternary)
site/src/content/docs/creator/index.mdx   # authoring page, alongside the existing playground
site/src/components/CreatorCard.astro     # one card: gesture · verdict · proof
```

---

## 7. Build it one lap at a time

The method is the DO loop ([`../crush-love-dev/backyardloop.rs`](../crush-love-dev/backyardloop.rs)):

1. `reflect` — the brief above.
2. `improve` — `src/creator_mode.rs` with the lifecycle + unit tests.
3. `wire` — expose it from `lib.rs`, add the Astro authoring page.
4. `polish` — self-only CSP, no network at build, `localStorage` progress.
5. `push` — deferred: the workspace has no git remote.
6. `readme` — record the lap in a `docs/ULTRABACKYARDLOOP.md` + ledger.

---

## 8. Gotchas

- **The oracle wins.** If a folded card disagrees with `spherepop_fold_64`, the
  bug is in `creator_mode`, not the oracle. Point at the oracle; never re-derive
  the fold "more simply" here.
- **Two ternary alphabets.** The *gate* codec is `-1 0b00 · 0 0b01 · +1 0b10`;
  the *weight* codec (`ternary-lane/pack.rs`) is `0 0b00 · +1 0b01 · -1 0b10`.
  Creator mode deals in the gate codec.
- **`-1` is load-bearing.** A zero is a real "don't know"; a `-1` is a real
  refusal. Out-of-range is a runtime error, never a clamp.
- **No consciousness claims.** The fold, the seal, and the engram are arithmetic
  and memory handles. Prose and commit messages must not imply an interior.
- **Zero dependencies.** `memnet` has `[dependencies]` empty; `creator_mode.rs`
  must keep it that way — the entropy, KL, POPCNT, and fold are all hand-rolled
  on `f64`/`u64`/`POPCNT`.
