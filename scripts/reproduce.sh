#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT
echo "[1/8] Rust toolchain and workspace"
rustc --version
cargo --version
cargo build --workspace --locked
cargo test --workspace --all-targets --locked
echo "[2/8] Validate and synthesize exact finite model"
cargo run --locked --quiet -p nomos-cli -- validate examples/water-allocation/world.nomos.yaml
cargo run --locked --quiet -p nomos-cli -- synth examples/water-allocation/world.nomos.yaml --engine enumerate --json "$temp/policy.json"
cargo run --locked --quiet -p nomos-cli -- verify "$temp/policy.json" --full-domain
echo "[3/8] Python reference and exact differential checks"
python3 reference/nomosforge_reference.py --output "$temp/python.json"
python3 -c 'import json,sys; assert json.load(open(sys.argv[1]))==json.load(open(sys.argv[2]))' "$temp/python.json" reference/nomosforge_experiment_results.json
cargo run --locked --quiet -p nomos-cli -- corpus > "$temp/corpus.jsonl"
python3 reference/compare_rust.py "$temp/corpus.jsonl" "$temp/policy.json"
echo "[4/8] Conflict certificate (expected status code 3)"
rc=0
cargo run --locked --quiet -p nomos-cli -- conflict > "$temp/conflict.json" || rc="$?"
test "$rc" -eq 3
rc=0
cargo run --locked --quiet -p nomos-cli -- explain "$temp/conflict.json" || rc="$?"
test "$rc" -eq 3
echo "[5/8] Explicit model relaxations and Pareto frontier"
cargo run --locked --quiet -p nomos-cli -- relax examples/symmetric-single-token/world.nomos.yaml --show-all-minimal > "$temp/relaxations.json"
cargo run --locked --quiet -p nomos-cli -- frontier examples/water-allocation/world.nomos.yaml > "$temp/frontier.json"
echo "[6/8] Multi-resource trajectory and expected coupling failure"
cargo run --locked --quiet -p nomos-cli -- simulate examples/multi-resource/feasible.json > "$temp/trace.json"
rc=0
cargo run --locked --quiet -p nomos-cli -- simulate examples/multi-resource/coupling-counterexample.json > "$temp/counterexample.json" || rc="$?"
test "$rc" -eq 3
echo "[7/8] Hypothetical bounded response"
cargo run --locked --quiet -p nomos-cli -- behavior examples/behavior/need-inflation.json > "$temp/behavior.json"
echo "[8/8] Tampered policy rejection"
python3 -c 'import json,sys; p=sys.argv[1];j=json.load(open(p));j["world_count"]+=1;open(sys.argv[2],"w").write(json.dumps(j))' "$temp/policy.json" "$temp/forgery.json"
if cargo run --locked --quiet -p nomos-cli -- verify "$temp/forgery.json" --full-domain > /dev/null 2>&1; then
    echo "ERROR: forged policy evidence was accepted" >&2
    exit 1
fi
echo "REPRODUCED: 1,296-state policy, 20,736 differential cases, scoped impossibility and relaxations; no social validity claim."
