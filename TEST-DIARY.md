# TEST-DIARY.md — `negative-space-testing` v0.2.0

**Tester:** Tomás (game physics engine dev)
**Date:** 2026-06-02
**Crate:** https://crates.io/crates/negative-space-testing
**Repo:** https://github.com/SuperInstance/negative-space-testing

---

## 1. First Impression

> "negative space testing" — does it make sense for game physics?

**Kind of, but not how you'd think.**

I came in expecting spatial analysis: "negative space" in game physics means the gaps in collision geometry — holes in navmeshes, corridors that don't connect, rooms unreachable from the spawn point. The name strongly implies you're mapping what *isn't* where it should be.

The crate *isn't* about that. It's about **testing methodology** — writing assertions about what your code must never do. "Negative space" here means forbidden outputs, not spatial emptiness. It's a testing philosophy inspired by the metaphor of the "meteorologist's blindness" (over-labeling kills imagination).

The mismatch matters: if a game dev searches for "negative space testing" hoping to find hole-detection in navmeshes, they'll find assertions about function outputs instead. The README leans hard into poetic metaphors (crackle glaze, cathedrals, meteorologists) without signaling this clearly enough to spatial/problem-domain searchers.

**Verdict on title alone:** Clever concept, misleading name for a game physics audience.

---

## 2. Build — cargo test & clippy

```text
$ cargo test
141 passed (140 unit + 1 doc-test)

$ cargo clippy
(no warnings)
```

**Clean as a whistle.** No warnings, no failures, full test coverage. The crate is well-tested and the CI would be green.

For a crate with algebraic topology code (Smith normal form, QR decomposition, Betti numbers), this is *impressive* — those are notoriously hard to get right and the test suite is thorough.

---

## 3. Architecture — Reading ALL Source

### What's exported (lib.rs, 580 lines):

| Module | Type | What it does |
|--------|------|-------------|
| `NegativeTest<T>` | Struct | Register forbidden predicates; check values against them |
| `SpaceMap<K,V>` | Struct | Two-zone map (occupied vs forbidden); detects intrusions |
| `ConservationChecker` | Struct | Track quantities that must never decrease beyond tolerance |
| `CracklePhase<T>` | Struct | Deferred assertions checked during a "cooling" phase |
| `CathedralProbe` | Struct | Laplacian eigenvalues of a component graph (spectral analysis) |

### What's NOT exported (but exists in source):

| File | Why notable |
|------|-------------|
| `negative_test.rs` | Redundant `NegativeTest` **trait** + `SpaceMap<T>` version — different API from the exported one in `lib.rs`. Feels like it was refactored mid-stream and both versions were kept. |
| `topology.rs` | **700+ line** algebraic topology library: simplicial complexes, Betti numbers, Smith normal form, persistent homology, nerve construction. **None of this is re-exported!** |
| `cathedral.rs` | Second version of `CathedralProbe` (relationship-based, not spectral) — not exported |
| `conservation.rs` | Second version of `ConservationChecker` (with *Monotonicity* enum) — not exported |
| `crackle.rs` | Second version of `CracklePhase` (consumes self, FnOnce) — not exported |

### The Unaddressed Elephant

The crate has an **entire algebraic topology library** sitting in `src/topology.rs` — simplicial complexes, boundary matrices, Smith normal form (SNF), Betti number computation, persistent homology, nerve construction — and it's **not publicly exported from lib.rs**. No `pub mod topology;` in the lib.rs. The `cathedral.rs`, `conservation.rs`, `crackle.rs` modules are also internal-only.

This feels like the crate started as a topological data analysis library for testing, got a refactor into the current simpler API, and the old code was left in. The topology code is genuinely well-written with solid tests — it deserves to be either dropped (dead code) or properly exposed.

### Quality Assessment

