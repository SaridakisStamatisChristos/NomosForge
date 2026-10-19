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

**Verification labels:** `BOUNDED_VERIFIED_IN_FINITE_GRAMMAR` (all declared reference worlds pass), `UNREALIZABLE_IN_FINITE_GRAMMAR` (only the bounded grammar), `MODEL_INFEASIBLE` (this explicitly enumerated admissible-outcome world), `UNKNOWN` (not proven), and `CERTIFICATE_CHECKED` (exactly re-evaluated evidence). No SMT-backed or universal claims.

**Not yet established:** reproducible solver speed comparisons; symbolic theorem proving; sampled or calibrated real-world human behavior; parser fuzz campaign coverage; coverage across compilers/OS architectures beyond observed CI; publication-level algorithmic novelty.

## YAML parser dependency

The finite-model parser uses `serde_norway 0.9.42`, a maintained fork of the retired `serde_yaml` crate, rather than relying on the original unmaintained crate. It retains the same Serde typed decoding API, including unknown-field rejection and v0.1 post-parse validation. Parser regression tests assert reference-model fields, equivalent YAML quoting/comment syntax, and fail-closed rejection of malformed, duplicate, or incorrectly typed inputs. The normal CI exercises the exact finite-model workflow, full reference corpus, and the declared Rust 1.82 MSRV. No conclusion about universal YAML equivalence or parser security follows from these tests. See [the parser migration decision](decisions/0002-yaml-parser.md).

## Versioned synthesis evidence

CLI `synth` emits an `evidence` object (`nomos/synthesis-evidence/v1`) with four SHA-256 identifiers for the **parsed typed model**, **ordered grammar**, **ordered enumerated worlds**, and **complete synthesis report**, plus declared producer and semantic versions. SHA-256 inputs use deterministic serde JSON for typed structs and vectors and domain-separated byte-length framing. Changes to YAML formatting alone do not alter the model fingerprint; a change to a parsed model field or grammar sequence does. The checker re-enumerates and re-synthesizes instead of trusting hashes as proof. Old reports without `evidence` are rejected; regenerate them using `nomos synth`. Report format changes are intentionally versioned.

These identifiers are unkeyed integrity checks, **not digital signatures** or guarantees about who generated an artifact. The producer version is the crate version, not a cryptographic binding to its Git commit; for audits record the Git commit, manifest and CI run URL separately. The evidence schema and declared semantics identifier must be revised when the corresponding model semantics change.

## Experiment provenance

`reference/nomosforge_reference.py` contains the original bounded executable specification. The committed `reference/nomosforge_experiment_results.json` is regenerated and compared in CI, not treated as external proof. `scripts/manifest.py` computes the SHA-256 of tracked research artifacts when invoked; no precomputed fake hashes are asserted. For independent claims, capture the Git commit, generated manifest, rustc/cargo/python versions, machine and benchmark flags.

**Dependency locking:** The committed workspace `Cargo.lock` records the selected Rust dependency versions and registry checksums. The lockfile was resolved using an MSRV-aware Cargo resolver targeting Rust 1.82, retained in the Cargo 1.82-compatible lockfile v3 format, and verified by a locked test and build on Rust 1.82. CI and the smoke script use `--locked` to reject silent dependency re-resolution. This improves repeatability of dependencies, but does **not** imply byte-for-byte reproducible binaries across toolchains, operating systems, or environments.

`cargo test` and the checked reference oracle do not substitute for an adversarial security or policy audit. The `fuzz/` harness is available but not part of normal CI.
