//! Exhaustive finite-world verifier, deliberately independent from synthesis control flow.
use nomos_execute::{allocate, violation};
use nomos_types::{Policy, World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counterexample {
    pub world: World,
    pub observed_allocation: Vec<u32>,
    pub violated_axiom: String,
}
pub fn check_all(worlds: &[World], policy: Policy) -> Result<Option<Counterexample>, String> {
    if worlds.is_empty() {
        return Err("no states: empty domain is not a proof".into());
    }
    for w in worlds {
        let allocation = allocate(w, policy)?;
        if let Some(axiom) = violation(w, &allocation) {
            return Ok(Some(Counterexample {
                world: w.clone(),
                observed_allocation: allocation,
                violated_axiom: axiom.into(),
            }));
        }
    }
    Ok(None)
}
pub fn validate_counterexample(witness: &Counterexample, policy: Policy) -> Result<(), String> {
    let alloc = allocate(&witness.world, policy)?;
    if alloc != witness.observed_allocation {
        return Err("counterexample allocation does not replay".into());
    }
    match violation(&witness.world, &alloc) {
        Some(axiom) if axiom == witness.violated_axiom => Ok(()),
        _ => Err("counterexample axiom does not replay".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use nomos_types::Priority;
    #[test]
    fn witness_tampering_rejected() {
        let p = Policy {
            priority: Priority::Index,
            reserve: 0,
        };
        let worlds = vec![World {
            stock: 1,
            needs: vec![1, 1, 1],
            critical: vec![0, 0, 1],
        }];
        let mut c = check_all(&worlds, p).unwrap().unwrap();
        assert!(validate_counterexample(&c, p).is_ok());
        c.observed_allocation[0] = 0;
        assert!(validate_counterexample(&c, p).is_err());
    }
    #[test]
    fn no_empty_domain_proof() {
        let p = Policy {
            priority: Priority::Index,
            reserve: 0,
        };
        assert!(check_all(&[], p).is_err());
    }
}

pub mod transitions;
