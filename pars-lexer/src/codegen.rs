use std::io::Write;
use std::io;

use dfa::Dfa;

/// Code generator
pub struct CodeGen {
    /// Return type of the lexer's `next_token` method. Replaces
    /// `%TOKEN_TYPE%` in the template. Defaults to `Token`.
    token_type: String,
    /// A list of additional parameters to the `next_token`
    /// method. Used to pass additional state if necessary. Defaults
    /// to empty.
    next_params: Vec<String>,
}


impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            token_type: "Token".into(),
            next_params: Vec::new(),
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

        let next_params: String = self.next_params.iter()
            .fold(String::new(), |mut s, p| {
                s.push_str(", ");
                s.push_str(p);
                s
            });

        // Just because I don't like having the leading space in the
        // generated method signature
        self.template_replace(&mut code, " %NEXT_PARAMS%", &next_params);
        self.template_replace(&mut code, "%NEXT_PARAMS%", &next_params);

        let mut states_decl = String::new();

        for (i, _) in states.iter().enumerate() {
            states_decl.push_str(&format!("\n    State{},", i));
        }

        self.template_replace(&mut code, "%DECLARE_STATES%", &states_decl);

        let matcher = self.generate_matcher(dfa);

        self.template_replace(&mut code, "%MATCH_INPUT%", &matcher);

        output.write_all(code.as_bytes())
    }

    /// This is where the magic happens: we generate the actual state
    /// machine used for matching the input. Will replace
    /// `%MATCH_INPUT%` in the template.
    fn generate_matcher(&self, dfa: &Dfa) -> String {
        let mut matcher = String::new();

        for (state_idx, state) in dfa.states().iter().enumerate() {
            matcher.push_str(&format!("\nStates::State{} => {{\n", state_idx));

            matcher.push_str("match input {\n");

            for (c, &target) in state.move_map() {

                matcher.push_str(&format!("'{}' => {{\n", c));

                if let Some(s) = dfa.states()[target].accepting() {
                    matcher.push_str(&format!(
                        "accepting_state = Some((self.buffer_offset, \"{}\".into()));\n",
                        s));
                }

                matcher.push_str(&format!("Some(States::State{})\n", target));
                matcher.push_str("}\n");
            }

            matcher.push_str("_ => None,\n");

            matcher.push_str("}\n");
            matcher.push_str("}");
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
