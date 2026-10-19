# Contributing to NomosForge

Open an issue describing the formal model, expected semantic property, counterexample, and reproduction steps. Prefer one behavioral change per PR, accompanied by a reference fixture and tests. All changes must preserve the evidence taxonomy.

Before opening a PR:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
python3 -m pytest -q reference/test_nomosforge_reference.py
bash scripts/reproduce.sh
```

Never weaken a hard invariant, omit an input domain in a proof claim, reinterpret a heuristic as sound global search, or invent performance results. Document new axioms' assumptions and who chooses them. Pure algorithmic changes go to the relevant Rust crate; leave platform and infrastructure concerns separate. Report any regressions with the smallest failing witness. See [scope guidance](docs/proof-boundaries.md).
