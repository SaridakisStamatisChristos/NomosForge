//! Explicit finite *hypothetical* reporting strategies; not a behavioral prediction.
use nomos_execute::allocate;
use nomos_types::{Policy,World};
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case")]
pub enum Behavior{
    Truthful,
    Inflate{extra:u32},
    Underreport{less:u32}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorModel{
    pub schema:String,
    pub world:World,
    pub policy:Policy,
    /// Explicit alternative reporting strategies available to each modeled agent.
    pub choices:Vec<Vec<Behavior>>,
    pub provenance:String
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Observation{
    pub strategy_profile:Vec<Behavior>,
    pub reported_needs:Vec<u32>,
    pub allocation:Vec<u32>,
    pub modeled_unmet_true_needs:u64,
    pub allocation_above_true_need:u64,
    pub difference_from_truthful:u64,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct BehaviorReport{
    pub status:String,
    pub assumption_scope:String,
    pub provenance:String,
    pub observations:Vec<Observation>,
    pub worst_modeled_unmet_need:u64,
}
impl BehaviorModel{
    pub fn validate(&self)->Result<(),String>{
        self.world.validate()?;
        if self.schema!="nomos/behavior/v0.1" || self.world.needs.len()>4 ||
           self.provenance.trim().is_empty() || self.choices.len()!=self.world.needs.len() ||
           self.choices.iter().any(|c|c.is_empty()||c.len()>4||!c.contains(&Behavior::Truthful)){
            return Err("invalid bounded hypothetical strategy catalog or missing provenance".into());
        }
        let combos=self.choices.iter().try_fold(1usize,|acc,c|acc.checked_mul(c.len())).ok_or("strategy cardinality overflow")?;
        if combos>256{return Err("more than 256 hypothetical strategy profiles".into());}
        Ok(())
    }
}
pub fn enumerate(model:&BehaviorModel)->Result<BehaviorReport,String>{
    model.validate()?;
    let baseline=allocate(&model.world,model.policy)?;
    let mut profiles:Vec<Vec<Behavior>>=vec![Vec::new()];
    for choices in &model.choices{
        profiles=profiles.into_iter().flat_map(|profile|
            choices.iter().map(move |item|{
                let mut next=profile.clone();next.push(item.clone());next
            })).collect();
    }
    let mut observations=Vec::new();
    for profile in profiles{
        let mut reported=Vec::new();
        for (need,behavior) in model.world.needs.iter().zip(&profile){
            let r=match behavior{
                Behavior::Truthful=>*need,
                Behavior::Inflate{extra}=>need.checked_add(*extra).ok_or("reported need integer overflow")?,
                Behavior::Underreport{less}=>need.saturating_sub(*less)
            };
            if r>10_000{return Err("reported need outside declared world bounds".into());}
            reported.push(r);
        }
        let altered=World{stock:model.world.stock,needs:reported.clone(),critical:model.world.critical.clone()};
        let allocation=allocate(&altered,model.policy)?;
        let shortfall=model.world.needs.iter().zip(&allocation)
            .map(|(&n,&a)|u64::from(n.saturating_sub(a))).sum();
        let over=model.world.needs.iter().zip(&allocation)
            .map(|(&n,&a)|u64::from(a.saturating_sub(n))).sum();
        let change=baseline.iter().zip(&allocation).map(|(&x,&y)|u64::from(x.abs_diff(y))).sum();
        observations.push(Observation{strategy_profile:profile,reported_needs:reported,allocation,
           modeled_unmet_true_needs:shortfall,allocation_above_true_need:over,difference_from_truthful:change});
    }
    let worst=observations.iter().map(|o|o.modeled_unmet_true_needs).max().unwrap_or(0);
    Ok(BehaviorReport{status:"HYPOTHETICAL_BEHAVIOR_ENUMERATED".into(),
        assumption_scope:"finite, user-supplied hypothetical need-reporting strategies only; no human calibration".into(),
        provenance:model.provenance.clone(),observations,worst_modeled_unmet_need:worst})
}
pub fn check_report(model:&BehaviorModel,report:&BehaviorReport)->Result<(),String>{
    model.validate()?;
    if report.status!="HYPOTHETICAL_BEHAVIOR_ENUMERATED" ||
       report.assumption_scope!="finite, user-supplied hypothetical need-reporting strategies only; no human calibration" ||
       &enumerate(model)?!=report {
        return Err("hypothetical behavior report does not replay".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    use nomos_types::Priority;
    fn scenario()->BehaviorModel{
        BehaviorModel{schema:"nomos/behavior/v0.1".into(),
            world:World{stock:3,needs:vec![1,1],critical:vec![0,0]},
            policy:Policy{priority:Priority::Equal,reserve:0},
            choices:vec![vec![Behavior::Truthful,Behavior::Inflate{extra:1}],
               vec![Behavior::Truthful]],
            provenance:"synthetic strategy stress test; no empirical calibration".into()}
    }
    #[test]fn hypothetical_inflation_changes_model(){
        let m=scenario();let r=enumerate(&m).unwrap();
        assert_eq!(r.observations.len(),2);
        assert_ne!(r.observations[0].allocation,r.observations[1].allocation);
        assert!(check_report(&m,&r).is_ok());
        let mut bad=r.clone();bad.observations[1].reported_needs[0]=999;
        assert!(check_report(&m,&bad).is_err());
    }
    #[test]fn requires_truthful_baseline(){
        let mut m=scenario();m.choices[0]=vec![Behavior::Inflate{extra:1}];
        assert!(enumerate(&m).is_err());
    }
}
