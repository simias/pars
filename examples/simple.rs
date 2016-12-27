extern crate pars;

use std::fs::File;

use pars::nfa::Nfa;
use pars::dfa::Dfa;
use pars::codegen::CodeGen;

fn main() {
    // (a|b)*abb
    let mut nfa = Nfa::new('a');
    nfa.union(Nfa::new('b'));
    nfa.star();


    nfa.concat(Nfa::new('a'));
    nfa.concat(Nfa::new('b'));
    nfa.concat(Nfa::new('b'));

    nfa.concat(Nfa::new_accepting("got (a|b)*abb".into()));

    // abc
    let mut other = Nfa::new('a');
    other.concat(Nfa::new('b'));
    other.concat(Nfa::new('c'));
    other.concat(Nfa::new_accepting("got abc".into()));

    nfa.combine(other);

    let dfa = Dfa::from_nfa(&nfa);

    println!("{:?}", nfa);

    println!("{:?}", dfa);

    let mut out = File::create("/tmp/lexer.rs").unwrap();

    let gen = CodeGen::new();

    gen.generate(&dfa, &mut out).unwrap();
}
