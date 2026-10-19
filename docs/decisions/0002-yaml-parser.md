# ADR 0002 — Replace retired serde_yaml with serde_norway

Status: Accepted for review (October 19, 2026)

## Context

NomosForge accepts strict, finite v0.1 YAML specifications using a Serde-derived typed schema. The existing `serde_yaml 0.9` package is unmaintained. Replacing the parser must not change the finite model interpretation, weaken unknown-field checks or proof-domain validation, or raise the declared Rust 1.82 MSRV.

## Decision

Use `serde_norway 0.9.42` through the workspace dependency. This is a source-compatible fork of the old `serde_yaml` family and declares Rust 1.71.1 as its minimum version. The application still deserializes directly into `Spec` followed by `Spec::validate`; only the upstream parsing dependency and direct call name change.

The `serde_yaml_ng 0.10` fork also preserves the API but depends on the original `unsafe-libyaml`, which the RustSec migration advice describes as unmaintained. `serde-saphyr 2.0` is actively developed but declares MSRV 1.89, incompatible with NomosForge's currently supported Rust 1.82. `serde_yml` is deprecated and has a RustSec unsoundness advisory for its earlier releases.

## Required evidence

- `cargo test --workspace --all-targets --locked` on stable and Rust 1.82.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all -- --check`.
- Full `bash scripts/reproduce.sh` plus the independent 20,736-case Rust/Python comparison.
- Fail-closed YAML regression inputs and reference-model typed-value equivalence checks.
- Cargo.lock no longer contains a `serde_yaml` package entry.

## Boundaries

This migration makes no claim of identical acceptance for every possible YAML document or immunity from parser vulnerabilities. Acceptable input remains the v0.1 subset enforced by `Spec::validate`. Retain optional parser fuzzing as a separate follow-on hardening activity. If any existing reference fixture or proof changes after migration, revert or stop and investigate before merging.
