# NomosForge

**Institutional rule synthesis with explicit evidence boundaries.** NomosForge compiles a finite grammar of allocation policies against declared constraints and either finds a rule that works on the bounded model, returns a checkable counterexample, or explains a precisely scoped impossibility.

> **Research software; not a government-policy advisor.** The current engine verifies exact finite toy models, not entire societies, human behavior, or moral legitimacy. The synthesis technique (CEGIS), finite enumeration, and the two-person indivisible-goods impossibility are established concepts; no algorithmic novelty claim is made.

## Two executable demonstrations

Requires Rust and Cargo (edition 2021; declared MSRV 1.82) and Python 3.12+ for the independent reference and CI tools. CI tests current stable Rust and independently tests the declared Rust 1.82 minimum supported version. No service, account, database, API key, or GPU.

```bash
# 1. Produce and check a policy against the 1,296-state reference domain.
cargo run --locked --quiet -p nomos-cli -- validate examples/water-allocation/world.nomos.yaml
cargo run --locked --quiet -p nomos-cli -- synth examples/water-allocation/world.nomos.yaml --engine enumerate --json /tmp/nomos-policy.json
cargo run --locked --quiet -p nomos-cli -- verify /tmp/nomos-policy.json --full-domain

# 2. Produce a scoped two-agent impossibility certificate and check it.
# Conflict and explanation exit with code 3 (verified model-level infeasibility).
cargo run --locked --quiet -p nomos-cli -- conflict > /tmp/nomos-conflict.json || test "$?" -eq 3
cargo run --locked --quiet -p nomos-cli -- explain /tmp/nomos-conflict.json || test "$?" -eq 3

# 3. Explore model changes and bounded feasible policies.
cargo run --locked --quiet -p nomos-cli -- relax examples/symmetric-single-token/world.nomos.yaml --show-all-minimal
cargo run --locked --quiet -p nomos-cli -- frontier examples/water-allocation/world.nomos.yaml
```

**Expected scope:** 3 agents, stock 0–5, needs 0–2, critical flags 0/1 = 1,296 worlds. The 16-program grammar has a feasible `critical_equal` / reserve-0 allocation policy. Its CEGIS trace rejects two earlier candidate policies. A symmetric two-agent, one-indivisible-token world has no integer allocation satisfying *both* full utilization and strict deterministic anonymity. Permitting a lottery, equal halves, or unused stock changes the assumptions.

## Reproduce the experiment

```bash
bash scripts/reproduce.sh
```

This script builds the CLI, tests the Rust workspace, runs two showcases and the independent 20,736-case Python/Rust oracle comparison. Cargo uses the committed `Cargo.lock` with `--locked` so dependency resolution cannot silently change. See [reproducibility matrix](docs/reproducibility.md) for evidence and limitations. The optional fuzzing and benchmark tools are not assertions of results. CI runs the release smoke script on each PR.

## Architecture

| Crate | Scope |
|---|---|
| `nomos-types` | Finite typed world, priority, and policy contracts |
| `nomos-parse` | Strict declarative YAML reader |
| `nomos-grammar` | Explicit 16-program grammar and finite universe enumeration |
| `nomos-execute` | Deterministic greedy allocation and hard-check semantics |
| `nomos-verify` | Finite policy and trace checking, hypothetical behaviors |
| `nomos-synth` | Witness-caching counterexample-guided synthesis |
| `nomos-conflict` | All-outcomes integer impossibility certificate and rational relaxations |
| `nomos-report` | Multiobjective dominance and constraint sensitivity |
| `nomos-cli` | `nomos` command-line research workflow |

[Documentation homepage](docs/index.html) · [Design decisions](docs/decisions/0001-evidence-scope.md) · [Reference tests](reference/test_nomosforge_reference.py) · [Research limits](docs/proof-boundaries.md) · [Prior art](docs/prior-art.md) · [Contributing](CONTRIBUTING.md)

## Evidence conventions

`BOUNDED_VERIFIED_IN_FINITE_GRAMMAR` = passes **all** enumerated worlds, not outside the domain. `UNREALIZABLE_IN_FINITE_GRAMMAR` = no program in the specified grammar works; not model-level impossibility. `MODEL_INFEASIBLE` = no admissible allocation works in **a specific** declared symmetric two-person state. `UNKNOWN` = computation budget exceeded or incompleteness. `CERTIFICATE_CHECKED` is emitted only when the checker re-evaluates the evidence.

The initial implementation milestones are merged into `main`; their reviews remain in the [pull request history](https://github.com/SaridakisStamatisChristos/NomosForge/pulls?q=is%3Apr+is%3Amerged). Contributions welcome under [Apache-2.0](LICENSE).
