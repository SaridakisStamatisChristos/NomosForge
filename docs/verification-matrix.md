# Evidence matrix

The independent Python interpreter enumerates 1,296 ordered legal states and 16 ordered policies, or **20,736** state-policy pairs. The Rust `nomos corpus` command exports each allocation and first violated axiom; `reference/compare_rust.py` compares every JSON row to independently calculated Python values and then compares bounded CEGIS transcripts.

The certificate checker separately reconstructs all four integer outcomes of the one-token impossibility and falsifies tampered rows/assumptions. Rust property tests exercise bounded conservation, cap and a known feasible policy. The normal Rust CI also executes deterministic single-character deletion and punctuation/control-character insertion mutations against the reference YAML fixture, asserting that the parser does not panic. This is a reproducible malformed-input smoke suite, not a fuzz-duration or vulnerability-freedom claim. An optional cargo-fuzz harness targets the strict YAML parser, and bench_finite.py emits measured CLI times with their scope explicitly disclosed. Do not cite performance or fuzz results until those tools actually run.

Run:
```bash
cargo test --workspace --all-targets
cargo run --locked --quiet -p nomos-cli -- corpus > /tmp/nomos-corpus.jsonl
cargo run --locked --quiet -p nomos-cli -- synth examples/water-allocation/world.nomos.yaml --json /tmp/nomos-synth.json
python3 reference/compare_rust.py /tmp/nomos-corpus.jsonl /tmp/nomos-synth.json
python3 -m pytest -q reference/test_nomosforge_reference.py
```
