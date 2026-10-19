//! Deterministic malformed-input smoke coverage; not a sustained fuzz campaign.
use nomos_parse::parse;

#[test]
fn finite_model_parser_never_panics_under_seed_mutations() {
    let seed = include_str!("../../../../examples/water-allocation/world.nomos.yaml");
    assert!(seed.is_ascii(), "mutation corpus is byte-indexed ASCII");

    // Removing one ASCII character at every position exercises truncated
    // tags, keys, scalar boundaries, list punctuation and YAML structure.
    for offset in 0..seed.len() {
        let mut mutated = seed.to_string();
        mutated.remove(offset);
        assert!(
            std::panic::catch_unwind(|| parse(&mutated)).is_ok(),
            "parser panicked after deletion at byte {offset}"
        );
    }

    // Exercise punctuation, control characters, aliases and token boundaries
    // with a deterministic set of reproducible malformed inputs.
    for offset in (0..=seed.len()).step_by(11) {
        for inserted in ["{", "}", "[", "]", ":", "\t", "\0", "&x ", "*x ", "\n"] {
            let mut mutated = seed.to_string();
            mutated.insert_str(offset, inserted);
            assert!(
                std::panic::catch_unwind(|| parse(&mutated)).is_ok(),
                "parser panicked after insertion at byte {offset}: {inserted:?}"
            );
        }
    }
}
