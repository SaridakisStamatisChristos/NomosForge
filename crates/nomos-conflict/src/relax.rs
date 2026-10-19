//! Explicit catalog of three single-change model relaxations; no ethical ranking.
use crate::{check_certificate, ConflictCertificate};
use nomos_types::World;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictCase {
    pub schema: String,
    pub world: World,
    pub resource_unit: String,
    pub divisible: bool,
    pub axioms: Vec<String>,
}
impl ConflictCase {
    pub fn validate(&self) -> Result<(), String> {
        self.world.validate()?;
        if self.schema != "nomos/conflict-model/v0.1"
            || self.world.stock != 1
            || self.world.needs != [1, 1]
            || self.world.critical != [0, 0]
            || self.resource_unit != "token"
            || self.divisible
            || self.axioms != ["full_utilization", "deterministic_anonymity"]
        {
            return Err("only the explicitly modeled symmetric, indivisible one-token conflict is supported".into());
        }
        Ok(())
    }
}
pub fn parse_case(src: &str) -> Result<ConflictCase, String> {
    let case: ConflictCase = serde_yaml_ng::from_str(src).map_err(|e| e.to_string())?;
    case.validate()?;
    Ok(case)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub changed_assumption: String,
    pub allocation_denominator: u32,
    pub outcomes: Vec<Vec<u32>>,
    pub probabilities: Vec<Fraction>,
    pub satisfaction_claim: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelaxationReport {
    pub status: String,
    pub baseline: ConflictCertificate,
    pub scope: String,
    pub proposals: Vec<Proposal>,
}
pub fn propose(
    case: &ConflictCase,
    baseline: ConflictCertificate,
) -> Result<RelaxationReport, String> {
    case.validate()?;
    check_certificate(&baseline)?;
    let proposals = vec![
        Proposal {
            id: "allow_waste".into(),
            changed_assumption: "remove_full_utilization".into(),
            allocation_denominator: 1,
            outcomes: vec![vec![0, 0]],
            probabilities: vec![Fraction {
                numerator: 1,
                denominator: 1,
            }],
            satisfaction_claim:
                "deterministic integer anonymity and capacity; utilization deliberately abandoned"
                    .into(),
        },
        Proposal {
            id: "divisible".into(),
            changed_assumption: "allow_halved_token".into(),
            allocation_denominator: 2,
            outcomes: vec![vec![1, 1]],
            probabilities: vec![Fraction {
                numerator: 1,
                denominator: 1,
            }],
            satisfaction_claim:
                "deterministic equal halves with full utilization under changed divisibility".into(),
        },
        Proposal {
            id: "lottery".into(),
            changed_assumption: "replace_deterministic_anonymity_with_distributional_anonymity"
                .into(),
            allocation_denominator: 1,
            outcomes: vec![vec![1, 0], vec![0, 1]],
            probabilities: vec![
                Fraction {
                    numerator: 1,
                    denominator: 2,
                },
                Fraction {
                    numerator: 1,
                    denominator: 2,
                },
            ],
            satisfaction_claim:
                "symmetric distribution, each ex-post allocation fully utilizes one token".into(),
        },
    ];
    let report=RelaxationReport{status:"CATALOG_RELAXATIONS".into(),baseline,
        scope:"Three catalog-defined single-change revisions, not an exhaustive universe of ethical compromises.".into(),
        proposals};
    check_report(&report)?;
    Ok(report)
}
/// Recheck each explicit changed model using exact rational arithmetic on this tiny state.
pub fn check_report(report: &RelaxationReport) -> Result<(), String> {
    check_certificate(&report.baseline)?;
    if report.status!="CATALOG_RELAXATIONS" || report.proposals.len()!=3 ||
       report.scope!="Three catalog-defined single-change revisions, not an exhaustive universe of ethical compromises."{
        return Err("relaxation scope or catalog altered".into());
    }
    for (i, p) in report.proposals.iter().enumerate() {
        if p.outcomes.len() != p.probabilities.len()
            || p.outcomes.is_empty()
            || p.allocation_denominator == 0
        {
            return Err("bad rational or outcome arity".into());
        }
        for (a, f) in p.outcomes.iter().zip(&p.probabilities) {
            if a.len() != 2
                || f.denominator == 0
                || f.numerator == 0
                || f.numerator > f.denominator
                || a.iter().any(|&v| v > p.allocation_denominator)
                || a.iter().map(|&v| u64::from(v)).sum::<u64>()
                    > u64::from(p.allocation_denominator)
            {
                return Err("outcome violates changed resource model".into());
            }
        }
        let result=match i{
            0=>p.id=="allow_waste" && p.changed_assumption=="remove_full_utilization" &&
               p.allocation_denominator==1 && p.outcomes==[vec![0,0]] &&
               p.probabilities==[Fraction{numerator:1,denominator:1}] &&
               p.satisfaction_claim=="deterministic integer anonymity and capacity; utilization deliberately abandoned",
            1=>p.id=="divisible" && p.changed_assumption=="allow_halved_token" &&
               p.allocation_denominator==2 && p.outcomes==[vec![1,1]] &&
               p.probabilities==[Fraction{numerator:1,denominator:1}] &&
               p.satisfaction_claim=="deterministic equal halves with full utilization under changed divisibility",
            2=>p.id=="lottery" && p.changed_assumption=="replace_deterministic_anonymity_with_distributional_anonymity" &&
               p.allocation_denominator==1 && p.outcomes==[vec![1,0],vec![0,1]] &&
               p.probabilities==[Fraction{numerator:1,denominator:2},Fraction{numerator:1,denominator:2}] &&
               p.satisfaction_claim=="symmetric distribution, each ex-post allocation fully utilizes one token",
            _=>false
        };
        if !result {
            return Err(format!("unverified model transformation: {}", p.id));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::symmetric_single_token;
    fn case() -> ConflictCase {
        parse_case(include_str!(
            "../../../examples/symmetric-single-token/world.nomos.yaml"
        ))
        .unwrap()
    }
    #[test]
    fn all_three_rechecked() {
        let r = propose(&case(), symmetric_single_token()).unwrap();
        assert_eq!(r.proposals.len(), 3);
        assert!(check_report(&r).is_ok());
    }
    #[test]
    fn corrupt_lottery_rejected() {
        let mut r = propose(&case(), symmetric_single_token()).unwrap();
        r.proposals[2].probabilities[0].numerator = 2;
        assert!(check_report(&r).is_err());
    }
}
