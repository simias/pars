extern crate pars_lexer;

use std::fs::File;
use std::env;
use std::path::Path;

use pars_lexer::nfa::Nfa;
use pars_lexer::dfa::Dfa;
use pars_lexer::codegen::CodeGen;
use pars_lexer::character::Interval;

fn main() {
    simple();
    intervals();
    intersecting_intervals();
}

pub fn simple() {
    let a = Interval::new_single('a');
    let b = Interval::new_single('b');
    let c = Interval::new_single('c');

    // (a|b)*abb
    let mut nfa = Nfa::new(a);
    nfa.union(Nfa::new(b));
    nfa.star();

    nfa.concat(Nfa::new(a));
    nfa.concat(Nfa::new(b));
    nfa.concat(Nfa::new(b));

    nfa.concat(Nfa::new_accepting("regex (a|b)*abb".into()));

    // abc
    let mut other = Nfa::new(a);
    other.concat(Nfa::new(b));
    other.concat(Nfa::new(c));
    other.concat(Nfa::new_accepting("regex abc".into()));

    nfa.combine(other);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("simple.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("String");

    gen.generate(&dfa, &mut out).unwrap();
}

pub fn intervals() {
    let az = Interval::new('a', 'z');
    let az_u = Interval::new('A', 'Z');
    let us = Interval::new_single('_');
    let num = Interval::new('0', '9');

    // [a-zA-Z_][a-zA-Z_0-9]*
    let mut alpha = Nfa::new(az);
    alpha.union(Nfa::new(az_u));
    alpha.union(Nfa::new(us));

    let mut id = alpha.clone();

    let mut alnum = alpha;
    alnum.union(Nfa::new(num));
    alnum.star();

    id.concat(alnum);
    id.concat(Nfa::new_accepting("id".into()));

    // [ ]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.positive();
    spaces.concat(Nfa::new_accepting("space".into()));

    id.combine(spaces);

    let dfa = Dfa::from_nfa(&id);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("intervals.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("String");

    gen.generate(&dfa, &mut out).unwrap();
}

pub fn intersecting_intervals() {
    let mut az = Nfa::new(Interval::new('a', 'z'));
    az.positive();
    az.concat(Nfa::new_accepting("az".into()));

    let mut ae = Nfa::new(Interval::new('a', 'e'));
    ae.positive();
    ae.concat(Nfa::new_accepting("ae".into()));

    let mut cz = Nfa::new(Interval::new('c', 'z'));
    cz.positive();
    cz.concat(Nfa::new_accepting("cz".into()));

    let mut bd = Nfa::new(Interval::new('b', 'd'));
    bd.positive();
    bd.concat(Nfa::new_accepting("bd".into()));

    // [ ]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.positive();
    spaces.concat(Nfa::new_accepting("space".into()));

    let mut nfa = bd;
    nfa.combine(ae);
    nfa.combine(cz);
    nfa.combine(az);
    nfa.combine(spaces);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("intersecting-intervals.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("String");

    gen.generate(&dfa, &mut out).unwrap();
}
