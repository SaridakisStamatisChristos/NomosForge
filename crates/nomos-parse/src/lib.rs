//! Strict typed YAML/JSON front end. Cannot silently change units or proof domain.
use nomos_types::Priority;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub min: u32,
    pub max: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub id: String,
    pub people: People,
    pub resources: Vec<Resource>,
    pub horizon: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct People {
    pub count: usize,
    pub attributes: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub name: String,
    pub unit: String,
    pub divisible: bool,
    pub max_stock: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub engine: String,
    pub stock: Bounds,
    pub need: Bounds,
    pub critical: Vec<bool>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Grammar {
    pub priority: Vec<Priority>,
    pub reserve: Vec<u32>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    pub grammar: Grammar,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Axioms {
    pub hard: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Objectives {
    pub report: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub schema: String,
    pub model: Model,
    pub verification: Verification,
    pub policies: Policies,
    pub axioms: Axioms,
    pub objectives: Objectives,
}
impl Spec {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "nomos/v0.1"
            || self.model.horizon != 1
            || self.verification.engine != "exhaustive"
        {
            return Err("unsupported schema, horizon or engine".into());
        }
        if self.model.people.count != 3
            || self.model.people.attributes != ["need:int", "critical:bool"]
        {
            return Err("v0.1 requires exactly 3 agents with need:int and critical:bool".into());
        }
        if self.model.resources.len() != 1
            || self.model.resources[0].divisible
            || self.model.resources[0].unit != "token"
            || self.model.resources[0].name.trim().is_empty()
        {
            return Err("v0.1 requires one indivisible token resource".into());
        }
        if self.verification.stock.min > self.verification.stock.max
            || self.verification.need.min > self.verification.need.max
            || self.verification.stock.max > self.model.resources[0].max_stock
        {
            return Err("invalid finite bounds".into());
        }
        if self.verification.critical.is_empty()
            || self.policies.grammar.priority.is_empty()
            || self.policies.grammar.reserve.is_empty()
        {
            return Err("empty verification domain or grammar".into());
        }
        const HARD: [&str; 5] = [
            "nonnegative_allocation",
            "need_cap",
            "conservation",
            "nonwaste",
            "critical_first",
        ];
        for ax in &self.axioms.hard {
            if !HARD.contains(&ax.as_str()) && ax != "bounded_peer_disparity" {
                return Err(format!("unsupported axiom: {ax}"));
            }
        }
        // v0.1 kernel checks all six checks: omission must not silently weaken requirements.
        if ![
            "nonnegative_allocation",
            "need_cap",
            "conservation",
            "nonwaste",
            "critical_first",
            "bounded_peer_disparity",
        ]
        .iter()
        .all(|ax| self.axioms.hard.iter().any(|x| x == ax))
        {
            return Err("v0.1 requires all six explicitly listed hard checks".into());
        }
        Ok(())
    }
}
pub fn parse(input: &str) -> Result<Spec, String> {
    let parsed: Spec = serde_yaml::from_str(input).map_err(|e| e.to_string())?;
    parsed.validate()?;
    Ok(parsed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_or_invalid_units() {
        let s = include_str!("../../../examples/water-allocation/world.nomos.yaml");
        assert!(parse(s).is_ok());
        assert!(parse(&s.replace("unit: token", "unit: litre")).is_err());
        assert!(parse(&s.replace("divisible: false", "divisible: true")).is_err());
    }
}
