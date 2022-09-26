// State-Update Concolic Execution

#![forbid(unsafe_code)]
#![allow(dead_code)]

fn main() {
    println!("Running test...");
    mock::test();
    println!("[OK] Test passed.");
}

mod mock {
    use im::vector;

    use std::rc::Rc;

    use crate::executor::{self, CRPTarget};

    pub fn test() {
        let target = MockCRPTarget::default();
        let solver = MockSolver::new();
        let base_cbs = executor::FullCBS {
            state_c: MockCo {
                the_var: 1,
                the_value: 912,
            },
            state_s: executor::Conj::<MockSym>::new(vector![]),
            block: target.top(),
        };
        println!("Running >>>executor::execute_cbs<<<...");
        let resp = executor::execute_cbs(
            target,
            base_cbs,
            Rc::new(solver),
        );
        println!("[OK-RESP] GOT {:?}", resp);
    }

    struct MockSolver {}

    impl MockSolver {
        fn new() -> Self {
            Self {}
        }
    }

    impl executor::Solver<executor::Conj<MockSym>> for MockSolver {
        type CoT = MockCo;

        fn solve(&self, sym: &executor::Conj<MockSym>) -> Option<Self::CoT> {
            let mut the_sym: Option<MockSym> = None;
            println!("(solver [MockSolver]executor::Solver invoked)");
            for i in sym.clone().into_iter() {
                println!("^ clause: {:?}", i);
                if the_sym.is_none() {
                    the_sym = Some(i);
                } else {
                    panic!("Cannot solve multiple clauses in <mock::MockSolver as executor::Solver<executor::Conj<mock::MockSym>>>::solve")
                }
            }
            let sym;
            if let Some(the_one_sym) = the_sym {
                sym = the_one_sym;
            } else {
                panic!("Cannot solve no clauses in <mock::MockSolver as executor::Solver<executor::Conj<mock::MockSym>>>::solve");
            }
            let MockSym {
                desired_eq, lhs, rhs
            } = sym;
            match (lhs, rhs) {
                (MockSymVar::Value(x), MockSymVar::Value(y)) => {
                    if (x == y) == desired_eq {
                        Some(MockCo::default())
                    } else {
                        None
                    }
                },
                (MockSymVar::Var(var), MockSymVar::Value(z)) => {
                    if desired_eq {
                        Some(MockCo {
                            the_var: var,
                            the_value: z,
                        })
                    } else {
                        Some(MockCo {
                            the_var: var,
                            the_value: z + 1,
                        })
                    }
                },
                (MockSymVar::Value(z), MockSymVar::Var(var)) => {
                    if desired_eq {
                        Some(MockCo {
                            the_var: var,
                            the_value: z,
                        })
                    } else {
                        Some(MockCo {
                            the_var: var,
                            the_value: z + 1,
                        })
                    }
                },
                (MockSymVar::Var(_), MockSymVar::Var(_)) => {
                    panic!("Cannot solve with multiple variables in <mock::MockSolver as executor::Solver<executor::Conj<mock::MockSym>>>::solve");
                }
            }
        }
    }

    #[derive(Default, Debug)]
    struct MockCo {
        the_var: usize,
        the_value: i32,
    }

    impl MockCo {
        fn get(&self, k: &usize) -> Option<&i32> {
            if *k == self.the_var {
                Some(&self.the_value)
            } else {
                panic!("Invalid value access in mock::MockCo[[inherent]]::get");
            }
        }
    }

    impl executor::StateCo for MockCo {}
    
    #[derive(Clone, Debug)]
    struct MockSym {
        desired_eq: bool,
        lhs: MockSymVar,
        rhs: MockSymVar,
    }

    #[derive(Clone, Debug)]
    enum MockSymVar {
        Value(i32),
        Var(usize),
    }

    impl executor::StateSym for MockSym {
        fn invert(&mut self) {
            self.desired_eq = !self.desired_eq;
        }
    }

    #[derive(Clone, Default)]
    struct MockCRPTarget {}

    impl executor::CRPTarget<MockSym> for MockCRPTarget {
        type CoT = MockCo;
        
        fn top(&self) -> executor::BlockId {
            Default::default()
        }
        
