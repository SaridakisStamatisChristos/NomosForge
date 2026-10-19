//! Deterministic finite counterexample-guided inductive synthesis (CEGIS).
use nomos_execute::{allocate,violation};
use nomos_types::{Policy,World};
use nomos_verify::{check_all,Counterexample};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct FailedCandidate {
    pub candidate: Policy,
    pub counterexample: World,
    pub observed_allocation: Vec<u32>,
    pub violated_axiom: String,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct SynthesisReport {
    pub status: String,
    pub grammar_size: usize,
    pub world_count: usize,
    #[serde(skip_serializing_if="Option::is_none")]
    pub policy: Option<Policy>,
    pub transcript: Vec<FailedCandidate>,
    pub witnesses: Vec<World>,
    pub verification_level: String,
    pub witness_order: String,
}
pub fn synthesize(worlds:&[World],programs:&[Policy],max_challenges:Option<usize>) -> Result<SynthesisReport,String> {
    if worlds.is_empty() || programs.is_empty() {
        return Err("empty world domain or candidate grammar is not a sound certificate".into());
    }
    for w in worlds { w.validate()?; }
    let mut witnesses:Vec<World>=Vec::new();
    let mut transcript=Vec::new();
    let make_report=|status:&str,policy:Option<Policy>,transcript:Vec<FailedCandidate>,witnesses:Vec<World>| {
        SynthesisReport {
            status:status.into(),grammar_size:programs.len(),world_count:worlds.len(),
            policy,transcript,witnesses,verification_level:"EXHAUSTIVE_FINITE".into(),
            witness_order:"first_failure_in_supplied_domain_order".into()
        }
    };
    loop {
        let candidate=programs.iter().copied().find(|&p| {
            witnesses.iter().all(|w| {
                allocate(w,p).ok().and_then(|a|violation(w,&a)).is_none()
            })
        });
        let Some(p)=candidate else {
            return Ok(make_report("UNREALIZABLE_IN_FINITE_GRAMMAR",None,transcript,witnesses));
        };
        if max_challenges.is_some_and(|limit| transcript.len()>=limit) {
            return Ok(make_report("UNKNOWN",None,transcript,witnesses));
        }
        match check_all(worlds,p)? {
            None => return Ok(make_report("BOUNDED_VERIFIED_IN_FINITE_GRAMMAR",Some(p),transcript,witnesses)),
            Some(Counterexample{world,observed_allocation,violated_axiom}) => {
                transcript.push(FailedCandidate{
                    candidate:p, counterexample:world.clone(),observed_allocation,
                    violated_axiom,
                });
                // Every new witness eliminates at least the challenged candidate.
                if witnesses.contains(&world) { return Err("internal CEGIS no-progress error".into()); }
                witnesses.push(world);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use nomos_grammar::{reference_grammar,reference_worlds};
    use nomos_types::Priority;
    #[test] fn exact_reference_synthesis() {
        let result=synthesize(&reference_worlds(),&reference_grammar(),None).unwrap();
        assert_eq!(result.status,"BOUNDED_VERIFIED_IN_FINITE_GRAMMAR");
        assert_eq!(result.policy,Some(Policy{priority:Priority::CriticalEqual,reserve:0}));
        assert_eq!(result.transcript.len(),2);
        assert_eq!(result.transcript[0].violated_axiom,"critical_first");
        assert_eq!(result.transcript[1].violated_axiom,"bounded_peer_disparity");
    }
    #[test] fn grammar_fail_is_not_model_impossibility() {
        let r=synthesize(&reference_worlds(),&[Policy{priority:Priority::Index,reserve:0}],None).unwrap();
        assert_eq!(r.status,"UNREALIZABLE_IN_FINITE_GRAMMAR");
        assert!(r.policy.is_none());
    }
    #[test] fn bounded_budget_is_unknown() {
        let r=synthesize(&reference_worlds(),&reference_grammar(),Some(0)).unwrap();
        assert_eq!(r.status,"UNKNOWN");
    }
    #[test] fn deterministic_replay() {
        let worlds=reference_worlds();let programs=reference_grammar();
        assert_eq!(synthesize(&worlds,&programs,None),synthesize(&worlds,&programs,None));
    }
}
