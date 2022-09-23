// State-Update Concolic Execution

#![forbid(unsafe_code)]
#![allow(dead_code)] // For now...

fn main() {
    println!("Hello world!");
}

mod data {
    use std::collections::{HashMap, HashSet};

    trait StateC {}

    trait StateS {}

    struct Conj<SymT: StateS>(Vec<SymT>);
    struct Disj<SymT: StateS>(Vec<SymT>);

    impl<SymT: StateS> StateS for Conj<SymT> {}
    impl<SymT: StateS> StateS for Disj<SymT> {}

    #[derive(Copy, Clone, PartialEq, Hash)]
    struct BlockId(u32);

    struct FullCBS<CoT: StateC, SymT: StateS> {
        state_c: CoT,
        state_s: Conj<SymT>,
        block: BlockId,
    }

    struct CBSSet<CoT: StateC, SymT: StateS> {
        set: HashSet<FullCBS<CoT, SymT>>,
    }

    struct PartialCBS<CoT: StateC, SymT: StateS> {
        state_c: CoT,
        state_s: Conj<SymT>,
    }

    enum IntermediateCBS<CoT: StateC, SymT: StateS> {
        CBSMap {
            map: HashMap<BlockId, PartialCBS<CoT, SymT>>
        },
        PureCBS {
            state_c: CoT,
            block: BlockId,
        },
    }

    enum Tree<LeafT, NodeT> {
        Branch {
            value: NodeT,
            l: Box<Tree<LeafT, NodeT>>,
        },
        Leaf {
            value: LeafT,
        },
    }
}

