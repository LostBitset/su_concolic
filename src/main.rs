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
        base_cbs: FullCBS<T::CoT, SymT>,
        solver: Box<dyn Solver<Conj<SymT>, CoT=T::CoT>>,
    ) -> CBSTree<T::CoT, SymT> {
        let FullCBS {
            state_c: base_c,
            state_s: base_s,
            block,
        } = base_cbs;
        let first_cbs = target.exec(base_c, block);
        let initial_path;
        let mut tree = match first_cbs {
            FullCBS { state_c, state_s, block } => {
                initial_path = state_s.0;
                Tree::<_, _>::from_line_right(
                    initial_path.clone(),
                    Some(PureCBS { state_c, block }),
                    || None,
                )
            }
        };
        let mut path_abstr = vec![true; initial_path.len()];
        let mut path = initial_path;
        let mut alt_depth = path.len() - 1;
        loop {
            // invert the path condition
            path[alt_depth].invert();
            path_abstr[alt_depth] = !path_abstr[alt_depth];
            // try to solve for the new path condition
            if let Some(sol) = solver.solve(Conj(path)) {
                let FullCBS {
                    state_c: new_c,
                    state_s: new_s,
                    block: new_block,
                } = target.exec(sol, block);
                if new_s.len() > alt_depth {
                    // extend the tree
                    // set the inversion target to the new bottom
                    todo!();
                } else {
                    if alt_depth == 0 { break; }
                    alt_depth -= 1;
                }
            } else {
                if alt_depth == 0 { break; }
                alt_depth -= 1;
            }
            todo!();
        }
        CBSTree { tree }
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
    }

    impl<CoT: StateCo, SymT: StateSym> Default for CBSTree<CoT, SymT> {
        fn default() -> Self {
            Self {
                tree: Default::default()
            }
        }
    }
}

