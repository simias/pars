use std::io::Write;
use std::io;

use dfa::Dfa;

/// Code generator
pub struct CodeGen {
    /// Return type of the lexer's `next_token` method. Replaces
    /// `%TOKEN_TYPE%` in the template. Defaults to `Token`.
    token_type: String,
}


impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            token_type: "Token".into(),
        }
    }

    pub fn set_token_type(&mut self, t: &str) {
        self.token_type = t.into()
    }

    pub fn generate(&self, dfa: &Dfa, output: &mut Write) -> io::Result<()> {
        let states = dfa.states();

        assert!(!states.is_empty());

        let mut code = include_str!("lexer.rs.in").to_owned();

        self.template_replace(&mut code, "%TOKEN_TYPE%", &self.token_type);

        let mut states_decl = String::new();

        for (i, _) in states.iter().enumerate() {
            states_decl.push_str(&format!("\n    State{},", i));
        }

        self.template_replace(&mut code, "%DECLARE_STATES%", &states_decl);

        // Declare accepting states
        states_decl.clear();

        for (i, s) in states.iter().enumerate() {
            if s.accepting().is_some() {
                states_decl.push_str(&format!("\n    State{},", i));
            }
        }

        self.template_replace(&mut code,
                              "%DECLARE_ACCEPTING_STATES%",
                              &states_decl);

        let matcher = self.generate_matcher(dfa);

        self.template_replace(&mut code, "%MATCH_INPUT%", &matcher);

        let accepting_matcher = self.generate_accepting_matcher(dfa);

        self.template_replace(&mut code,
                              "%MATCH_ACCEPTING_STATE%",
                              &accepting_matcher);

        output.write_all(code.as_bytes())
    }

    /// This is where the magic happens: we generate the actual state
    /// machine used for matching the input. Will replace
    /// `%MATCH_INPUT%` in the template.
    fn generate_matcher(&self, dfa: &Dfa) -> String {
        let mut matcher = String::new();

        for (state_idx, state) in dfa.states().iter().enumerate() {
            matcher.push_str(&format!("\nState::State{} => {{\n", state_idx));

            matcher.push_str("match input as u32 {\n");

            for (c, &target) in state.move_map() {

                let first = c.first();
                let last = c.last();

                if first == last {
                    matcher.push_str(&format!("0x{:x} => {{\n", first));
                } else {
                    matcher.push_str(&format!("0x{:x}...0x{:x} => {{\n",
                                              first, last));
                }

                if dfa.states()[target].is_accepting() {
                    matcher.push_str(&format!(
                        "accepting_state = Some((self.buffer_offset, \
                                                 AcceptingState::State{}));\n",
                        target));
                }

                matcher.push_str(&format!("Some(State::State{})\n", target));
                matcher.push_str("}\n");
            }

            matcher.push_str("_ => None,\n");

            matcher.push_str("}\n");
            matcher.push_str("}");
        }

        matcher
    }

    /// Generate the code that will run when an accepting state has
    /// been found. This will run the action code associated with the
    /// token.
    fn generate_accepting_matcher(&self, dfa: &Dfa) -> String {
        let mut matcher = String::new();

        let accepting_states = dfa.states().iter()
            .enumerate()
            .filter_map(|(i, s)| {
                s.accepting().map(|s| (i, s))
            });

        for (state_idx, action_code) in accepting_states {

            matcher.push_str(&format!("\nAcceptingState::State{} => {{\n",
                                      state_idx));

            matcher.push_str(action_code);

            matcher.push_str("\n}\n");
        }

        matcher
    }

    fn template_replace(&self, code: &mut String, template: &str, val: &str) {
        while let Some(m) = code.find(template) {
            code.drain(m..(m + template.len()));

            // XXX Replace with `String::insert_str` when it's
            // stabilized
            for c in val.chars().rev() {
                code.insert(m, c);
            }
        }
    }
}
