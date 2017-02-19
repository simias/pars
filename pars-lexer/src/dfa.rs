//! Deterministic Finite Automaton (DFA) implementation.

use nfa::Nfa;
use std::collections::BTreeMap;
use std::collections::btree_map::Keys;
use std::fmt;

use character::Interval;

pub struct State {
    moves: BTreeMap<Interval, usize>,
    accepting: Option<String>,
}

impl State {
    /// Create a new non-accepting state with no moves
    fn new() -> State {
        State {
            moves: BTreeMap::new(),
            accepting: None,
        }
    }

    /// Create a new accepting state with no moves
    fn new_accepting(desc: String) -> State {
        State {
            moves: BTreeMap::new(),
            accepting: Some(desc),
        }
    }

    fn set_move(&mut self, input: Interval, target: usize) {
        self.moves.insert(input, target);
    }

    pub fn move_map(&self) -> &BTreeMap<Interval, usize> {
        &self.moves
    }

    /// Return the set of intervals for which this state has a move.
    pub fn move_intervals(&self) -> Keys<Interval, usize> {
        self.moves.keys()
    }

    /// Returns `true` if this state has at least one move on some
    /// input.
    pub fn has_moves(&self) -> bool {
        return !self.moves.is_empty()
    }

    pub fn accepting(&self)-> Option<&String> {
        self.accepting.as_ref()
    }

    /// Return `true` if this is an accepting state
    pub fn is_accepting(&self) -> bool {
        self.accepting.is_some()
    }
}

pub struct Dfa {
    states: Vec<State>,
}

impl Dfa {
    /// Builds a DFA from the provided NFA.
    ///
    /// If the NFA is valid this can't fail, however the DFA can have
    /// up to the square of the number of states of the NFA in the
    /// worst case.
    pub fn from_nfa(nfa: &Nfa) -> Dfa {
        // We need a temporary state holding the correspondance
        // between each DFA state and the corresponding NFA states
        struct DState {
            nfa_states: Vec<usize>,
            dfa_state: State,
        };

        impl DState {
            fn from_nfa_states(nfa: &Nfa, mut n_s: Vec<usize>) -> DState {
                n_s.sort();
                n_s.dedup();

                // Check if we have an accepting state
                let mut accepting = None;

                for &s in &n_s {
                    if let Some(a) = nfa.accepting(s) {
                        accepting = Some(a);
                        // The first accepting state found takes
                        // priority over any subsequent one so it's no
                        // use searching further.
                        break;
                    }
                }

                let dfa_state =
                    match accepting {
                        Some(a) => State::new_accepting(a),
                        None => State::new(),
                    };

                DState {
                    nfa_states: n_s,
                    dfa_state: dfa_state,
                }
            }
        }

        // We start from ε-closure of state (0) of the NFA and work
        // our way through recursively.
        let epsi_0 = nfa.epsilon_closure(&[0]);
        let mut dfa_states = vec![DState::from_nfa_states(nfa, epsi_0)];

        let mut cur_state = 0;

        while cur_state < dfa_states.len() {
            // We want to know all the NFA states we can reach from
            // `state.nfa_states`.
            let mut move_set =
                nfa.get_move_set(&dfa_states[cur_state].nfa_states);

            Dfa::resolve_intersections(&mut move_set);

            for (&transition, states) in &move_set {
                // See if we already have a DFA state for this set of
                // NFA states, otherwise create it.
                let target =
                    match dfa_states.iter()
                    .enumerate()
                    .find(|&(_, s)| s.nfa_states == states.as_slice()) {
                        Some((pos, _)) => pos,
                        None => {
                            // Create a new DFA state
                            let states = states.clone();

                            dfa_states.push(DState::from_nfa_states(nfa,
                                                                    states));
                            dfa_states.len() - 1
                        }
                    };

                dfa_states[cur_state].dfa_state.set_move(transition, target);
            }

            cur_state += 1;
        }

        // The conversion is done, we can drop the NFA states
        // altogether
        let mut dfa =
            Dfa {
                states: dfa_states.into_iter().map(|s| s.dfa_state).collect()
            };

        dfa.optimize();

        dfa
    }

