# Reproducibility and evidence matrix

## Reference domain

The included water model fixes three agents, stock 0–5, need 0–2 and binary critical flags, for **1,296 finite world states**. Eight ordering operators times two reserve parameters yield **16 policies**, meaning **20,736 policy/state comparisons**. Critical-equal reserve 0 is feasible; CEGIS rejects two earlier candidates. The deterministic one-token impossibility certificate is checked independently over every integer outcome of the declared two-agent world.

**Evidence scope:** The [GitHub Actions workflow](../.github/workflows/check.yml) is configured to run Rust formatting, linting, tests, the reference tests, and Rust/Python differential comparison. A passing check establishes only what ran for that particular commit; cite its commit SHA and run URL with external results. No performance superiority or real-life institutional validity is established.

## Exact repeatable smoke commands

From a checkout of the repository root:

```sh
bash scripts/reproduce.sh
```

The smoke script compiles and runs the Rust CLI, checks a synthesized policy against the reference domain, validates the independent conflict witness, enumerates explicit relaxations, exercises finite multi-resource transitions and behavior, and compares 20,736 outputs to the Python reference.

**Evidence binding:** the CLI writes the exact canonical model representation and ordered candidate grammar into synthesis reports. `nomos verify` checks these against the supplied model before accepting the recomputed evidence. This deliberately does not claim signed provenance or byte-level file identity. Older unbound synthesis reports must be regenerated.

**Verification labels:** `BOUNDED_VERIFIED_IN_FINITE_GRAMMAR` (all declared reference worlds pass), `UNREALIZABLE_IN_FINITE_GRAMMAR` (only the bounded grammar), `MODEL_INFEASIBLE` (this explicitly enumerated admissible-outcome world), `UNKNOWN` (not proven), and `CERTIFICATE_CHECKED` (exactly re-evaluated evidence). No SMT-backed or universal claims.

**Not yet established:** reproducible solver speed comparisons; symbolic theorem proving; sampled or calibrated real-world human behavior; parser fuzz campaign coverage; coverage across compilers/OS architectures beyond observed CI; publication-level algorithmic novelty.

## Experiment provenance

`reference/nomosforge_reference.py` contains the original bounded executable specification. The committed `reference/nomosforge_experiment_results.json` is regenerated and compared in CI, not treated as external proof. `scripts/manifest.py` computes the SHA-256 of tracked research artifacts when invoked; no precomputed fake hashes are asserted. For independent claims, capture the Git commit, generated manifest, rustc/cargo/python versions, machine and benchmark flags.

**Dependency locking:** The committed workspace `Cargo.lock` records the selected Rust dependency versions and registry checksums. The lockfile was resolved using an MSRV-aware Cargo resolver targeting Rust 1.82, retained in the Cargo 1.82-compatible lockfile v3 format, and verified by a locked test and build on Rust 1.82. CI and the smoke script use `--locked` to reject silent dependency re-resolution. This improves repeatability of dependencies, but does **not** imply byte-for-byte reproducible binaries across toolchains, operating systems, or environments.

`cargo test` and the checked reference oracle do not substitute for an adversarial security or policy audit. The `fuzz/` harness is available but not part of normal CI.
