//! Nondeterministic Finite Automaton (NFA) implementation.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use character::Interval;

/// NFA state
#[derive(Clone)]
struct State {
    /// Map of all valid moves from this state. The target state is
    /// stored as a signed offset in the
    moves: BTreeMap<Transition, Vec<isize>>,
    /// `None` if the state is non-accepting
    accepting: Option<String>,
}

impl State {
    /// Creates a new non-accepting state with no moves
    fn new() -> State {
        State {
            moves: BTreeMap::new(),
            accepting: None,
        }
    }

    /// Creates a new accepting state with no moves
    fn new_accepting(desc: String) -> State {
        State {
            moves: BTreeMap::new(),
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

    /// Add a moves on `input`.
    fn add_move(&mut self, input: Transition, target: isize) {
        let m = self.moves.entry(input).or_insert(Vec::new());

        m.push(target)
    }


    /// Returns the moves on `input`
    fn get_moves(&self, input: Transition) -> &[isize] {
        match self.moves.get(&input) {
            Some(v) => v,
            None => &[],
        }
    }

    fn move_map(&self) -> &BTreeMap<Transition, Vec<isize>> {
        &self.moves
    }
}

/// NFA graph
#[derive(Clone)]
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
    pub fn new(i: Interval) -> Nfa {
        let mut state = State::new();

        state.set_moves(Input(i), vec![1]);

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

    /// Creates a new NFA with no states
    ///
    /// ```text
    /// (0)
    /// ```
    pub fn new_empty() -> Nfa {
        Nfa {
            states: VecDeque::new(),
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

    /// Compute the positive closure of this NFA. `a.positive()` matches
    /// `a+` or `aa*`.
    ///
    /// ```text
    ///                    ε
    ///               ,--------.
    ///        ε     v     a    \      ε
    /// (0) ------> (1) ------> (2) ------> (f)
    /// ```
    pub fn positive(&mut self) {
        // Create state (2) and have it point at the new final state
        // (f) and the current first state (1)
        let mut state_2 = State::new();

        state_2.set_moves(Epsilon, vec![1, -(self.states.len() as isize)]);

        self.states.push_back(state_2);

        // Create state (0) pointing at (1)
        let mut state_0 = State::new();

        state_0.set_moves(Epsilon, vec![1]);

        self.states.push_front(state_0);
    }

    /// Combines two NFAs by adding an ε-transition between the first
    /// state of `self` and the first state of `other`:
    ///
    /// ```text
    /// self(0) -----> ...
    ///    \    ε
    ///     `------> other(0) ------> ...
    ///
    /// ```
    ///
    /// This is useful for combining several NFAs with accepting state
    /// in order to attempt to match them all at once. This should
    /// *not* be used with non-accepting NFAs since it would result in
    /// a bogus NFA.
    ///
    /// For this reason this method `panic`s if `self` or `other`
    /// don't end with an accepting state.
    ///
    /// While matching if a string leads to two accepting state then
    /// the first one in combination order is used:
    ///
    /// ```rust
    /// use pars_lexer::character::Interval;
    /// use pars_lexer::nfa::Nfa;
    ///
    /// // Accepts [a-z]+
    /// let mut lower = Nfa::new(Interval::new('a', 'z'));
    /// lower.positive();
    /// lower.concat(Nfa::new_accepting("found lowercase".into()));
    ///
    /// // Accepts [a-zA-Z]+
    /// let mut mixed = Nfa::new(Interval::new('a', 'z'));
    /// mixed.union(Nfa::new(Interval::new('A', 'Z')));
    /// mixed.positive();
    /// mixed.concat(Nfa::new_accepting("found mixed case".into()));
    ///
    /// // Clearly both regexes above accept the string "lowercase"
    /// // since `lower` is a subset of `mixed`. In this situation the
    /// // combination order matters:
    ///
    /// // If `mixed` is combined before `lower` then it takes precedence,
    /// // therefore both the strings "lowercase" and "MixedCase" will end
    /// // up being accepted by `mixed`. `lower` will never match since
    /// // it's a subset of `mixed`.
    /// let mixed_first = mixed.clone().combine(lower.clone());
    ///
    /// // On the other hand if we combine `lower` first then it will
    /// // take precedence in case of a "double match", so "lowercase"
    /// // will match `lower` while "MixedCase" will naturally still
    /// // match `mixed`.
    /// let lower_first = lower.clone().combine(mixed.clone());
    /// ```
    pub fn combine(&mut self, mut other: Nfa) {
        assert!(self.is_accepting());
        assert!(other.is_accepting());

        // Add the ε-transition to the start of other
        let other_start = self.states.len() as isize;

        self.states[0].add_move(Epsilon, other_start);

        // Copy the states of other into self
        self.states.append(&mut other.states);
    }

    /// Return true if `self` finishes with an accepting state.
    pub fn is_accepting(&self) -> bool {
        match self.states.back() {
            Some(ref s) => s.accepting().is_some(),
            None => false
        }
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
    /// `get_move_set(&[0])` would return a `BTreeMap` containing a
    /// single entry: `'a' -> [2, 3]`.
    ///
    /// `get_move_set(&[0, 1, 6])` (the ε-closure of state 0) would
    /// return a `BTreeMap` with 3 entries:
    ///
    /// ```text
    /// 'a' -> [2, 3]
    /// 'c' -> [5, 7]
    /// 'd' -> [7]
    /// ```
    pub fn get_move_set(&self, states: &[usize]) -> BTreeMap<Interval, Vec<usize>> {
        let mut m_s = BTreeMap::new();

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
                try!(write!(f, "    {:?} ->", transition));
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
#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum Transition {
    /// Transition on some input character
    Input(Interval),
    /// ε-transition, doens't consume any input
    Epsilon,
}

use self::Transition::{Input, Epsilon};

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Input(i) => write!(f, "{:?}", i),
            &Epsilon => write!(f, "ε"),
        }
    }
}
