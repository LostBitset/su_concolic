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
        fn exec(&self, state: &Self::CoT, block: BlockId)
            -> FullCBS<Self::CoT, SymT>;
    }

    fn execute_cbs<SymT: StateSym, T: CRPTarget<SymT>>(
        target: T,
        base_cbs: FullCBS<T::CoT, SymT>,
        solver: Box<dyn Solver<Conj<SymT>, CoT=T::CoT>>,
    ) -> CBSTree<T::CoT, SymT> {
        let FullCBS {
            state_c: base_c,
            state_s: base_s,
            block,
        } = base_cbs;
        let first_cbs = target.exec(&base_c, block); 
        CBSTree {
            tree: execute_cbs_rec(target, block, first_cbs, solver),
            precedent: base_s,
        }
    }

    fn execute_cbs_rec<SymT: StateSym, T: CRPTarget<SymT>>(
        target: T,
        block: BlockId,
        left_example: FullCBS<T::CoT, SymT>,
        solver: Box<dyn Solver<Conj<SymT>, CoT=T::CoT>>,
    ) -> Tree<Option<PureCBS<T::CoT>>, SymT> {
        match &left_example.state_s.0[..] {
            [head, tail @ ..] => {
                todo!()
                /*Tree::Branch {
                    value: head,
                    l: Box::new(execute_cbs_rec(
                        target,
                        block,
                        FullCBS {
                            state_s: Conj(tail),
                            ..left_example
                        },
                        solver,
                    )),
                    r: todo!(),
                }*/
            },
            _ => {
                todo!()
            },
        }
    }

    trait StateCo {
        // Methods will be defined here...
    }

    trait StateSym: Clone {
        // Methods will be defined here...
        
        fn invert(&mut self);
    }

    #[derive(Clone)]
    struct Conj<SymT: StateSym>(Vec<SymT>);

    #[derive(Clone)]
    struct Disj<SymT: StateSym>(Vec<SymT>);

    impl<SymT: StateSym> Conj<SymT> {
        fn len(&self) -> usize {
            self.0.len()
        }
    }

    impl<SymT: StateSym> Disj<SymT> {
        fn len(&self) -> usize {
            self.0.len()
        }
    }

    trait Solver<T> {
        type CoT;

        fn solve(&self, sym: T) -> Option<Self::CoT>;
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
        fn from_line_left(
            line: Vec<NodeT>,
            leaf_end: LeafT,
            leaf_default: impl Fn() -> LeafT,
        ) -> Self {
            let mut line = line.into_iter().rev();
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
        
        fn from_line_right(
            line: Vec<NodeT>,
            leaf_end: LeafT,
            leaf_default: impl Fn() -> LeafT,
        ) -> Self {
            let mut line = line.into_iter().rev();
            let mut tree = Self::Leaf { value: leaf_end };
            while let Some(head) = line.next() {
                tree = Self::Branch {
                    value: head,
                    l: Box::new(
                        Self::Leaf { value: leaf_default() }
                    ),
                    r: Box::new(tree),
                };
            }
            tree
        }
    }

    struct CBSTree<CoT: StateCo, SymT: StateSym> {
        tree: Tree<Option<PureCBS<CoT>>, SymT>,
        precedent: Conj<SymT>,
    }

    impl<CoT: StateCo, SymT: StateSym> CBSTree<CoT, SymT> {
        fn from_precedent(precedent: Conj<SymT>) -> Self {
            Self {
                tree: Default::default(),
                precedent,
            }
        }
    }
}

