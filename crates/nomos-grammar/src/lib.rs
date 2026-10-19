//! Finite grammar and deterministic bounded world enumeration.
use nomos_types::{Policy, Priority, World};
pub fn grammar(priorities: &[Priority], reserves: &[u32]) -> Result<Vec<Policy>, String> {
    if priorities.is_empty()
        || reserves.is_empty()
        || priorities
            .len()
            .checked_mul(reserves.len())
            .is_none_or(|n| n > 512)
    {
        return Err("grammar must be nonempty and contain at most 512 programs".into());
    }
    if priorities
        .iter()
        .enumerate()
        .any(|(i, p)| priorities[..i].contains(p))
        || reserves
            .iter()
            .enumerate()
            .any(|(i, r)| reserves[..i].contains(r))
    {
        return Err("duplicate priority or reserve in candidate grammar".into());
    }
    if reserves.iter().any(|&r| r > 10_000) {
        return Err("reserve outside declared policy bounds".into());
    }
    Ok(reserves
        .iter()
        .flat_map(|&reserve| {
            priorities
                .iter()
                .map(move |&priority| Policy { priority, reserve })
        })
        .collect())
}
pub fn reference_grammar() -> Vec<Policy> {
    grammar(&Priority::ALL, &[0, 1]).expect("reference grammar is valid")
}
fn vectors<T: Clone>(choices: &[T], n: usize) -> Vec<Vec<T>> {
    let mut accum: Vec<Vec<T>> = vec![vec![]];
    for _ in 0..n {
        accum = accum
            .into_iter()
            .flat_map(|prefix| {
                choices.iter().map(move |v| {
                    let mut x = prefix.clone();
                    x.push(v.clone());
                    x
                })
            })
            .collect();
    }
    accum
}
/// Validate cardinality before allocating or claiming a finite proof domain.
pub fn domain_size(
    agents: usize,
    stock_min: u32,
    stock_max: u32,
    need_min: u32,
    need_max: u32,
    flags: &[u8],
) -> Result<usize, String> {
    if agents == 0
        || agents > 6
        || stock_max > 128
        || need_max > 32
        || stock_min > stock_max
        || need_min > need_max
        || flags.is_empty()
        || flags.iter().any(|&f| f > 1)
    {
        return Err("invalid finite domain".into());
    }
    if flags
        .iter()
        .enumerate()
        .any(|(i, f)| flags[..i].contains(f))
    {
        return Err("duplicate critical flag would count identical worlds repeatedly".into());
    }
    let stocks = u64::from(stock_max - stock_min + 1);
    let needs = u64::from(need_max - need_min + 1)
        .checked_pow(agents as u32)
        .ok_or("finite domain cardinality overflow")?;
    let crits = (flags.len() as u64)
        .checked_pow(agents as u32)
        .ok_or("finite domain cardinality overflow")?;
    let count = stocks
        .checked_mul(needs)
        .and_then(|n| n.checked_mul(crits))
        .ok_or("finite domain cardinality overflow")?;
    if count > 200_000 {
        return Err("domain exceeds 200000 worlds".into());
    }
    Ok(count as usize)
}

pub fn worlds(
    agents: usize,
    stock_min: u32,
    stock_max: u32,
    need_min: u32,
    need_max: u32,
    flags: &[u8],
) -> Result<Vec<World>, String> {
    let capacity = domain_size(agents, stock_min, stock_max, need_min, need_max, flags)?;
    let need_choices: Vec<u32> = (need_min..=need_max).collect();
    let nv = vectors(&need_choices, agents);
    let cv = vectors(flags, agents);
    let mut result = Vec::with_capacity(capacity);
    for stock in stock_min..=stock_max {
        for need in &nv {
            for critical in &cv {
                result.push(World {
                    stock,
                    needs: need.clone(),
                    critical: critical.clone(),
                });
            }
        }
    }
    // Exact Python reference's committed first-witness ordering.
    result.sort_by(|a, b| {
        fn k(w: &World) -> (u64, u64, u32, &Vec<u32>, &Vec<u8>) {
            (
                w.demand()
                    + u64::from(w.stock)
                    + w.critical.iter().map(|&x| u64::from(x)).sum::<u64>(),
                w.demand(),
                w.stock,
                &w.needs,
                &w.critical,
            )
        }
        k(a).cmp(&k(b))
    });
    Ok(result)
}
pub fn reference_worlds() -> Vec<World> {
    worlds(3, 0, 5, 0, 2, &[0, 1]).expect("reference domain is valid")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_counts() {
        assert_eq!(reference_worlds().len(), 1296);
        assert_eq!(reference_grammar().len(), 16);
        assert_eq!(reference_worlds()[0].stock, 0);
    }
    #[test]
    fn bounds_enforced() {
        assert!(worlds(6, 0, 128, 0, 32, &[0, 1]).is_err());
        assert!(worlds(0, 0, 1, 0, 1, &[0, 1]).is_err());
    }

    #[test]
    fn unique_worlds_and_programs_are_required() {
        assert!(worlds(3, 0, 5, 0, 2, &[1, 1]).is_err());
        assert!(worlds(3, 0, 5, 0, 2, &[0, 1, 0]).is_err());
        assert!(grammar(&[Priority::Index, Priority::Index], &[0]).is_err());
        assert!(grammar(&[Priority::Index], &[0, 0]).is_err());
        assert!(grammar(&[Priority::Index], &[10_001]).is_err());
        assert_eq!(domain_size(3, 0, 5, 0, 2, &[0, 1]), Ok(1_296));
    }
}
