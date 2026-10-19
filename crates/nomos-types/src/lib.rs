//! Bounded and explicit world and policy contracts; never claims social validity.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub stock: u32,
    pub needs: Vec<u32>,
    /// Encodes a declared critical flag, not an inferred human property.
    pub critical: Vec<u8>,
}
impl World {
    pub fn validate(&self) -> Result<(), String> {
        if self.needs.is_empty() || self.needs.len() != self.critical.len() {
            return Err("nonempty needs and critical must have equal lengths".into());
        }
        if self.needs.len() > 16 || self.stock > 10_000 || self.needs.iter().any(|n| *n > 10_000) {
            return Err("outside bounded execution limits".into());
        }
        if self.critical.iter().any(|c| *c > 1) {
            return Err("critical flags must be 0 or 1".into());
        }
        Ok(())
    }
    pub fn demand(&self) -> u64 {
        self.needs.iter().map(|&n| u64::from(n)).sum()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Index,
    Equal,
    NeedsDesc,
    NeedsAsc,
    CriticalIndex,
    CriticalEqual,
    CriticalNeedsDesc,
    CriticalNeedsAsc,
}
impl Priority {
    pub const ALL: [Priority; 8] = [
        Self::Index,
        Self::Equal,
        Self::NeedsDesc,
        Self::NeedsAsc,
        Self::CriticalIndex,
        Self::CriticalEqual,
        Self::CriticalNeedsDesc,
        Self::CriticalNeedsAsc,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub priority: Priority,
    pub reserve: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationLevel {
    BoundedVerified,
    InfeasibleInGrammar,
    ModelInfeasible,
    Unknown,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_flags_and_bounds() {
        assert!(World {
            stock: 1,
            needs: vec![1],
            critical: vec![1]
        }
        .validate()
        .is_ok());
        assert!(World {
            stock: 1,
            needs: vec![1],
            critical: vec![2]
        }
        .validate()
        .is_err());
        assert!(World {
            stock: 1,
            needs: vec![],
            critical: vec![]
        }
        .validate()
        .is_err());
    }
}
