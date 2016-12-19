//! Nondeterministic Finite Automaton (NFA) implementation.

use std::collections::{HashMap, VecDeque};
use std::fmt;

/// An `Option`-like enum holding a state transition
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Transition {
    /// Transition on some input character
    Input(char),
    /// ε-transition, doens't consume any input
    Epsilon,
}

use self::Transition::{Input, Epsilon};

/// NFA state
pub struct Nfa {
    /// Vector of states. Each state contains a `HashMap` to lookup
    /// the next possible states for any input character (or `None`
    /// for ε-transitions). Stored as a double ended queue since we
    /// might have to push new states to the front (for instance in
    /// the case of unions).
    ///
    /// Since it's possible for an NFA to have multiple possible
    /// transitions for a single input those transitions are stored in
    /// a `Vec`.
    ///
    /// The state transitions themselves are stored as a signed offset
    /// to the next state within the table. This way we can move those
    /// states around or even copy them into an other table without
    /// having to rewrite all the pointers.
    ///
    /// The final state of the NFA (denoted `(f)` in this file) is not
    /// stored in the vector since it doesn't have any transition and
    /// it makes implementing concatenations easier. That means that
    /// states that transition into the final state point one past the
    /// last element.
    states: VecDeque<HashMap<Transition, Vec<isize>>>,
}

impl Nfa {
    /// Create a new NFA matching `c`
    ///
    /// ```text
    ///        c
    /// (0) ------> (f)
    /// ```
    pub fn new(c: char) -> Nfa {
        let mut map = HashMap::new();

        map.insert(Input(c), vec![1]);

        let mut states = VecDeque::new();

        states.push_back(map);

        Nfa {
            states: states,
        }
    }

    /// Concatenate two NFAs. `a.concat(b)` matches `ab`.
    ///
    /// ```text
    ///        a           b
    /// (0) ------> (1) ------> (f)
    /// ```
    pub fn concat(&mut self, mut other: Nfa) {
        self.states.append(&mut other.states);
    }

    /// "Or" two NFAs. `a.union(b)` matches `a|b`.
    ///
    /// ```text
    ///        ε           a           ε
    /// (0) ------> (1) ------> (2) ------> (f)
    ///   \    ε           b           ε    ^
    ///    `------> (3) ------> (4) -------'
    /// ```
    pub fn union(&mut self, mut other: Nfa) {
        // Since we store all the states in a linear vector we'll
        // concatenate them like so:
        //
        //  ```text
        //              ε
        //    ,------------------.
        //   /  ε      a          v  b      ε
        //  (0) -> (1) -> (2)    (3) -> (4) -> (f)
        //                 \         ε         ^
        //                  `-----------------'
        //  ```

        // XXX I don't know if states (2) and (4) above are really
        // necessary here, it seems to me we could just transition
        // from (1) and (2) straight to (f). That being said the
        // dragon book does it so I'm going to dutifully abide for
        // now, once this is working it'll be easier to modify it and
        // figure out if it breaks something...


        // First let's create the previous final state of `other` ((4)
        // above) which will transition into the new final state
        let mut state_4 = HashMap::new();

        // We're going to put the new final state at the end (as usual).
        state_4.insert(Epsilon, vec![1]);

        other.states.push_back(state_4);

        // Next let's do the same thing for `self`
        let mut state_2 = HashMap::new();

        // We're going to append `other`'s state to `self.state` so we
        // need to compute the index accordingly.
        state_2.insert(Epsilon, vec![other.states.len() as isize + 1]);

        self.states.push_back(state_2);

        // We also need to create state (0) which just ε-transitions
        // into the two previous NFAs
        let mut state_0 = HashMap::new();

        state_0.insert(Epsilon,
                       vec![
                           // points to (1)
                           1,
                           // points to (3)
                           self.states.len() as isize + 1]);


        self.states.push_front(state_0);

        // We can now concatenate everything and it should just
        // work. (2) and (4) will be pointing one-past the last item
        // in `states` as usual.
        self.states.append(&mut other.states);
    }

    /// Compute the Kleene closure or Kleene star of this
    /// NFA. `a.star()` matches `a*`.
    ///
    /// ```text
    ///                    ε
    ///               ,--------.
    ///        ε     v     a    \      ε
    /// (0) ------> (1) ------> (2) ------> (f)
    ///  \                 ε                 ^
    ///   `---------------------------------'
    /// ```
    pub fn star(&mut self) {
        // Create state (2) and have it point at the new final state
        // (f) and the current first state (1)
        let mut state_2 = HashMap::new();

        state_2.insert(Epsilon, vec![1, -(self.states.len() as isize)]);

        self.states.push_back(state_2);

        // Create state (0) pointing at (1) and (f)
        let mut state_0 = HashMap::new();

        state_0.insert(Epsilon, vec![1, self.states.len() as isize]);

        self.states.push_front(state_0);
    }

    /// Returns a `Vec` of states that are reachable from `states`
    /// using ε-transitions alone.
    pub fn epsilon_closure(&self, states: &[usize]) -> Vec<usize> {
        let mut epsi_states = Vec::new();

        // Any state can ε-transition to itself
        epsi_states.extend(states);

        // A stack used to track all the states that remain to be
        // visited since we want to travel ε-transitions recursively.
        let mut remaining_states = epsi_states.clone();

        while let Some(state) = remaining_states.pop() {
            let transitions = self.transitions(state, Epsilon);

            for t in transitions {
                let next_state = (state as isize + t) as usize;

                if epsi_states.iter().find(|&&s| s == next_state).is_none() {
                    // We found a new state for the ε-closure
                    epsi_states.push(next_state);
                    remaining_states.push(next_state);
                }
            }
        }

        epsi_states
    }

    /// Returns the list of state *offsets* from `state` reachable
    /// through a transition using `input`.
    pub fn transitions(&self, state: usize, input: Transition) -> &[isize] {
        if let Some(state) = self.states.get(state) {
            if let Some(ref trans) = state.get(&input) {
                return trans
            }
        }

        &[]
    }
}

impl fmt::Debug for Nfa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (state, transitions) in self.states.iter().enumerate() {
            try!(write!(f, "({}):\n", state));
            for (&input, transition) in transitions {
                let input =
                    match input {
                        Input(c) => c,
                        Epsilon => 'ε',
                    };

                try!(write!(f, "    {} ->", input));
                for t in transition {
                    try!(write!(f, " {}", state as isize + t));
                }
                try!(writeln!(f, ""));
            }
        }
        Ok(())
    }
}
