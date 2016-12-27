//! Nondeterministic Finite Automaton (NFA) implementation.

use std::collections::{HashMap, VecDeque};
use std::fmt;

/// NFA state
struct State {
    /// Map of all valid moves from this state. The target state is
    /// stored as a signed offset in the
    moves: HashMap<Transition, Vec<isize>>,
    /// `None` if the state is non-accepting
    accepting: Option<String>,
}

impl State {
    /// Creates a new non-accepting state with no moves
    fn new() -> State {
        State {
            moves: HashMap::new(),
            accepting: None,
        }
    }


    /// Creates a new accepting state with no moves
    fn new_accepting(desc: String) -> State {
        State {
            moves: HashMap::new(),
            accepting: Some(desc),
        }
    }

    fn accepting(&self) -> Option<String> {
        self.accepting.clone()
    }

    /// Set the moves on `input`. Any previous moves on `input` are
    /// discarded.
    fn set_moves(&mut self, input: Transition, targets: Vec<isize>) {
        self.moves.insert(input, targets);
    }

    /// Returns the moves on `input`
    fn get_moves(&self, input: Transition) -> &[isize] {
        match self.moves.get(&input) {
            Some(v) => v,
            None => &[],
        }
    }

    fn move_map(&self) -> &HashMap<Transition, Vec<isize>> {
        &self.moves
    }
}

/// NFA graph
pub struct Nfa {
    /// Vector of the states of the Nfa. Each state represents
    /// transitions as signed indices pointing to the other states in
    /// this vector.
    states: VecDeque<State>,
}

impl Nfa {
    /// Create a new NFA matching `c`
    ///
    /// ```text
    ///        c
    /// (0) ------> (f)
    /// ```
    pub fn new(c: char) -> Nfa {
        let mut state = State::new();

        state.set_moves(Input(c), vec![1]);

        let mut states = VecDeque::new();

        states.push_back(state);

        Nfa {
            states: states,
        }
    }

    /// Create a new NFA with a single accepting state having no
    /// transitions
    ///
    /// ```text
    /// ((0))
    /// ```
    pub fn new_accepting(desc: String) -> Nfa {
        let state = State::new_accepting(desc);

        let mut states = VecDeque::new();

        states.push_back(state);

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
        let mut state_4 = State::new();

        // We're going to put the new final state at the end (as usual).
        state_4.set_moves(Epsilon, vec![1]);

        other.states.push_back(state_4);

        // Next let's do the same thing for `self`
        let mut state_2 = State::new();

        // We're going to append `other`'s state to `self.state` so we
        // need to compute the index accordingly.
        state_2.set_moves(Epsilon, vec![other.states.len() as isize + 1]);

        self.states.push_back(state_2);

        // We also need to create state (0) which just ε-transitions
        // into the two previous NFAs
        let mut state_0 = State::new();

        state_0.set_moves(Epsilon,
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
        let mut state_2 = State::new();

        state_2.set_moves(Epsilon, vec![1, -(self.states.len() as isize)]);

        self.states.push_back(state_2);

        // Create state (0) pointing at (1) and (f)
        let mut state_0 = State::new();

        state_0.set_moves(Epsilon, vec![1, self.states.len() as isize + 1]);

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
                if epsi_states.iter().find(|&&s| s == t).is_none() {
                    // We found a new state for the ε-closure
                    epsi_states.push(t);
                    remaining_states.push(t);
                }
            }
        }

        epsi_states
    }

    /// Returns the list of states reachable through a transition
    /// from `state` using `input`.
    pub fn transitions(&self, state_idx: usize, input: Transition) -> Vec<usize> {
        let mut states = Vec::new();

        if let Some(state) = self.states.get(state_idx) {
            let moves = state.get_moves(input);

            states.extend(moves.iter()
                          .map(|off| (state_idx as isize + off) as usize));
        }

        states
    }

    /// Returns the ε-closure of all the states reachable by any non-ε
    /// transition from any of the `states` provided.
    ///
    /// For instance, given the following NFA:
    ///
    /// ```text
    ///       a           ε
    ///   ,------> (2) ------> (3)
    ///  /          \     b
    /// (0)          `-------> (4)
    ///  \    ε           c           ε
    ///   `------> (1) ------> (5) --------.
    ///             \     ε           d     v
    ///              `-------> (6) ------> (7)
    /// ```
    ///
    /// `get_move_set(&[0])` would return a `HashMap` containing a
    /// single entry: `'a' -> [2, 3]`.
    ///
    /// `get_move_set(&[0, 1, 6])` (the ε-closure of state 0) would
    /// return a `HashMap` with 3 entries:
    ///
    /// ```text
    /// 'a' -> [2, 3]
    /// 'c' -> [5, 7]
    /// 'd' -> [7]
    /// ```
    pub fn get_move_set(&self, states: &[usize]) -> HashMap<char, Vec<usize>> {
        let mut m_s = HashMap::new();

        for &s in states {
            if let Some(state) = self.states.get(s) {

                for (transition, target) in state.move_map() {
                    // We ignore ε-transitions
                    if let &Input(c) = transition {
                        let cur_states = m_s.entry(c).or_insert(Vec::new());

                        let s: Vec<_> =
                            target.iter()
                            .map(|off| (s as isize + off) as usize)
                            .collect();

                        let mut e_c = self.epsilon_closure(&s);

                        cur_states.append(&mut e_c);
                        cur_states.sort();
                        cur_states.dedup();
                    }
                }
            }
        }

        m_s
    }

    /// Returns `None` if `state_idx` is non-accepting
    pub fn accepting(&self, state_idx: usize) -> Option<String> {
        if let Some(state) = self.states.get(state_idx) {
            state.accepting()
        } else {
            None
        }
    }
}

impl fmt::Debug for Nfa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (state_idx, state) in self.states.iter().enumerate() {
            match state.accepting() {
                Some(desc) => try!(writeln!(f, "(({})) `{}`:", state_idx, desc)),
                None => try!(writeln!(f, "({}):", state_idx)),
            }

            for (&transition, target) in state.move_map() {
                let transition =
                    match transition {
                        Input(c) => c,
                        Epsilon => 'ε',
                    };

                try!(write!(f, "    {} ->", transition));
                for t in target {
                    try!(write!(f, " {}", state_idx as isize + t));
                }
                try!(writeln!(f, ""));
            }
        }
        Ok(())
    }
}

/// An `Option`-like enum holding a state transition
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Transition {
    /// Transition on some input character
    Input(char),
    /// ε-transition, doens't consume any input
    Epsilon,
}

use self::Transition::{Input, Epsilon};
