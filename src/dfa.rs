//! Deterministic Finite Automaton (DFA) implementation.

use nfa::Nfa;

pub struct Dfa {
    states: Vec<usize>,
}

impl Dfa {
    pub fn from_nfa(nfa: &Nfa) -> Dfa {
        let start_states = nfa.epsilon_closure(&[0]);

        for s in &start_states {
            println!("ε-closure: {}", s);
        }

        unimplemented!();
    }
}
