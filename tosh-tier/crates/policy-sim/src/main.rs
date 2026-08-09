#![forbid(unsafe_code)]

use expert_cache::ExpertCacheState;
use tier_core::ExpertId;

fn main() {
    let mut state = ExpertCacheState::default();
    let mut accepted = 0_u64;

    for argument in std::env::args().skip(1) {
        let Some((expert, token)) = argument.split_once(':') else {
            eprintln!("ignoring invalid sample {argument:?}; expected expert:token");
            continue;
        };
        let Ok(expert) = expert.parse::<u32>() else {
            eprintln!("ignoring invalid expert id {expert:?}");
            continue;
        };
        let Ok(token) = token.parse::<u64>() else {
            eprintln!("ignoring invalid token position {token:?}");
            continue;
        };
        state.observe(ExpertId(expert), token);
        accepted = accepted.saturating_add(1);
    }

    println!("tosh-tier policy simulator scaffold");
    println!("samples={accepted}");
    println!("lru={:?}", state.rank_lru());
    println!("lfu={:?}", state.rank_lfu());
    if accepted == 0 {
        println!("example: cargo run -p policy-sim -- 4:1 9:2 4:3 7:4");
    }
}
