//! Total deterministic integer token interpreter and explicit normative checks.
use nomos_types::{Policy, Priority, World};
pub fn allocate(world: &World, policy: Policy) -> Result<Vec<u32>, String> {
    world.validate()?;
    let mut given = vec![0_u32; world.needs.len()];
    let mut remaining = world.stock.saturating_sub(policy.reserve);
    while remaining > 0 {
        let best = (0..given.len())
            .filter(|&i| given[i] < world.needs[i])
            .min_by_key(|&i| {
                let n = i64::from(world.needs[i]);
                let c = i64::from(world.critical[i]);
                let g = i64::from(given[i]);
                let idx = i as i64;
                match policy.priority {
                    Priority::Index => (idx, 0, 0, 0),
                    Priority::Equal => (g, idx, 0, 0),
                    Priority::NeedsDesc => (-n, g, idx, 0),
                    Priority::NeedsAsc => (n, g, idx, 0),
                    Priority::CriticalIndex => (-c, idx, 0, 0),
                    Priority::CriticalEqual => (-c, g, idx, 0),
                    Priority::CriticalNeedsDesc => (-c, -n, g, idx),
                    Priority::CriticalNeedsAsc => (-c, n, g, idx),
                }
            });
        match best {
            None => break,
            Some(i) => given[i] += 1,
        }
        remaining -= 1;
    }
    Ok(given)
}
pub fn violation(world: &World, given: &[u32]) -> Option<&'static str> {
    if given.len() != world.needs.len() || world.validate().is_err() {
        return Some("nonnegative_allocation");
    }
    if given.iter().zip(&world.needs).any(|(&a, &n)| a > n) {
        return Some("need_cap");
    }
    let total: u64 = given.iter().map(|&n| u64::from(n)).sum();
    if total > u64::from(world.stock) {
        return Some("conservation");
    }
    if total != u64::from(world.stock).min(world.demand()) {
        return Some("nonwaste");
    }
    if given
        .iter()
        .enumerate()
        .any(|(i, &x)| world.critical[i] == 1 && x < world.needs[i])
        && given
            .iter()
            .enumerate()
            .any(|(i, &x)| world.critical[i] == 0 && x > 0)
    {
        return Some("critical_first");
    }
    for i in 0..given.len() {
        for j in (i + 1)..given.len() {
            if world.critical[i] == world.critical[j]
                && world.needs[i] == world.needs[j]
                && given[i].abs_diff(given[j]) > 1
            {
                return Some("bounded_peer_disparity");
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use nomos_types::Priority;
    #[test]
    fn all_policies_conserve_all_worlds() {
        for w in nomos_test_worlds() {
            for p in Priority::ALL {
                for reserve in [0, 1] {
                    let a = allocate(
                        &w,
                        Policy {
                            priority: p,
                            reserve,
                        },
                    )
                    .unwrap();
                    assert!(a.iter().zip(&w.needs).all(|(&x, &y)| x <= y));
                    assert!(a.iter().map(|&x| u64::from(x)).sum::<u64>() <= u64::from(w.stock));
                }
            }
        }
    }
    fn nomos_test_worlds() -> Vec<World> {
        let mut v = vec![];
        for stock in 0..=5 {
            for a in 0..=2 {
                for b in 0..=2 {
                    for c in 0..=2 {
                        for f in 0..8 {
                            v.push(World {
                                stock,
                                needs: vec![a, b, c],
                                critical: vec![
                                    (f >> 2) as u8 & 1,
                                    (f >> 1) as u8 & 1,
                                    (f & 1) as u8,
                                ],
                            });
                        }
                    }
                }
            }
        }
        v
    }
    #[test]
    fn critical_peer_and_nonwaste() {
        let w = World {
            stock: 2,
            needs: vec![2, 2, 2],
            critical: vec![1, 0, 0],
        };
        assert_eq!(
            violation(
                &w,
                &allocate(
                    &w,
                    Policy {
                        priority: Priority::Equal,
                        reserve: 0
                    }
                )
                .unwrap()
            ),
            Some("critical_first")
        );
        assert_eq!(
            violation(
                &w,
                &allocate(
                    &w,
                    Policy {
                        priority: Priority::CriticalEqual,
                        reserve: 0
                    }
                )
                .unwrap()
            ),
            None
        );
    }
}
