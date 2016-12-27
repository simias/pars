//! Deterministic Finite Automaton (DFA) implementation.

use nfa::Nfa;
use std::collections::HashMap;
use std::fmt;

struct State {
    moves: HashMap<char, usize>,
    accepting: Option<String>,
}

impl State {
    /// Create a new non-accepting state with no moves
    fn new() -> State {
        State {
            moves: HashMap::new(),
            accepting: None,
        }
    }

    /// Create a new accepting state with no moves
    fn new_accepting(desc: String) -> State {
        State {
            moves: HashMap::new(),
            accepting: Some(desc),
        }
    }

    fn set_move(&mut self, input: char, target: usize) {
        self.moves.insert(input, target);
    }

    fn move_map(&self) -> &HashMap<char, usize> {
        &self.moves
    }
}

pub struct Dfa {
    states: Vec<State>,
}

impl Dfa {
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
                        if let Some(p) = accepting {
                            // XXX should have a priority paremeter
                            // here (I believe the rule in lex is that
                            // the first rule in the file takes
                            // precedence in this case)
                            panic!("DFA state has two accepting NFA states: \
                                    {} and {}", p, a);
                        }

                        accepting = Some(a);
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
}

impl fmt::Debug for Dfa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (state_idx, state) in self.states.iter().enumerate() {
            match state.accepting {
                Some(ref a) => try!(writeln!(f, "(({})) `{}`:", state_idx, a)),
                None => try!(writeln!(f, "({}):", state_idx)),
            }
            for (c, target) in state.move_map() {
                try!(writeln!(f, "  {} -> {}", c, target));
            }
        }
        Ok(())
    }
}
