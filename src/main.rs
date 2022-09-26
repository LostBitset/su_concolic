// State-Update Concolic Execution

#![forbid(unsafe_code)]
#![allow(dead_code)]

fn main() {
    println!("Hello world!");
}

mod executor {
    use im::vector::Vector;

    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    pub trait CRPTarget<SymT: StateSym>: Clone {
        type CoT: StateCo;
        fn top(&self) -> BlockId;
        fn exec(&self, state: &Self::CoT, block: BlockId)
            -> FullCBS<Self::CoT, SymT>;
    }

    pub fn execute_cbs<SymT: StateSym, T: CRPTarget<SymT>>(
        target: T,
        base_cbs: FullCBS<T::CoT, SymT>,
        solver: Rc<dyn Solver<Conj<SymT>, CoT=T::CoT>>,
    ) -> CBSTree<T::CoT, SymT> {
        let FullCBS {
            state_c: base_c,
            state_s: base_s,
            block,
        } = base_cbs;
        let first_cbs = target.exec(&base_c, block); 
        CBSTree {
            tree: execute_cbs_rec(
                target,
                block,
                first_cbs,
                base_s.clone(),
                solver
            ),
            precedent: base_s,
        }
    }

    fn execute_cbs_rec<SymT: StateSym, T: CRPTarget<SymT>>(
        target: T,
        block: BlockId,
        left_example: FullCBS<T::CoT, SymT>,
        precedent: Conj<SymT>,
        solver: Rc<dyn Solver<Conj<SymT>, CoT=T::CoT>>,
    ) -> Tree<Option<PureCBS<T::CoT>>, SymT> {
        let mut state_conj = left_example.state_s.0.clone();
        match state_conj.pop_front() {
            Some(head) => {
                let l = Box::new(execute_cbs_rec(
                   target.clone(),
                   block,
                   FullCBS {
                       state_s: Conj(state_conj),
                       ..left_example
                   },
                   {
                       let mut pre_conj = precedent.0.clone();
                       pre_conj.push_front(head.clone());
                       Conj(pre_conj)
                   },
                   solver.clone(),
                ));
                let r = Box::new({
                    let mut pre_conj = precedent.0.clone();
                    pre_conj.push_front(head.invert_clone());
                    let precedent_inv = Conj(pre_conj);
                    if let Some(sol) = solver.solve(&precedent_inv) {
                        execute_cbs_rec(
                            target.clone(),
                            block,
                            target.exec(&sol, block),
                            precedent_inv,
                            solver,
                        )
                    } else {
                        Tree::Leaf {
                            value: None,
                        }
                    }
                });
                Tree::Branch {
                    value: head.to_owned(),
                    l,
                    r,
                }
            },
            None => {
                Tree::Leaf {
                    value: Some(
                        PureCBS {
                            state_c: left_example.state_c,
                            block: left_example.block,
                        }
                    ),
                }
            },
        }
    }

    pub trait StateCo {
        // Methods will be defined here...
    }

    pub trait StateSym: Clone {
        // Methods will be defined here...
        
        fn invert(&mut self);
    }

    trait InvertClone: Clone {
        fn invert_clone(&self) -> Self;
    }

    impl<ImplT: StateSym> InvertClone for ImplT {
        fn invert_clone(&self) -> Self {
            let mut new = self.clone();
            new.invert();
            new
        }
    }

    #[derive(Clone)]
    pub struct Conj<SymT: StateSym>(Vector<SymT>);

    #[derive(Clone)]
    pub struct Disj<SymT: StateSym>(Vector<SymT>);

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

    pub trait Solver<T> {
        type CoT;

        fn solve(&self, sym: &T) -> Option<Self::CoT>;
    }

    #[derive(Copy, Clone, PartialEq, Hash)]
    pub struct BlockId(u32);

    pub struct FullCBS<CoT: StateCo, SymT: StateSym> {
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

    pub struct CBSTree<CoT: StateCo, SymT: StateSym> {
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

