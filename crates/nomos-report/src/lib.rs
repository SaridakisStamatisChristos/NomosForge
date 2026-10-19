//! Exact bounded Pareto comparisons; no hidden weighted social-welfare scalar.
use nomos_execute::allocate;
use nomos_types::{Policy,World};
use nomos_verify::check_all;
use serde::{Deserialize,Serialize};
const AXIOMS:[&str;6]=["nonnegative_allocation","need_cap","conservation","nonwaste","critical_first","bounded_peer_disparity"];
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Metrics {
    pub total_allocated:u64,
    pub maximum_individual_shortfall:u32,
    pub maximum_peer_disparity:u32,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Point{pub policy:Policy,pub metrics:Metrics}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Sensitivity{pub removed_axiom:String,pub feasible_policies:usize}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct FrontierReport{
    pub status:String,
    pub verification_level:String,
    pub world_count:usize,
    pub grammar_size:usize,
    pub feasible_count:usize,
    pub points:Vec<Point>,
    pub frontier:Vec<Point>,
    pub one_axiom_relaxation:Vec<Sensitivity>,
    pub dominance_rule:String,
}
pub fn metrics(worlds:&[World],p:Policy)->Result<Metrics,String>{
    if check_all(worlds,p)?.is_some(){return Err("cannot report feasible metrics for violated requirements".into());}
    let mut total=0u64;let mut shortfall=0u32;let mut peers=0u32;
    for w in worlds{
        let a=allocate(w,p)?;
        total+=a.iter().map(|&v|u64::from(v)).sum::<u64>();
        for (need,given) in w.needs.iter().zip(&a){shortfall=shortfall.max(need-given);}
        for i in 0..a.len(){for j in (i+1)..a.len(){
            if w.needs[i]==w.needs[j]&&w.critical[i]==w.critical[j]{
                peers=peers.max(a[i].abs_diff(a[j]));
            }
        }}
    }
    Ok(Metrics{total_allocated:total,maximum_individual_shortfall:shortfall,maximum_peer_disparity:peers})
}
pub fn dominates(a:&Metrics,b:&Metrics)->bool{
    a.total_allocated>=b.total_allocated &&
    a.maximum_individual_shortfall<=b.maximum_individual_shortfall &&
    a.maximum_peer_disparity<=b.maximum_peer_disparity &&
    (a.total_allocated>b.total_allocated ||
      a.maximum_individual_shortfall<b.maximum_individual_shortfall ||
      a.maximum_peer_disparity<b.maximum_peer_disparity)
}
// All failures (not just the first failing axiom) for valid bounded allocations.
fn failed_axioms(w:&World,a:&[u32])->Vec<&'static str>{
    let mut failed=Vec::new();
    if a.len()!=w.needs.len(){return AXIOMS.to_vec();}
    if a.iter().zip(&w.needs).any(|(&x,&y)|x>y){failed.push("need_cap");}
    let total:u64=a.iter().map(|&x|u64::from(x)).sum();
    if total>u64::from(w.stock){failed.push("conservation");}
    if total!=u64::from(w.stock).min(w.demand()){failed.push("nonwaste");}
    if a.iter().enumerate().any(|(i,&v)|w.critical[i]==1&&v<w.needs[i]) &&
       a.iter().enumerate().any(|(i,&v)|w.critical[i]==0&&v>0){failed.push("critical_first");}
    for i in 0..a.len(){for j in (i+1)..a.len(){
        if w.critical[i]==w.critical[j]&&w.needs[i]==w.needs[j]&&a[i].abs_diff(a[j])>1{
            if !failed.contains(&"bounded_peer_disparity"){failed.push("bounded_peer_disparity");}
        }
    }}
    failed
}
pub fn analyze(worlds:&[World],programs:&[Policy])->Result<FrontierReport,String>{
    if worlds.is_empty()||programs.is_empty(){return Err("empty bounded world or grammar".into());}
    for w in worlds{w.validate()?;}
    let mut points=Vec::new();
    for &p in programs{
        if check_all(worlds,p)?.is_none(){points.push(Point{policy:p,metrics:metrics(worlds,p)?});}
    }
    let frontier=points.iter().filter(|p|
       !points.iter().any(|q|dominates(&q.metrics,&p.metrics))).cloned().collect();
    let mut one_axiom_relaxation=Vec::new();
    for dropped in AXIOMS{
        let mut feasible=0;
        'program:for &p in programs{
            for w in worlds{
                let a=allocate(w,p)?;
                if failed_axioms(w,&a).iter().any(|&ax|ax!=dropped){continue 'program;}
            }
            feasible+=1;
        }
        one_axiom_relaxation.push(Sensitivity{removed_axiom:dropped.into(),feasible_policies:feasible});
    }
    Ok(FrontierReport{
        status:if points.is_empty(){"INFEASIBLE_IN_GRAMMAR".into()}else{"PARETO_FRONTIER_WITHIN_FINITE_GRAMMAR".into()},
        verification_level:"EXHAUSTIVE_FINITE".into(),
        world_count:worlds.len(),grammar_size:programs.len(),feasible_count:points.len(),
        points,frontier,one_axiom_relaxation,
        dominance_rule:"maximize total_allocated; minimize maximum_individual_shortfall and maximum_peer_disparity; strict improvement on at least one".into()
    })
}
#[cfg(test)]
mod tests{
    use super::*;
    use nomos_grammar::{reference_worlds,reference_grammar};
    #[test]fn exact_bounded_frontier(){
        let r=analyze(&reference_worlds(),&reference_grammar()).unwrap();
        assert!(r.feasible_count>0);
        assert!(!r.frontier.is_empty());
        for p in &r.frontier{assert!(!r.points.iter().any(|q|dominates(&q.metrics,&p.metrics)));}
        assert_eq!(r,analyze(&reference_worlds(),&reference_grammar()).unwrap());
    }
    #[test]fn no_scalar_preference_from_equal_metrics(){
        let a=Metrics{total_allocated:3,maximum_individual_shortfall:1,maximum_peer_disparity:0};
        assert!(!dominates(&a,&a));
    }
}
