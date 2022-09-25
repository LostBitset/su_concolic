// State-Update Concolic Execution

#![forbid(unsafe_code)]
#![allow(dead_code)] // For now...
#![allow(unused_variables)] // For now...

fn main() {
    println!("Hello world!");
}

mod executor {
    use std::collections::{HashMap, HashSet};

    trait CRPTarget<SymT: StateSym> {
        type CoT: StateCo;
        fn top(&self) -> BlockId;
        fn exec(&self, state: Self::CoT, block: BlockId)
            -> FullCBS<Self::CoT, SymT>;
    }

    fn execute_cbs<SymT: StateSym, T: CRPTarget<SymT>>(
        target: T,
        cbs: FullCBS<T::CoT, SymT>
    ) -> CBSTree<T::CoT, SymT> {
        let FullCBS {
           state_c, state_s, block 
        } = cbs;
        let initial_exec = target.exec(state_c, block);
        todo!()
    }

    #[derive(Clone, Default)]
    struct Pathsrc {
        known_part: Vec<bool>,
        found_part: Vec<bool>,
        idx: usize,
    }

    impl Pathsrc {
        fn from_known(known_part: Vec<bool>) -> Self {
            let at_known = !known_part.is_empty();
            Self {
                known_part,
                found_part: Vec::new(),
                ..Default::default()
            }
        }

        fn solidify(&mut self) {
            self.known_part.append(&mut self.found_part);
        }

        fn get_choice(&mut self, fallback: bool) -> bool {
            let known_len = self.known_part.len();
            let res;
            if self.idx >= known_len {
                res = self.known_part[self.idx]
            } else if self.idx >= known_len + self.found_part.len() {
                res = self.found_part[self.idx + known_len]
            } else {
                res = fallback
            }
            self.idx += 1;
            return fallback;
        }
    }

    trait StateCo {
        // Methods will be defined here...
    }

    trait StateSym {
        // Methods will be defined here...
    }

    struct Conj<SymT: StateSym>(Vec<SymT>);
    struct Disj<SymT: StateSym>(Vec<SymT>);

    impl<SymT: StateSym> StateSym for Conj<SymT> {
        // Implementation will be defined here...
    }

    impl<SymT: StateSym> StateSym for Disj<SymT> {
        // Implementation will be defined here...
    }

    #[derive(Copy, Clone, PartialEq, Hash)]
    struct BlockId(u32);

    struct FullCBS<CoT: StateCo, SymT: StateSym> {
        state_c: CoT,
        state_s: Conj<SymT>,
        block: BlockId,
    }

    struct CBSSet<CoT: StateCo, SymT: StateSym> {
        set: HashSet<FullCBS<CoT, SymT>>,
    }

    struct PartialCBS<CoT: StateCo, SymT: StateSym> {
        state_c: CoT,
        state_s: Conj<SymT>,
    }

    enum IntermediateCBS<CoT: StateCo, SymT: StateSym> {
        CBSMap(HashMap<BlockId, PartialCBS<CoT, SymT>>),
        PureCBSMap(PureCBS<CoT>),
    }

    struct PureCBS<CoT: StateCo> {
        state_c: CoT,
        block: BlockId,
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

    struct CBSTree<CoT: StateCo, SymT: StateSym> {
        tree: Tree<PureCBS<CoT>, SymT>,
    }
}

