//! All-outcomes enumeration for a *single* symmetric, indivisible-token world.
use nomos_types::World;
use serde::{Deserialize, Serialize};

pub const AXIOMS: [&str; 3] = ["capacity", "full_utilization", "deterministic_anonymity"];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeRow {
    pub allocation: Vec<u32>,
    pub capacity: bool,
    pub full_utilization: bool,
    pub deterministic_anonymity: bool,
    pub all: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictCertificate {
    pub schema: String,
    pub status: String,
    pub world: World,
    pub axioms: Vec<String>,
    pub possible_allocations: Vec<OutcomeRow>,
    pub satisfying_allocations: usize,
    pub minimal_conflict_cores: Vec<Vec<String>>,
    pub claim_scope: String,
}
fn row_for(a: u32, b: u32) -> OutcomeRow {
    let capacity = a + b <= 1;
    let full_utilization = a + b == 1;
    let deterministic_anonymity = a == b;
    OutcomeRow {
        allocation: vec![a, b],
        capacity,
        full_utilization,
        deterministic_anonymity,
        all: capacity && full_utilization && deterministic_anonymity,
    }
}
fn satisfies(row: &OutcomeRow, mask: u8) -> bool {
    (mask & 1 == 0 || row.capacity)
        && (mask & 2 == 0 || row.full_utilization)
        && (mask & 4 == 0 || row.deterministic_anonymity)
}
fn cores(rows: &[OutcomeRow]) -> Vec<Vec<String>> {
    let unsat = |mask: u8| !rows.iter().any(|r| satisfies(r, mask));
    let mut out = Vec::new();
    for size in 1..=3 {
        for mask in 1u8..8 {
            if mask.count_ones() != size || !unsat(mask) {
                continue;
            }
            if (0u8..8).any(|sub| sub != mask && sub & mask == sub && unsat(sub)) {
                continue;
            }
            out.push(
                (0..3)
                    .filter(|&i| mask & (1 << i) != 0)
                    .map(|i| AXIOMS[i].to_string())
                    .collect(),
            );
        }
    }
    out
}
pub fn symmetric_single_token() -> ConflictCertificate {
    let world = World {
        stock: 1,
        needs: vec![1, 1],
        critical: vec![0, 0],
    };
    let mut rows = Vec::new();
    for a in 0..=1 {
        for b in 0..=1 {
            rows.push(row_for(a, b));
        }
    }
    ConflictCertificate{
        schema:"nomos/conflict/v0.1".into(),status:"MODEL_INFEASIBLE".into(),
        world,axioms:AXIOMS.iter().map(|s|s.to_string()).collect(),
        satisfying_allocations:rows.iter().filter(|r|r.all).count(),
        minimal_conflict_cores:cores(&rows),possible_allocations:rows,
        claim_scope:"Only the symmetric two-agent, single indivisible-token state; no grammar-wide or social conclusion.".into()
    }
}
/// Independent checker: recomputes outcomes from the declared integer universe,
/// verifies every field and checks subset-minimality without calling the producer.
pub fn check_certificate(cert: &ConflictCertificate) -> Result<(), String> {
    let w = &cert.world;
    w.validate()?;
    if w.stock != 1
        || w.needs != [1, 1]
        || w.critical != [0, 0]
        || cert.schema != "nomos/conflict/v0.1"
        || cert.status != "MODEL_INFEASIBLE"
        || cert.axioms != AXIOMS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    {
        return Err("unsupported or altered certificate assumptions".into());
    }
    if cert.possible_allocations.len() != 4 {
        return Err("outcome coverage incomplete".into());
    }
    let mut independent = Vec::new();
    for a in [0u32, 1] {
        for b in [0u32, 1] {
            let within = a + b <= w.stock;
            let utilization = a + b == w.stock;
            let anonymity = a == b; // swapping the identical inputs leaves the state fixed
            independent.push(OutcomeRow {
                allocation: vec![a, b],
                capacity: within,
                full_utilization: utilization,
                deterministic_anonymity: anonymity,
                all: within && utilization && anonymity,
            });
        }
    }
    if cert.possible_allocations != independent {
        return Err("outcomes do not independently replay".into());
    }
    if cert.satisfying_allocations != independent.iter().filter(|r| r.all).count()
        || cert.satisfying_allocations != 0
    {
        return Err("false satisfiability claim".into());
    }
    // Verify the reported minimal core by trying all strict subsets over the full outcome universe.
    let mut core_masks = Vec::new();
    for names in &cert.minimal_conflict_cores {
        let mut mask = 0u8;
        for name in names {
            let i = AXIOMS
                .iter()
                .position(|a| *a == name)
                .ok_or("unknown axiom")?;
            if mask & (1 << i) != 0 {
                return Err("duplicate core axiom".into());
            }
            mask |= 1 << i;
        }
        core_masks.push(mask);
    }
    if core_masks != vec![6] {
        return Err("incorrect or nonminimal conflict core".into());
    }
    let subset_satisfies = |m: u8| {
        independent.iter().any(|r| {
            (m & 1 == 0 || r.capacity)
                && (m & 2 == 0 || r.full_utilization)
                && (m & 4 == 0 || r.deterministic_anonymity)
        })
    };
    if subset_satisfies(6) || !subset_satisfies(2) || !subset_satisfies(4) {
        return Err("reported conflict is not subset-minimal".into());
    }
    if cert.claim_scope!="Only the symmetric two-agent, single indivisible-token state; no grammar-wide or social conclusion." {
        return Err("certificate claim scope altered".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_minimal_conflict() {
        let c = symmetric_single_token();
        assert_eq!(c.satisfying_allocations, 0);
        assert_eq!(
            c.minimal_conflict_cores,
            vec![vec![
                "full_utilization".to_string(),
                "deterministic_anonymity".to_string()
            ]]
        );
        assert!(check_certificate(&c).is_ok());
    }
    #[test]
    fn tamper_attacks() {
        let c = symmetric_single_token();
        let mut m = c.clone();
        m.possible_allocations[0].allocation[0] = 1;
        assert!(check_certificate(&m).is_err());
        let mut m = c.clone();
        m.minimal_conflict_cores[0].remove(0);
        assert!(check_certificate(&m).is_err());
        let mut m = c.clone();
        m.world.stock = 2;
        assert!(check_certificate(&m).is_err());
        let mut m = c.clone();
        m.claim_scope = "all societies".into();
        assert!(check_certificate(&m).is_err());
    }
}
