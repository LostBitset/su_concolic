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
        cbs: FullCBS<T::CoT, SymT>,
        solver: Box<dyn Solver<SymT, CoT=T::CoT>>,
    ) -> CBSTree<T::CoT, SymT> {
        let mut tree = match cbs {
            FullCBS { state_c, state_s, block } => {
                Tree::<_, _>::from_line(
                    &mut state_s.0.into_iter(),
                    Some(PureCBS { state_c, block }),
                    || None,
                )
            }
        };
        todo!();
        CBSTree { tree }
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

    trait Solver<SymT: StateSym> {
        type CoT;
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
            r: Box<Tree<LeafT, NodeT>>,
        },
        Leaf {
            value: LeafT,
        },
    }

    impl<LeafT, NodeT> Default for Tree<Option<LeafT>, NodeT> {
        fn default() -> Self {
            Self::Leaf {
                value: None
            }
        }
    }

    impl <LeafT, NodeT> Tree<LeafT, NodeT> {
        fn from_line<'a>(
            line: &'a mut impl Iterator<Item=NodeT>,
            leaf_end: LeafT,
            leaf_default: impl Fn() -> LeafT,
        ) -> Self {
            let mut tree = Self::Leaf { value: leaf_end };
            while let Some(head) = line.next() {
                tree = Self::Branch {
                    value: head,
                    l: Box::new(tree),
                    r: Box::new(
                        Self::Leaf { value: leaf_default() }
                    ),
                };
            }
            tree
        }
    }

    struct CBSTree<CoT: StateCo, SymT: StateSym> {
        tree: Tree<Option<PureCBS<CoT>>, SymT>,
    }

    impl<CoT: StateCo, SymT: StateSym> Default for CBSTree<CoT, SymT> {
        fn default() -> Self {
            Self {
                tree: Default::default()
            }
        }
    }
}

