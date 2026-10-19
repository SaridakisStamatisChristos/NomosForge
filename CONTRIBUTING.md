# Contributing to NomosForge

Open an issue describing the formal model, expected semantic property, counterexample, and reproduction steps. Prefer one behavioral change per PR, accompanied by a reference fixture and tests. All changes must preserve the evidence taxonomy.

The primary contributor path is [the fresh-checkout quickstart](README.md#quickstart-from-a-fresh-clone). Changes to the CLI user journey should also run `bash scripts/smoke_install.sh` (it performs an isolated local Cargo install; no publication or deployment). For model semantics, preserve the existing independently checked finite corpus.

Before opening a PR:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
python3 -m pytest -q reference/test_nomosforge_reference.py
bash scripts/reproduce.sh
bash scripts/smoke_install.sh
```

Never weaken a hard invariant, omit an input domain in a proof claim, reinterpret a heuristic as sound global search, or invent performance results. Document new axioms' assumptions and who chooses them. Pure algorithmic changes go to the relevant Rust crate; leave platform and infrastructure concerns separate. Report any regressions with the smallest failing witness. See [scope guidance](docs/proof-boundaries.md).
