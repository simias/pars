extern crate pars_lexer;

use std::fs::File;
use std::env;
use std::path::Path;

use pars_lexer::nfa::Nfa;
use pars_lexer::dfa::Dfa;
use pars_lexer::codegen::CodeGen;

fn main() {
    // (a|b)*abb
    let mut nfa = Nfa::new('a');
    nfa.union(Nfa::new('b'));
    nfa.star();


    nfa.concat(Nfa::new('a'));
    nfa.concat(Nfa::new('b'));
    nfa.concat(Nfa::new('b'));

    nfa.concat(Nfa::new_accepting("regex (a|b)*abb".into()));

    // abc
    let mut other = Nfa::new('a');
    other.concat(Nfa::new('b'));
    other.concat(Nfa::new('c'));
    other.concat(Nfa::new_accepting("regex abc".into()));

    nfa.combine(other);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("simple.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("String");

    gen.generate(&dfa, &mut out).unwrap();
}
