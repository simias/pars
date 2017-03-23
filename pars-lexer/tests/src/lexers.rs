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
    utf8();
    c_basic();
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

    nfa.concat(Nfa::new_accepting(stringify!({
        let m = _lexer_match.as_str().into_owned();

        Some(Token::Abaab(m))
    }).into()));

    // abc
    let mut other = Nfa::new(a);
    other.concat(Nfa::new(b));
    other.concat(Nfa::new(c));
    other.concat(Nfa::new_accepting(stringify!({
        let m = _lexer_match.as_str().into_owned();

        Some(Token::Abc(m))
    }).into()));

    nfa.combine(other);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("simple.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("Token");

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
    id.concat(Nfa::new_accepting(stringify!({
        let id = _lexer_match.as_str().into_owned();

        Some(id)
    }).into()));

    // [ ]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.positive();
    spaces.concat(Nfa::new_accepting(stringify!({
        None
    }).into()));

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
    az.concat(Nfa::new_accepting(stringify!({
        Some(Token::Az)
    }).into()));

    let mut ae = Nfa::new(Interval::new('a', 'e'));
    ae.positive();
    ae.concat(Nfa::new_accepting(stringify!({
        Some(Token::Ae)
    }).into()));

    let mut cz = Nfa::new(Interval::new('c', 'z'));
    cz.positive();
    cz.concat(Nfa::new_accepting(stringify!({
        Some(Token::Cz)
    }).into()));

    let mut bd = Nfa::new(Interval::new('b', 'd'));
    bd.positive();
    bd.concat(Nfa::new_accepting(stringify!({
        Some(Token::Bd)
    }).into()));

    // [ ]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.positive();
    spaces.concat(Nfa::new_accepting(stringify!({
        None
    }).into()));

    let mut nfa = bd;
    nfa.combine(ae);
    nfa.combine(cz);
    nfa.combine(az);
    nfa.combine(spaces);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("intersecting-intervals.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("Token");

    gen.generate(&dfa, &mut out).unwrap();
}

pub fn utf8() {
    // [a-z]+
    let mut en = Nfa::new(Interval::new('a', 'z'));
    en.positive();
    en.concat(Nfa::new_accepting(stringify!({
        Some(Token::English)
    }).into()));

    // [a-ya]+
    let mut ru = Nfa::new(Interval::new('\u{0430}', '\u{044f}'));
    ru.positive();
    ru.concat(Nfa::new_accepting(stringify!({
        Some(Token::Russian)
    }).into()));

    // [ ]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.positive();
    spaces.concat(Nfa::new_accepting(stringify!({
        None
    }).into()));

    let mut nfa = en;
    nfa.combine(ru);
    nfa.combine(spaces);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("utf8.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("Token");

    gen.generate(&dfa, &mut out).unwrap();
}

pub fn c_basic() {
    let lower = Nfa::new(Interval::new('a', 'z'));
    let upper = Nfa::new(Interval::new('A', 'Z'));
    let num   = Nfa::new(Interval::new('0', '9'));
    let under = Nfa::new(Interval::new_single('_'));

    // [0-9]+
    let mut number = num.clone();
    number.positive();
    number.concat(Nfa::new_accepting(stringify!({
        use std::str::FromStr;

        let n = i64::from_str(&_lexer_match.as_str()).unwrap();

        Some(Token::Number(n))
    }).into()));

    // [a-zA-Z_][a-zA-Z_0-9]*
    let mut id = lower.clone();
    id.union(upper.clone());
    id.union(under.clone());

    let mut rest = id.clone();
    rest.union(num);
    rest.star();

    id.concat(rest);
    id.concat(Nfa::new_accepting(stringify!({
        let id = _lexer_match.as_str().into_owned();
        Some(Token::Id(id))
    }).into()));

    // [ \t\n]+
    let mut spaces = Nfa::new(Interval::new_single(' '));
    spaces.union(Nfa::new(Interval::new_single('\t')));
    spaces.union(Nfa::new(Interval::new_single('\n')));
    spaces.positive();
    spaces.concat(Nfa::new_accepting(stringify!({
        None
    }).into()));

    fn symbol(s: &str) -> Nfa {
        let mut nfa = Nfa::new_empty();

        for c in s.chars() {
            nfa.concat(Nfa::new(Interval::new_single(c)));
        }

        nfa.concat(Nfa::new_accepting(stringify!({
            let s = _lexer_match.as_str().into_owned();
            Some(Token::Symbol(s))
        }).into()));

        nfa
    }

    let mut nfa = spaces;
    nfa.combine(symbol("auto"));
    nfa.combine(symbol("break"));
    nfa.combine(symbol("case"));
    nfa.combine(symbol("char"));
    nfa.combine(symbol("const"));
    nfa.combine(symbol("continue"));
    nfa.combine(symbol("default"));
    nfa.combine(symbol("do"));
    nfa.combine(symbol("double"));
    nfa.combine(symbol("else"));
    nfa.combine(symbol("enum"));
    nfa.combine(symbol("extern"));
    nfa.combine(symbol("float"));
    nfa.combine(symbol("for"));
    nfa.combine(symbol("goto"));
    nfa.combine(symbol("if"));
    nfa.combine(symbol("int"));
    nfa.combine(symbol("long"));
    nfa.combine(symbol("register"));
    nfa.combine(symbol("return"));
    nfa.combine(symbol("short"));
    nfa.combine(symbol("signed"));
    nfa.combine(symbol("sizeof"));
    nfa.combine(symbol("static"));
    nfa.combine(symbol("struct"));
    nfa.combine(symbol("switch"));
    nfa.combine(symbol("typedef"));
    nfa.combine(symbol("union"));
    nfa.combine(symbol("unsigned"));
    nfa.combine(symbol("void"));
    nfa.combine(symbol("volatile"));
    nfa.combine(symbol("while"));
    nfa.combine(symbol("..."));
    nfa.combine(symbol(">>="));
    nfa.combine(symbol("<<="));
    nfa.combine(symbol("+="));
    nfa.combine(symbol("-="));
    nfa.combine(symbol("*="));
    nfa.combine(symbol("/="));
    nfa.combine(symbol("%="));
    nfa.combine(symbol("&="));
    nfa.combine(symbol("^="));
    nfa.combine(symbol("|="));
    nfa.combine(symbol(">>"));
    nfa.combine(symbol("<<"));
    nfa.combine(symbol("++"));
    nfa.combine(symbol("--"));
    nfa.combine(symbol("->"));
    nfa.combine(symbol("&&"));
    nfa.combine(symbol("||"));
    nfa.combine(symbol("<="));
    nfa.combine(symbol(">="));
    nfa.combine(symbol("=="));
    nfa.combine(symbol("!="));
    nfa.combine(symbol(";"));
    nfa.combine(symbol("{"));
    nfa.combine(symbol("<%"));
    nfa.combine(symbol("}"));
    nfa.combine(symbol("%>"));
    nfa.combine(symbol(","));
    nfa.combine(symbol(":"));
    nfa.combine(symbol("="));
    nfa.combine(symbol("("));
    nfa.combine(symbol(")"));
    nfa.combine(symbol("["));
    nfa.combine(symbol("<:"));
    nfa.combine(symbol("]"));
    nfa.combine(symbol(":>"));
    nfa.combine(symbol("."));
    nfa.combine(symbol("&"));
    nfa.combine(symbol("!"));
    nfa.combine(symbol("~"));
    nfa.combine(symbol("-"));
    nfa.combine(symbol("+"));
    nfa.combine(symbol("*"));
    nfa.combine(symbol("/"));
    nfa.combine(symbol("%"));
    nfa.combine(symbol("<"));
    nfa.combine(symbol(">"));
    nfa.combine(symbol("^"));
    nfa.combine(symbol("|"));
    nfa.combine(symbol("?"));
    nfa.combine(number);
    nfa.combine(id);

    let dfa = Dfa::from_nfa(&nfa);

    let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("c-basic.rs");

    let mut out = File::create(outfile).unwrap();

    let mut gen = CodeGen::new();

    gen.set_token_type("Token");

    gen.generate(&dfa, &mut out).unwrap();
}
