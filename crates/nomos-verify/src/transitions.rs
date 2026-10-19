//! Finite explicit multi-resource trajectories, not universal temporal verification.
use nomos_execute::{allocate,violation};
use nomos_types::{Policy,World};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub name:String,
    pub stock:u32,
    pub max_stock:u32,
    pub needs:Vec<u32>,
    pub critical:Vec<u8>,
    pub policy:Policy,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShockStep {
    pub replenishment:Vec<u32>,
    pub shock_loss:Vec<u32>
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionModel {
    pub schema:String,
    pub unit:String,
    pub agents:usize,
    pub horizon:usize,
    pub resources:Vec<Resource>,
    pub coupling_per_agent:Option<Vec<u32>>,
    pub steps:Vec<ShockStep>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Frame {
    pub index:usize,
    pub stock_before:Vec<u32>,
    pub replenishment:Vec<u32>,
    pub shock_loss:Vec<u32>,
    pub allocations:Vec<Vec<u32>>,
    pub stock_after:Vec<u32>,
    pub violation:Option<String>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Trace {
    pub status:String,
    pub verification_scope:String,
    pub frames:Vec<Frame>,
}
impl TransitionModel{
    pub fn validate(&self)->Result<(),String>{
        if self.schema!="nomos/transition/v0.1"||self.unit!="token" ||
           self.agents==0||self.agents>6 ||
           self.horizon==0||self.horizon>16||self.horizon!=self.steps.len()||
           self.resources.is_empty()||self.resources.len()>4 {
            return Err("unsupported transition schema or finite bounds".into());
        }
        if let Some(caps)=&self.coupling_per_agent {
            if caps.len()!=self.agents || caps.iter().any(|&x|x>40_000){
                return Err("invalid coupling units or caps".into());
            }
        }
        for (i,r) in self.resources.iter().enumerate(){
            if r.name.trim().is_empty()||self.resources[..i].iter().any(|x|x.name==r.name) ||
               r.stock>r.max_stock||r.max_stock>10_000||
               r.needs.len()!=self.agents||r.critical.len()!=self.agents {
                return Err("invalid or duplicate resource channel".into());
            }
            World{stock:r.stock,needs:r.needs.clone(),critical:r.critical.clone()}.validate()?;
        }
        for step in &self.steps{
            if step.replenishment.len()!=self.resources.len()||
               step.shock_loss.len()!=self.resources.len(){
                return Err("resource/shock array length mismatch".into());
            }
        }
        Ok(())
    }
}
pub fn simulate(model:&TransitionModel)->Result<Trace,String>{
    model.validate()?;
    let mut stocks:Vec<u32>=model.resources.iter().map(|r|r.stock).collect();
    let mut frames=Vec::new();
    for (index,step) in model.steps.iter().enumerate(){
        let opening=stocks.clone();
        let mut allocations=Vec::new();
        let mut issue:Option<String>=None;
        for (i,r) in model.resources.iter().enumerate(){
            let incoming=stocks[i].checked_add(step.replenishment[i]).ok_or("stock overflow")?;
            if incoming>r.max_stock || step.shock_loss[i]>incoming{
                return Err(format!("invalid loss or cap at step {index}, resource {}",r.name));
            }
            let available=incoming-step.shock_loss[i];
            let w=World{stock:available,needs:r.needs.clone(),critical:r.critical.clone()};
            let a=allocate(&w,r.policy)?;
            if let Some(ax)=violation(&w,&a){
                if issue.is_none(){issue=Some(format!("resource {}: {ax}",r.name));}
            }
            let spent:u32=a.iter().copied().sum();
            stocks[i]=available.checked_sub(spent).ok_or("conservation violated")?;
            allocations.push(a);
        }
        if let Some(caps)=&model.coupling_per_agent{
            for (citizen,&cap) in caps.iter().enumerate(){
                let spent:u64=allocations.iter().map(|a|u64::from(a[citizen])).sum();
                if spent>u64::from(cap) && issue.is_none(){
                    issue=Some(format!("coupling cap violated for agent {citizen}"));
                }
            }
        }
        let fail=issue.is_some();
        frames.push(Frame{index,stock_before:opening,
            replenishment:step.replenishment.clone(),
            shock_loss:step.shock_loss.clone(),allocations,
            stock_after:stocks.clone(),violation:issue});
        if fail {return Ok(Trace{status:"COUNTEREXAMPLE".into(),
            verification_scope:"exact supplied bounded trajectory only".into(),frames});}
    }
    Ok(Trace{status:"BOUNDED_TRACE_CHECKED".into(),
        verification_scope:"exact supplied bounded trajectory only".into(),frames})
}
pub fn check_trace(model:&TransitionModel,trace:&Trace)->Result<(),String>{
    model.validate()?;
    if trace.verification_scope!="exact supplied bounded trajectory only" ||
       trace.frames.is_empty() || trace.frames.len()>model.horizon{
        return Err("trace scope or frame count invalid".into());
    }
    for (i,frame) in trace.frames.iter().enumerate(){
        if frame.index!=i ||
           frame.stock_before.len()!=model.resources.len() ||
           frame.stock_after.len()!=model.resources.len() ||
           frame.allocations.len()!=model.resources.len(){
            return Err("malformed trace frame".into());
        }
        for j in 0..model.resources.len(){
            let incoming=u64::from(frame.stock_before[j])+u64::from(frame.replenishment[j]);
            let used:u64=frame.allocations[j].iter().map(|&x|u64::from(x)).sum();
            if incoming < u64::from(frame.shock_loss[j])+used+u64::from(frame.stock_after[j]){
                return Err("created stock in transition certificate".into());
            }
            if incoming!=u64::from(frame.shock_loss[j])+used+u64::from(frame.stock_after[j]){
                return Err("lost stock in transition certificate".into());
            }
        }
    }
    if &simulate(model)?!=trace {return Err("transition replay does not match recorded trace".into());}
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use nomos_types::Priority;
    fn model(cap:u32)->TransitionModel{
        let resource=|name:&str|Resource{name:name.into(),stock:2,max_stock:3,
            needs:vec![1,1],critical:vec![0,0],
            policy:Policy{priority:Priority::CriticalEqual,reserve:0}};
        TransitionModel{schema:"nomos/transition/v0.1".into(),unit:"token".into(),
            agents:2,horizon:2,resources:vec![resource("water"),resource("food")],
            coupling_per_agent:Some(vec![cap,cap]),steps:vec![
                ShockStep{replenishment:vec![0,0],shock_loss:vec![0,0]},
                ShockStep{replenishment:vec![2,2],shock_loss:vec![0,0]}
            ]}
    }
    #[test]fn conservation_two_steps(){
        let m=model(2);let report=simulate(&m).unwrap();
        assert_eq!(report.status,"BOUNDED_TRACE_CHECKED");
        assert_eq!(report.frames.len(),2);
        assert!(check_trace(&m,&report).is_ok());
        let mut altered=report.clone();altered.frames[0].stock_after[0]+=1;
        assert!(check_trace(&m,&altered).is_err());
    }
    #[test]fn coupled_counterexample(){
        let m=model(1);let report=simulate(&m).unwrap();
        assert_eq!(report.status,"COUNTEREXAMPLE");
        assert_eq!(report.frames.len(),1);
        assert!(check_trace(&m,&report).is_ok());
    }
    #[test]fn invalid_shock_rejected(){
        let mut m=model(2);m.steps[0].shock_loss[0]=3;
        assert!(simulate(&m).is_err());
    }
}
