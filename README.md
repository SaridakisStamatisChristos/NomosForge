# NomosForge

**Finite institutional-rule synthesis with checkable evidence and explicit limits.** NomosForge takes a bounded allocation model, searches a finite grammar of candidate rules, and either produces a policy that satisfies the model, reports a replayable counterexample, or identifies a strictly scoped infeasibility.

> **Research software, not a governance or decision service.** A finite-model proof does not validate moral assumptions, human behavior, or real-world policy outcomes. CEGIS, finite enumeration, and the two-agent impossibility example use established techniques; no algorithmic novelty or performance-superiority claim is made.

## Quickstart from a fresh clone

Requirements: Git, Rust and Cargo **1.82 or later** (Rust 2021 workspace). The first build needs registry access to download the dependencies pinned by `Cargo.lock`. **Python 3.12+** is needed for the independent reference/reproducibility script, but not to run the Rust CLI.

```bash
git clone https://github.com/SaridakisStamatisChristos/NomosForge.git
cd NomosForge
cargo build --workspace --locked
cargo run --locked --quiet -p nomos-cli -- --help
cargo run --locked --quiet -p nomos-cli -- validate examples/water-allocation/world.nomos.yaml
cargo run --locked --quiet -p nomos-cli -- synth examples/water-allocation/world.nomos.yaml --engine enumerate --json target/nomos-policy.json
cargo run --locked --quiet -p nomos-cli -- verify target/nomos-policy.json --full-domain --model examples/water-allocation/world.nomos.yaml
```

The validation command should print `VALID: water-rationing (nomos/v0.1)`. Successful verification should print `CERTIFICATE_CHECKED: BOUNDED_VERIFIED_IN_FINITE_GRAMMAR` for **1,296** enumerated states. The 16-program reference grammar contains a feasible `critical_equal` / reserve-0 policy. The example report is written under the ignored `target/` directory. Run commands from the repository root.

**Install the CLI locally** (optional): `cargo install --locked --path crates/nomos-cli`, then run `nomos --help` if Cargo's `bin` directory is on your `PATH`. For an isolated install with its own destination and a real run/verify smoke check, use `bash scripts/smoke_install.sh` (Bash on macOS/Linux; Git Bash/WSL on Windows). The normal `cargo run` commands work anywhere Rust supports the platform.

## Other bounded demonstrations

```bash
# Conflict/explain intentionally return status 3 for checked infeasibility.
cargo run --locked --quiet -p nomos-cli -- conflict > target/nomos-conflict.json || test "$?" -eq 3
cargo run --locked --quiet -p nomos-cli -- explain target/nomos-conflict.json || test "$?" -eq 3

# Inspect explicit model relaxations and bounded feasible frontiers.
cargo run --locked --quiet -p nomos-cli -- relax examples/symmetric-single-token/world.nomos.yaml --show-all-minimal
cargo run --locked --quiet -p nomos-cli -- frontier examples/water-allocation/world.nomos.yaml
```

The conflict example concerns two identical agents, one indivisible token and strict deterministic anonymity together with full utilization. Equal halves, randomization, or leaving stock unused **change the assumptions**; they are not newly proved policies for actual people.

## CLI contract and troubleshooting

| Exit code | Meaning |
| --- | --- |
| `0` | Successful operation or verified feasible bounded synthesis |
| `2` | Invalid inputs/arguments or a read/parse error |
| `3` | Checked model/grammar infeasibility or a declared finite failure result |
| `4` | Incomplete bounded synthesis (`UNKNOWN`) |
| `5` | Unrecognized synthesis status |

Use `cargo run --locked --quiet -p nomos-cli -- --help` to see all commands and `--version` for the crate version. If Cargo reports a missing toolchain, install Rust 1.82+ from the official Rust toolchain distribution; if `--locked` reports dependency drift, check that `Cargo.lock` matches the committed repository instead of silently regenerating it. If verification reports an evidence-provenance mismatch, pass the *same original model* with `--model` (and regenerate an old unbound report using `synth`); hashes are **not** signatures or independent proofs. Codes `3` for `conflict`/`explain` are intentional, not application crashes.

## Reproducibility

```bash
bash scripts/reproduce.sh
```

The script tests the Rust workspace, checks synthesis and scoped conflict certificates, and executes **20,736** independent Rust/Python allocation comparisons in the committed reference domain. CI separately tests stable Rust and the declared Rust 1.82 minimum supported version. A passing workflow proves only what ran on that commit; it is **not** evidence of general formal verification or production security.

Optional measurements: run `cargo build --locked -p nomos-cli`, install Python's `PyYAML` dependency in your preferred isolated environment, then run `python3 scripts/bench_finite.py --binary target/debug/nomos --repeats 3`. These are descriptive end-to-end CLI times (startup and serialization included), **not** solver-core comparisons or a SOTA claim. The optional `fuzz/` harness is not exercised in baseline CI.

See [reproducibility and provenance](docs/reproducibility.md) and the [verification matrix](docs/verification-matrix.md).

## Architecture

The workspace separates `nomos-types`, `nomos-parse`, `nomos-grammar`, `nomos-execute`, `nomos-synth`, `nomos-verify`, `nomos-conflict`, `nomos-report`, and `nomos-cli`. Read the [architecture diagram and crate boundaries](docs/architecture.md) or the [documentation homepage](docs/index.html).

The synthesizer checks only the declared finite domains and grammar. Transition checks inspect supplied traces rather than all shock sequences; behavioral examples enumerate operator-defined scenarios, not observed human response. The two-agent result is an assumption-scoped impossibility, not a policy recommendation. The synthesis report binds typed-model, grammar, domain, and transcript fingerprints, and `verify` independently recomputes and replays the finite result.

## Contributing and usage boundaries

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), the [research limitations](docs/proof-boundaries.md), and [prior art](docs/prior-art.md) before proposing changes. NomosForge is licensed under [Apache-2.0](LICENSE). No hosted service, account, API key or GPU is required for the local reference workflow.
