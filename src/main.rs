// State-Update Concolic Execution

#![forbid(unsafe_code)]
#![allow(dead_code)] // For now...

fn main() {
    println!("Hello world!");
}

mod executor {
    use crate::data;

    pub trait CRPTarget<SymT: data::StateSym> {
        type CoT: data::StateCo;
        fn top(&self) -> data::BlockId;
        fn exec(&self, state: Self::CoT, block: data::BlockId)
            -> data::FullCBS<Self::CoT, SymT>;
    }

    fn execute_cbs<SymT: data::StateSym, T: CRPTarget<SymT>>(
        target: T,
        cbs: data::FullCBS<T::CoT, SymT>
    ) -> data::CBSSet<T::CoT, SymT> {
        todo!()
    }
}

mod data {
    use std::collections::{HashMap, HashSet};

    pub trait StateCo {
        // Methods will be defined here...
    }

    pub trait StateSym {
        // Methods will be defined here...
    }

    pub struct Conj<SymT: StateSym>(Vec<SymT>);
    pub struct Disj<SymT: StateSym>(Vec<SymT>);

    impl<SymT: StateSym> StateSym for Conj<SymT> {
        // Implementation will be defined here...
    }

    impl<SymT: StateSym> StateSym for Disj<SymT> {
        // Implementation will be defined here...
    }

    #[derive(Copy, Clone, PartialEq, Hash)]
    pub struct BlockId(u32);

    pub struct FullCBS<CoT: StateCo, SymT: StateSym> {
        state_c: CoT,
        state_s: Conj<SymT>,
        block: BlockId,
    }

    pub struct CBSSet<CoT: StateCo, SymT: StateSym> {
        set: HashSet<FullCBS<CoT, SymT>>,
    }

    pub struct PartialCBS<CoT: StateCo, SymT: StateSym> {
        state_c: CoT,
        state_s: Conj<SymT>,
    }

    pub enum IntermediateCBS<CoT: StateCo, SymT: StateSym> {
        CBSMap(HashMap<BlockId, PartialCBS<CoT, SymT>>),
        PureCBSMap(PureCBS<CoT>),
    }

    pub struct PureCBS<CoT: StateCo> {
        state_c: CoT,
        block: BlockId,
    }

    pub enum Tree<LeafT, NodeT> {
        Branch {
            value: NodeT,
            l: Box<Tree<LeafT, NodeT>>,
        },
        Leaf {
            value: LeafT,
        },
    }

    pub struct CBSTree<CoT: StateCo, SymT: StateSym> {
        tree: Tree<PureCBS<CoT>, SymT>,
    }
}

