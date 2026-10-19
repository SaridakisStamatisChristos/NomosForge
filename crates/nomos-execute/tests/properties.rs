use nomos_execute::{allocate, violation};
use nomos_types::{Policy, Priority, World};
use proptest::prelude::*;
proptest! {
    #[test]
    fn bounded_policies_conserve(stock in 0u32..128,
        agents in prop::collection::vec((0u32..32, any::<bool>()),1..=6)) {
        let w=World{
            stock,needs:agents.iter().map(|(n,_)|*n).collect(),
            critical:agents.iter().map(|(_,c)|u8::from(*c)).collect()
        };
        w.validate().unwrap();
        for priority in Priority::ALL {
            for reserve in [0,1] {
                let a=allocate(&w,Policy{priority,reserve}).unwrap();
                prop_assert_eq!(a.len(),w.needs.len());
                prop_assert!(a.iter().zip(&w.needs).all(|(&x,&n)|x<=n));
                prop_assert!(a.iter().map(|&x|u64::from(x)).sum::<u64>()<=u64::from(stock));
                if priority==Priority::CriticalEqual && reserve==0 {
                    prop_assert_eq!(violation(&w,&a),None);
                }
            }
        }
    }
}
