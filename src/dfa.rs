//! Deterministic Finite Automaton (DFA) implementation.

use nfa::Nfa;
use std::collections::HashMap;
use std::fmt;

pub struct Dfa {
    states: Vec<HashMap<char, usize>>,
}

impl Dfa {
    pub fn from_nfa(nfa: &Nfa) -> Dfa {
        // We need a temporary state holding the correspondance
        // between each DFA state and the corresponding NFA states
        struct DState {
            nfa_states: Vec<usize>,
            transitions: HashMap<char, usize>,
        };

        impl DState {
            fn from_nfa_states(mut n_s: Vec<usize>) -> DState {
                n_s.sort();
                n_s.dedup();

                DState {
                    nfa_states: n_s,
                    transitions: HashMap::new(),
                }
            }
        }

        // We start from ε-closure of state (0) of the NFA and work
        // our way through recursively.
        let mut dfa_states = vec![DState::from_nfa_states(nfa.epsilon_closure(&[0]))];

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
                            dfa_states.push(DState::from_nfa_states(states.clone()));
                            dfa_states.len() - 1
                        }
                    };

                dfa_states[cur_state].transitions.insert(transition, target);
            }

            cur_state += 1;
        }

        // The conversion is done, we can drop the NFA states
        // altogether
        Dfa {
            states: dfa_states.into_iter().map(|s| s.transitions).collect()
        }
    }
}

impl fmt::Debug for Dfa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (state, transitions) in self.states.iter().enumerate() {
            try!(writeln!(f, "({}):", state));
            for (c, target) in transitions {
                try!(writeln!(f, "  {} -> {}", c, target));
            }
        }
        Ok(())
    }
}