        fn exec(&self, state: Self::CoT, block: executor::BlockId)
            -> executor::FullCBS<Self::CoT, MockSym>
        {
            let top = self.top();
            if block == top {
                let cond = {
                    if let Some(value) = state.get(&1) {
                        *value == 42
                    } else {
                        panic!("Value #1 not defined in <mock::MockCRPTarget as executor::CRPTarget<mock::MockSym, CoT=mock::MockCo>>::exec");
                    }
                };
                executor::FullCBS {
                    state_c: state,
                    state_s: executor::Conj::new(
                        vector![
                            MockSym {
                                desired_eq: cond,
                                lhs: MockSymVar::Value(42),
                                rhs: MockSymVar::Var(1),
                            },
                        ]
                    ),
                    block: executor::BlockId::Term,
                }
            } else {
                panic!("Invalid block in <mock::MockCRPTarget as executor::CRPTarget<mock::MockSym, CoT=mock::MockCo>>::exec");
            }
        }
    }
}

mod executor {
    use im::vector::Vector;

    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    pub trait CRPTarget<SymT: StateSym>: Clone {
        type CoT: StateCo;
        fn top(&self) -> BlockId;
        fn exec(&self, state: Self::CoT, block: BlockId)
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
        let first_cbs = target.exec(base_c, block); 
        CBSTree {
            tree: execute_cbs_rec(
                target,
                block,
                first_cbs,
                base_s.clone(),
                solver,
                0
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
        skip_conditions: usize,
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
                       println!(">>In the found (left) case:");
                       println!("^^left> {:?}", pre_conj);
                       Conj(pre_conj)
                   },
                   solver.clone(),
                   skip_conditions + 1,
                ));
                let r = Box::new({
                    let mut pre_conj = precedent.0.clone();
                    pre_conj.push_front(head.invert_clone());
                    println!(">>In the solved (right) case:");
                    println!("^^right> {:?}", pre_conj);
                    let precedent_inv = Conj(pre_conj);
                    if let Some(sol) = solver.solve(&precedent_inv) {
                        let FullCBS {
                            state_c, mut state_s, block
                        } = target.exec(sol, block);
                        state_s.skip_clauses(skip_conditions);
                        execute_cbs_rec(
                            target.clone(),
                            block,
                            FullCBS {
                                state_c,
                                state_s,
                                block,
                            },
                            precedent_inv,
                            solver,
                            skip_conditions + 1,
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

    pub trait StateSym: Clone + std::fmt::Debug {
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

    #[derive(Clone, Debug)]
    pub struct Conj<SymT: StateSym>(Vector<SymT>);

    #[derive(Clone, Debug)]
    pub struct Disj<SymT: StateSym>(Vector<SymT>);

    impl<SymT: StateSym> Conj<SymT> {
        pub fn new(vector: Vector<SymT>) -> Self {
            Self(vector)
        }

        fn len(&self) -> usize {
            self.0.len()
        }
    }

    impl<SymT: StateSym> Disj<SymT> {
        pub fn new(vector: Vector<SymT>) -> Self {
            Self(vector)
        }

        fn len(&self) -> usize {
            self.0.len()
        }
    }

    impl<SymT: StateSym> IntoIterator for Conj<SymT> {
        type IntoIter = <Vector<SymT> as IntoIterator>::IntoIter;
        type Item = SymT;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<SymT: StateSym> IntoIterator for Disj<SymT> {
        type IntoIter = <Vector<SymT> as IntoIterator>::IntoIter;
        type Item = SymT;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    pub trait Solver<T> {
        type CoT;

        fn solve(&self, sym: &T) -> Option<Self::CoT>;
    }

    #[derive(Copy, Clone, PartialEq, Hash, Debug)]
    pub enum BlockId {
        Id(u32),
        Term,
    }

    impl Default for BlockId {
        fn default() -> Self {
            Self::Id(0)
        }
    }

    #[derive(Debug)]
    #[non_exhaustive]
    enum BlockIdError {
        CannotIncrementTerm,
    }

    impl BlockId {
        fn try_inc(&self) -> Result<Self, BlockIdError> {
            match self {
                Self::Id(n) => Ok(
                    Self::Id(n + 1)
                ),
                Self::Term => Err(
                    BlockIdError::CannotIncrementTerm
                ),
            }
        }
    }

    #[derive(Default)]
    pub struct BlockIdGen {
        last: BlockId,
    }

    impl BlockIdGen {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn from_last(last: BlockId) -> Self {
            Self {
                last,
            }
        }
    }

    impl Iterator for BlockIdGen {
        type Item = BlockId;

        fn next(&mut self) -> Option<Self::Item> {
            self.last.try_inc().unwrap();
            Some(self.last)
        }
    }

    pub struct FullCBS<CoT: StateCo, SymT: StateSym> {
        pub state_c: CoT,
        pub state_s: Conj<SymT>,
        pub block: BlockId,
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

    #[derive(Debug)]
    struct PureCBS<CoT: StateCo> {
        state_c: CoT,
        block: BlockId,
    }

    #[derive(Debug)]
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

    #[derive(Debug)]
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

