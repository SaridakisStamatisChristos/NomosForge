//! Finite grammar and deterministic bounded world enumeration.
use nomos_types::{Policy, Priority, World};
pub fn grammar(priorities: &[Priority], reserves: &[u32]) -> Result<Vec<Policy>, String> {
    if priorities.is_empty() || reserves.is_empty() || priorities.len() * reserves.len() > 512 {
        return Err("grammar must be nonempty and contain at most 512 programs".into());
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
pub fn worlds(
    agents: usize,
    stock_min: u32,
    stock_max: u32,
    need_min: u32,
    need_max: u32,
    flags: &[u8],
) -> Result<Vec<World>, String> {
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
    let stocks = u64::from(stock_max - stock_min + 1);
    let needs = u64::from(need_max - need_min + 1).pow(agents as u32);
    let crits = (flags.len() as u64).pow(agents as u32);
    if stocks
        .checked_mul(needs)
        .and_then(|n| n.checked_mul(crits))
        .unwrap_or(u64::MAX)
        > 200_000
    {
        return Err("domain exceeds 200000 worlds".into());
    }
    let need_choices: Vec<u32> = (need_min..=need_max).collect();
    let nv = vectors(&need_choices, agents);
    let cv = vectors(flags, agents);
    let mut result = Vec::with_capacity((stocks * needs * crits) as usize);
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
}
