# Reproducibility and evidence matrix

## Reference domain

The included water model fixes three agents, stock 0–5, need 0–2 and binary critical flags, for **1,296 finite world states**. Eight ordering operators times two reserve parameters yield **16 policies**, meaning **20,736 policy/state comparisons**. Critical-equal reserve 0 is feasible; CEGIS rejects two earlier candidates. The deterministic one-token impossibility certificate is checked independently over every integer outcome of the declared two-agent world.

**Evidence at PR09:** GitHub CI passed Rust formatting/linting/tests and the Python reference plus exact Rust/Python differential comparisons on the corrected PR09 head. This statement does not imply any published performance result or real-life institutional validity. Run URLs and exact revision should accompany future external reports.

## Exact repeatable smoke commands

From repository root, on the branch incorporating PR01–PR10:

```sh
bash scripts/reproduce.sh
```

The release smoke path compiles and runs the Rust CLI, checks a synthesized policy against the reference domain, validates the independent conflict witness, enumerates explicit relaxations, exercises finite multi-resource transitions and behavior, and compares 20,736 outputs to the Python reference.

**Verification labels:** `BOUNDED_VERIFIED_IN_FINITE_GRAMMAR` (all declared reference worlds pass), `UNREALIZABLE_IN_FINITE_GRAMMAR` (only the bounded grammar), `MODEL_INFEASIBLE` (this explicitly enumerated admissible-outcome world), `UNKNOWN` (not proven), and `CERTIFICATE_CHECKED` (exactly re-evaluated evidence). No SMT-backed or universal claims.

**Not yet established:** reproducible solver speed comparisons; symbolic theorem proving; sampled or calibrated real-world human behavior; parser fuzz campaign coverage; coverage across compilers/OS architectures beyond observed CI; publication-level algorithmic novelty.

## Experiment provenance

`reference/nomosforge_reference.py` contains the original bounded executable specification. The committed `reference/nomosforge_experiment_results.json` is regenerated and compared in CI, not treated as external proof. `scripts/manifest.py` computes the SHA-256 of tracked research artifacts when invoked; no precomputed fake hashes are asserted. For independent claims, capture the Git commit, generated manifest, rustc/cargo/python versions, machine and benchmark flags.

`cargo test` and the checked reference oracle do not substitute for an adversarial security or policy audit. The `fuzz/` harness is available but not part of normal CI.
