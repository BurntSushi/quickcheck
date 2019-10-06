
use crate::tester::{Testable, TestResult};
use crate::arbitrary::{Gen, Arbitrary};
use std::fmt::{Debug};

pub trait Model: Default + Debug {
    type Operation: Arbitrary + Clone + Debug;

    /// Generates a next operation given the current state of self.
    /// Subsequently, this operation will be subject of [Self::pre] check to
    /// determine, if it's a correct one in context of a current state.
    fn next<G: Gen>(&self, g: &mut G) -> Option<Self::Operation> { 
        Some(Self::Operation::arbitrary(g)) 
    }

    /// A preconditions used to check if generated operation is correct 
    /// in sense of a current worflow. This function may be executed 
    /// multiple times in a single run, therefore it's expected to not 
    /// produce any side effects.
    fn pre(&self, _: &Self::Operation) -> bool { true }

    /// An actual operation to be run.
    fn run(&mut self, op: &Self::Operation) -> bool;
}

pub struct StateMachine<T: Model> {
    min_ops: usize,
    max_ops: usize,
    init: fn() -> T
}

impl<T: Model> StateMachine<T> {

    /// Creates a new state machine for a specific test scenario. It will
    /// be able to create a stateful specification model instances of type
    /// T per each test run. 
    pub fn from(init: fn() -> T) -> Self {
        StateMachine {
            min_ops: 1,
            max_ops: 100,
            init
        }
    }

    /// Defines a maximum number of operations to be generated in order to
    /// produce a stateful test scenario for model T.
    pub fn max_ops(mut self, value: usize) -> Self {
        self.max_ops = value;
        self
    }

    /// Defines a minimal number of operations to be generated in order to
    /// produce a stateful test scenario for model T.
    pub fn min_ops(mut self, value: usize) -> Self {
        self.min_ops = value;
        self
    }
}

impl<T: Default + Model> Default for StateMachine<T> {
    fn default() -> Self {
        StateMachine {
            min_ops: 1,
            max_ops: 100,
            init: T::default
        }
    }
}

impl<T: Model + 'static> Testable for StateMachine<T> {

    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        let mut state = (self.init)();
        let op_count = (g.size() % (self.max_ops - self.min_ops)) + self.min_ops;
        let mut operations = Vec::with_capacity(op_count);
        let mut i = 0;
        while i < op_count {
            i += 1;
            loop {
                let op = state.next(g);
                match op {
                    Some(ref o) => {
                        if state.pre(o) {
                            operations.push(o.clone());
                            let result = state.run(o);
                            if !result {
                                let arguments = operations.clone()
                                    .into_iter()
                                    .take(i+1)
                                    .map(|op| format!("{:?}", op))
                                    .collect();
                                let msg = format!("Model failed in state {:?} after executing {} operations", state, i);
                                return TestResult::error_with_args(msg, arguments);
                            } else {
                                break; // break current generation loop
                            }
                        }
                        // if state.pre failed - loop around and regenerate operation
                    },
                    // prematurelly finish test eg. because we reached final state
                    None => return TestResult::passed(), 
                };
            }
        }

        TestResult::passed()
    }
}

#[cfg(test)]
mod test {

    use rand::rngs::OsRng;
    use crate::statem::{Model, StateMachine};
    use crate::arbitrary::{Gen, StdGen, Arbitrary};
    use crate::tester::QuickCheck;
    use std::fmt::Debug;

    #[derive(Default, Clone, Debug, PartialEq, Eq)]
    struct Counter(u32);

    impl Counter {

        fn inc(&mut self) -> u32 {
            /*
            // intentional bug
            if self.0 < 3 {
                self.0 += 1;
            } else {
                self.0 += 2; 
            }*/
            self.0 += 1;
            self.0
        }

        fn dec(&mut self) -> Result<u32, ()> {
            if self.0 == 0 {
                Err(())
            } else {
                self.0 -= 1;
                Ok(self.0)
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum CounterOp {
        Increment,
        Decrement,
    }

    impl Arbitrary for CounterOp {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if g.next_u32() % 2 == 0 {
                CounterOp::Increment
            } else {
                CounterOp::Decrement
            }
        }
    }

    #[derive(Default, Debug)]
    struct CounterSpec {
        counter: Counter,
    }

    impl Model for CounterSpec {
        type Operation = CounterOp;

        fn next<G: Gen>(&self, g: &mut G) -> Option<Self::Operation> {
            Some(CounterOp::arbitrary(g))
        }

        fn pre(&self, op: &Self::Operation) -> bool {
            match op {
                CounterOp::Decrement => self.counter.0 > 0,
                _ => true
            }
        }

        fn run(&mut self, op: &Self::Operation) -> bool {
            match op {
                CounterOp::Increment => {
                    let expected = self.counter.0 + 1;
                    expected == self.counter.inc()
                },
                CounterOp::Decrement => {
                    let expected = self.counter.0 - 1;
                    Ok(expected) == self.counter.dec()
                }
            }
        }
    }

    #[test]
    fn test_counter() {
        let spec = StateMachine::from(CounterSpec::default)
            .min_ops(20)
            .max_ops(50);

        QuickCheck::with_gen(StdGen::new(OsRng, 129))
            .quickcheck(spec);
    }
}