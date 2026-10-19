#!/usr/bin/env bash
# Validate an isolated local installation and the documented finite-model journey.
set -euo pipefail
cd "$(dirname "$0")/.."
install_root="$(mktemp -d)"
trap 'rm -rf "$install_root"' EXIT

cargo install --locked --path crates/nomos-cli --root "$install_root"
nomos="$install_root/bin/nomos"
test -x "$nomos"

"$nomos" --help > "$install_root/help.txt"
grep -Fq "Usage: nomos <command>" "$install_root/help.txt"
"$nomos" --version > "$install_root/version.txt"
grep -Fq "nomos 0.1.0" "$install_root/version.txt"
"$nomos" validate examples/water-allocation/world.nomos.yaml > "$install_root/validate.txt"
grep -Fq "VALID: water-rationing (nomos/v0.1)" "$install_root/validate.txt"

"$nomos" synth examples/water-allocation/world.nomos.yaml --engine enumerate --json "$install_root/policy.json"
"$nomos" verify "$install_root/policy.json" --full-domain --model examples/water-allocation/world.nomos.yaml > "$install_root/verify.txt"
grep -Fq "CERTIFICATE_CHECKED: BOUNDED_VERIFIED_IN_FINITE_GRAMMAR" "$install_root/verify.txt"

# Bad arguments must fail, and scope-only infeasibility must keep its documented status.
if "$nomos" validate examples/water-allocation/world.nomos.yaml --misspelled > /dev/null 2>&1; then
  echo "ERROR: invalid command arguments unexpectedly succeeded" >&2
  exit 1
fi
set +e
"$nomos" conflict > "$install_root/conflict.json"
code="$?"
set -e
if [ "$code" -ne 3 ]; then
  echo "ERROR: expected conflict exit code 3, observed $code" >&2
  exit 1
fi
echo "INSTALL_SMOKE_VERIFIED: local cargo install, help/version, validation, synthesis, replay, and scoped conflict"
