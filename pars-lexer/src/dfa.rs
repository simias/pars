//! Deterministic Finite Automaton (DFA) implementation.

use nfa::Nfa;
use std::collections::BTreeMap;
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

    /// Returns `true` if this state has at least one move on some
    /// input.
    pub fn has_moves(&self) -> bool {
        return !self.moves.is_empty()
    }

    pub fn accepting(&self)-> Option<&String> {
        self.accepting.as_ref()
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
            let move_set = nfa.get_move_set(&dfa_states[cur_state].nfa_states);

            println!("Pre:");
            for (&transition, states) in &move_set {
                print!("{:?} -> ", transition);
                for s in states {
                    print!("{} ", s);
                }
                println!("");
            }

            let move_set = Dfa::resolve_intersections(move_set);

            println!("Post:");
            for (&transition, states) in &move_set {
                print!("{:?} -> ", transition);
                for s in states {
                    print!("{} ", s);
                }
                println!("");
            }

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
        Dfa {
            states: dfa_states.into_iter().map(|s| s.dfa_state).collect()
        }
    }

    // In order for the DFA to be deterministic we must have exactly
    // one move per input character. That means that we must be
    // careful with character intervals: they can't be
    // overlapping. For instance if we have a move on `[a]` and a move
    // on `[a-z]` we wouldn't know which one to use when matching a
    // character 'a' in the input stream.
    fn resolve_intersections(mut move_set: BTreeMap<Interval, Vec<usize>>)
                             -> BTreeMap<Interval, Vec<usize>> {
        // Since we only attempt to resolve intersections two
        // intervals at a time it's possible we might miss some
        // intersections during the first pass, for instance in the
        // case of several nested intervals (i.e. if `n` intersects
        // `n+1` it's also possible that it intersects `n+2` and
        // others after that). To keep things simple I simply repeat
        // the algorithm again and again until no intersections are
        // left.
        let mut dirty = true;

        while dirty {
            dirty = false;

            let mut out = BTreeMap::new();

            let mut iter = move_set.into_iter();

            // Since the intervals are sorted in the map we know that
            // any intersecting intervals have to be
            // contiguous. i.e. if entry `n` doesn't intersect with
            // `n+1` we're sure it won't intersect with anything else
            // after that.
            if let Some((mut interval, mut states)) = iter.next() {
                while let Some((next_interval, next_states)) = iter.next() {
                    let (left, inter, right) =
                        interval.intersect(next_interval);

                    println!("{:?},{:?} {:?} {:?} {:?}", interval, next_interval, left, inter, right);

                    if let Some(inter) = inter {

                        // Intervals intersect, we need to "split" them
                        // into a subset of mutually-exclusive intervals.
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
                        let mut union: Vec<_> =
                            states.iter()
                            .chain(next_states.iter())
                            .map(|&u| u).collect();

                        union.sort();
                        union.dedup();

                        if let Some(left) = left {
                            out.insert(left, states.clone());
                        }

                        out.insert(inter, union.clone());

                        if let Some(right) = right {
                            // The right part can belong either to
                            // `interval` or `next_interval` depending
                            // on which extends further.
                            states =
                                if interval.last() > next_interval.last() {
                                    states
                                } else {
                                    next_states
                                };

                            out.insert(right, states.clone());

                            interval = right;
                        } else {
                            interval = inter;
                            states = union;
                        }

                        dirty = true;
                    } else {
                        // Interval doesn't intersect, we can use it in
                        // the DFA without modification
                        out.insert(interval, states);
                        interval = next_interval;
                        states = next_states;
                    }
                }

                // Make sure we don't lose any state
                out.insert(interval, states);
            }

            move_set = out;
        }

        move_set
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