- **Code quality:** Good. `#![deny(unsafe_code)]`, no warnings, well-documented with proper references (Hatcher 2002, Edelsbrunner & Harer 2010).
- **API design:** Reasonable. The `forbid/check` pattern is ergonomic.
- **Missing:** Generic `Monotonicity` and `NegativeTest` trait from the private modules are *better* than the public API. The public API's `ConservationChecker.register()` lacks the `Monotonicity` parameter — you only get "one-sided" non-decreasing.

---

## 4. Real Test — Spatial Query Analysis for a Game Level

I built `examples/game_physics_test.rs` — a complete example that:

1. **Defines a 2D game level** with Floor/Wall/Pit/Spawn/Goal cells
2. **NegativeTest**: Checks all walkable cells for forbidden pits (holes in the collision mesh)
3. **SpaceMap**: Maps forbidden zones vs occupied regions; checks for intrusions
4. **ConservationChecker**: Tracks game state invariants (HP, ammo, items) across gameplay
5. **BFS Reachability**: Finds disconnected level regions and cross-references with pits
6. **Cross-analysis**: Identifies wasted geometry — pits in unreachable areas

### Results

All checks passed. The `NegativeTest` correctly flagged zero pits in clean levels, `SpaceMap` tracked negative space ratio, and `ConservationChecker` caught health violations. The BFS reachability analysis found that a level with a wall split had 39 unreachable walkable cells — with a pit hidden in the unreachable zone.

### What it revealed

The crate **can** be applied to spatial problems, but you have to build the spatial logic yourself. The crate provides the *framework* (forbid predicates, zone maps, invariants) but none of the *spatial tools* (mesh intersection, navmesh analysis, pathfinding). For a game engine use case, you'd need to graft your own collision detection on top.

---

## 5. What's Missing for Game Engine Use

### Critical gaps

| Gap | Impact |
|-----|--------|
| **No WASM build support** | Can't run in-browser game tests |
| **No parallelism** | `Send + Sync` bounds on predicates, but no `rayon` integration or parallel `check_all` |
| **No real-time support** | `Check` isn't designed for frame-by-frame assertions; `ConservationChecker` is batch-oriented |
| **No spatial data structures** | No BVH, no octree, no navmesh analysis — you build it yourself |
| **No `no_std` support** | Can't run on GPU or embedded game hardware |
| **Topology module hidden** | The Betti number / persistence homology code that *would* be useful for spatial analysis is internal-only |

### What would it take

- Expose `pub mod topology` with serde support
- Add `Monotonicity` to public `ConservationChecker` (it's only in the private `conservation.rs`)
- Nightly `parallel` feature flag for rayon-backed check_all
- WASM target CI
- Example showing n-body or spatial-hash collision testing

---

## 6. Score: 3.5 / 5 ★ — A Game Dev's Honest Opinion

```
★★★★☆  Concept
★★★☆☆  Name clarity
★★★★★  Code quality
★★☆☆☆  Game physics relevance
★★★☆☆  Practical utility
```

### Breakdown

- **What it does well:** Clean API, solid tests, interesting philosophical approach. The `NegativeTest`/`SpaceMap`/`ConservationChecker` combo is genuinely useful for general assertion-based testing.
- **What holds it back for games:** The name promises spatial problem-solving but delivers test methodology. The algebraic topology module (which *would* be relevant for spatial analysis if exposed) is hidden. No performance features (parallel, WASM, no_std).
- **Best use case for a game dev:** Use it *inside* your test suite to express "my physics engine must never produce NaN velocities" or "no two rigidbodies should have overlapping AABBs." Don't use it for runtime spatial queries.
- **Hidden gem:** The `topology.rs` module is genuinely interesting for test-suite structural analysis. If exposed, it could do things like "detect tests that form circular dependency cycles (β₁ > 0)" or "find coverage gaps (β₂ > 0)."

**Would I use it in a game engine?** Maybe for test infrastructure. Definitely not for runtime. The name sets the wrong expectation for my field, but the engineering is solid.
