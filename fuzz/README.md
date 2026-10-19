# Parser fuzz target

Install cargo-fuzz and use `cargo +nightly fuzz run nomos_yaml` from the repository root when a compatible nightly toolchain is available. This target does not run in baseline CI and must not be claimed executed without an observed invocation. Input is arbitrary bytes; valid UTF-8 is passed to the strict YAML model parser. Runtime and finding evidence must be reported separately.
