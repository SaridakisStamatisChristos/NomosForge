//! Versioned, domain-separated SHA-256 bindings for finite synthesis evidence.
//! Digests are integrity/provenance identifiers, not signatures or independent proofs.
use nomos_parse::Spec;
use nomos_synth::{synthesize, SynthesisReport};
use nomos_types::{Policy, World};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const EVIDENCE_SCHEMA: &str = "nomos/synthesis-evidence/v1";
const SEMANTICS: &str = "nomos/finite-cegis/v0.1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema: String,
    pub semantics: String,
    pub producer_version: String,
    pub model_sha256: String,
    pub grammar_sha256: String,
    pub ordered_worlds_sha256: String,
    pub synthesis_report_sha256: String,
}

/// Preserve the v0.1 top-level report layout for Python differential comparisons.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundReport {
    #[serde(flatten)]
    pub report: SynthesisReport,
    pub evidence: Evidence,
}

fn fingerprint<T: Serialize>(domain: &[u8], value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(b"NomosForge evidence v1\0");
    hasher.update((domain.len() as u64).to_be_bytes());
    hasher.update(domain);
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn seal(
    spec: &Spec,
    worlds: &[World],
    grammar: &[Policy],
    report: SynthesisReport,
) -> Result<BoundReport, String> {
    spec.validate()?;
    let evidence = Evidence {
        schema: EVIDENCE_SCHEMA.into(),
        semantics: SEMANTICS.into(),
        producer_version: env!("CARGO_PKG_VERSION").into(),
        model_sha256: fingerprint(b"model", spec)?,
        grammar_sha256: fingerprint(b"grammar", &grammar)?,
        ordered_worlds_sha256: fingerprint(b"ordered-worlds", &worlds)?,
        synthesis_report_sha256: fingerprint(b"synthesis-report", &report)?,
    };
    Ok(BoundReport { report, evidence })
}

/// Check both the fingerprints and the actual synthesizer result. Digest equality alone
/// is never accepted as proof of bounded feasibility.
pub fn check_binding(
    spec: &Spec,
    worlds: &[World],
    grammar: &[Policy],
    supplied: &BoundReport,
) -> Result<(), String> {
    let recomputed = synthesize(worlds, grammar, None)?;
    if supplied.report != recomputed {
        return Err("reported policy, transcript, status or domain differs from recomputation".into());
    }
    let expected = seal(spec, worlds, grammar, recomputed)?;
    if supplied.evidence != expected.evidence {
        return Err("evidence provenance mismatch (model, grammar, domain, report or version)".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomos_grammar::{grammar, worlds};

    fn fixture() -> (Spec, Vec<World>, Vec<Policy>) {
        let spec = nomos_parse::parse(include_str!(
            "../../../examples/water-allocation/world.nomos.yaml"
        ))
        .unwrap();
        let flags: Vec<u8> = spec
            .verification
            .critical
            .iter()
            .map(|&f| u8::from(f))
            .collect();
        let states = worlds(
            spec.model.people.count,
            spec.verification.stock.min,
            spec.verification.stock.max,
            spec.verification.need.min,
            spec.verification.need.max,
            &flags,
        )
        .unwrap();
        let programs = grammar(
            &spec.policies.grammar.priority,
            &spec.policies.grammar.reserve,
        )
        .unwrap();
        (spec, states, programs)
    }

    #[test]
    fn bound_reference_replays_and_has_stable_digest_lengths() {
        let (spec, states, programs) = fixture();
        let result = synthesize(&states, &programs, None).unwrap();
        let sealed = seal(&spec, &states, &programs, result).unwrap();
        assert_eq!(sealed.evidence.model_sha256.len(), 64);
        assert_eq!(sealed.evidence.grammar_sha256.len(), 64);
        assert_eq!(sealed.evidence.ordered_worlds_sha256.len(), 64);
        assert_eq!(sealed.evidence.synthesis_report_sha256.len(), 64);
        let json = serde_json::to_string(&sealed).unwrap();
        let from_json: BoundReport = serde_json::from_str(&json).unwrap();
        assert_eq!(from_json, sealed);
        assert!(check_binding(&spec, &states, &programs, &from_json).is_ok());
    }

    #[test]
    fn model_grammar_report_and_version_tampering_fail_closed() {
        let (spec, states, programs) = fixture();
        let result = synthesize(&states, &programs, None).unwrap();
        let sealed = seal(&spec, &states, &programs, result).unwrap();

        let mut renamed = spec.clone();
        renamed.model.id = "different-model".into();
        assert!(check_binding(&renamed, &states, &programs, &sealed).is_err());

        let mut reordered = programs.clone();
        reordered.swap(0, 1);
        assert!(check_binding(&spec, &states, &reordered, &sealed).is_err());

        let mut altered = sealed.clone();
        altered.report.world_count += 1;
        assert!(check_binding(&spec, &states, &programs, &altered).is_err());

        let mut altered = sealed.clone();
        altered.evidence.producer_version = "0.0.0".into();
        assert!(check_binding(&spec, &states, &programs, &altered).is_err());

        let mut altered = sealed.clone();
        altered.evidence.model_sha256.replace_range(0..1, "0");
        assert!(
            altered == sealed
                || check_binding(&spec, &states, &programs, &altered).is_err()
        );

        let mut reordered_states = states.clone();
        reordered_states.swap(0, 1);
        assert!(check_binding(&spec, &reordered_states, &programs, &sealed).is_err());
    }

    #[test]
    fn legacy_unbound_reports_are_not_accepted() {
        let (spec, states, programs) = fixture();
        let result = synthesize(&states, &programs, None).unwrap();
        let json = serde_json::to_string(&result).unwrap();
        assert!(serde_json::from_str::<BoundReport>(&json).is_err());
        // Human-facing field order and whitespace do not influence typed fingerprints.
        let report = seal(&spec, &states, &programs, result).unwrap();
        let formatted: BoundReport =
            serde_json::from_str(&serde_json::to_string_pretty(&report).unwrap()).unwrap();
        assert!(check_binding(&spec, &states, &programs, &formatted).is_ok());
    }
}