    /// Optimize the DFA by factoring equivalent states.
    fn optimize(&mut self) {
        // First we partition the states to isolate the accepting
        // states.
        let mut accepting = Vec::new();
        let mut non_accepting = Vec::new();

        for (i, s) in self.states.iter().enumerate() {
            let g =
                if s.is_accepting() {
                    &mut accepting
                } else {
                    &mut non_accepting
                };

            g.push(i)
        }

        let mut partition = vec![accepting, non_accepting];

        let mut needs_loop = true;

        while needs_loop {
            needs_loop = false;

            let mut next_partition: Vec<Vec<usize>> = Vec::new();
            let mut subgroup_start;

            for group in &partition {

                subgroup_start = next_partition.len();

                for &state_idx in group {
                    let state = &self.states[state_idx];

                    // See if this state fits in any of our current
                    // subgroups
                    let mut found_equiv = false;

                    for sub in &mut next_partition[subgroup_start..] {
                        // Pick the first state in the subgroup as a
                        // representative
                        let &s = sub.first().unwrap();

                        let sub_state = &self.states[s];

                        let equivalent =
                            if sub_state.move_intervals().ne(state.move_intervals()) {
                                false
                            } else {
                                let mut equivalent = true;

                                // Both states move on the same keys, they
                                // could be equivalent
                                for (i, &s) in sub_state.move_map() {
                                    println!("Looking for {:?}", i);
                                    let other = state.move_map()[i];

                                    // XXX fixme
                                    if other != s {
                                        equivalent = false;
                                        break;
                                    }
                                }

                                equivalent
                            };

                        if equivalent {
                            // We can push to this subgroup
                            sub.push(state_idx);
                            found_equiv = true;
                            break;
                        }
                    }

                    if !found_equiv {
                        // Looks like we need a new subgroup
                        next_partition.push(vec![state_idx]);
                    }
                }
            }

            needs_loop = partition.len() != next_partition.len();

            partition = next_partition
        }


        for p in partition {
            println!("Partition:");
            for i in p {
                println!("{}", i);
            }
        }
    }

    /// In order for the DFA to be deterministic we must have exactly
    /// one move per input character. That means that we must be
    /// careful with character intervals: they can't be
    /// overlapping. For instance if we have a move on `[a]` and a
    /// move on `[a-z]` we wouldn't know which one to use when
    /// matching a character 'a' in the input stream.
    fn resolve_intersections(move_set: &mut BTreeMap<Interval, Vec<usize>>) {

        while let Some((a, b)) = Dfa::find_intersection(move_set) {
            // Intervals intersect, we need to "split" them into a
            // subset of mutually-exclusive intervals.
            //
            // For instance if we have:
            //   [0-5] => (1, 2, 3)
            //   [2-8] => (2, 4)
            //
            // We must handle the intersection on input [2-5]
            // by creating:
            //
            //   [0-1] => (1, 2, 3)    # Part exclusize to [0-5]
            //   [2-5] => (1, 2, 3, 4) # Intersection
            //   [6-8] => (2, 4)       # Part exclusive to [2-8]
            let a_states = move_set.remove(&a).unwrap();
            let b_states = move_set.remove(&b).unwrap();

            let mut ab_states: Vec<_> =
                a_states.iter()
                .chain(b_states.iter())
                .map(|&u| u).collect();

            ab_states.sort();
            ab_states.dedup();

            let (left, inter, right) = a.intersect(b);

            // a and b are known to intersect, inter can't be `None`.
            let inter = inter.unwrap();

            move_set.insert(inter, ab_states);

            if let Some(right) = right {
                // The right part can belong either to `a` or `b`
                // depending on which one extends further.
                let states =
                    if a.last() > b.last() {
                        a_states.clone()
                    } else {
                        b_states
                    };

                move_set.insert(right, states);
            }

            if let Some(left) = left {
                move_set.insert(left, a_states);
            }
        }
    }

    /// Return the first pair of intervals that intersect or `None` if
    /// all intervals are inclusive
    fn find_intersection(move_set: &BTreeMap<Interval, Vec<usize>>)
                         -> Option<(Interval, Interval)> {
        let mut iter = move_set.keys();

        // Since the intervals are sorted in the map we know that
        // intersecting intervals have to be contiguous when
        // iterating. So we can just check them two at a time.
        if let Some(&first) = iter.next() {
            let mut a = first;

            while let Some(&b) = iter.next() {
                if a.intersects(b) {
                    return Some((a, b));
                }

                a = b
            }
        }

        None
    }

    /// Returns the vector of states of this DFA
    pub fn states(&self) -> &Vec<State> {
        &self.states
    }
}

impl fmt::Debug for Dfa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (state_idx, state) in self.states.iter().enumerate() {
            match state.accepting {
                Some(ref a) => try!(writeln!(f, "(({})) `{}`:", state_idx, a)),
                None => try!(writeln!(f, "({}):", state_idx)),
            }
            for (c, target) in state.move_map() {
                try!(writeln!(f, "  {:?} -> {}", c, target));
            }
        }
        Ok(())
    }
}
