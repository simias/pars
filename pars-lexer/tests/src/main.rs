#[cfg(not(test))]
fn main() {
    generate::simple();
    generate::intervals();
    generate::intersecting_intervals();
}

#[cfg(not(test))]
mod generate {
    extern crate pars_lexer;

    use std::fs::File;
    use std::env;
    use std::path::Path;

    use self::pars_lexer::nfa::Nfa;
    use self::pars_lexer::dfa::Dfa;
    use self::pars_lexer::codegen::CodeGen;
    use self::pars_lexer::character::Interval;

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
        let lower = Nfa::new(Interval::new('a', 'z'));

        // [a-z]+
        let mut match_lower = lower.clone();
        match_lower.positive();
        match_lower.concat(Nfa::new_accepting("lowercase".into()));

        // a+
        let mut match_aplus = Nfa::new(Interval::new_single('a'));
        match_aplus.positive();
        match_aplus.concat(Nfa::new_accepting("a+".into()));

        // [ ]+
        let mut spaces = Nfa::new(Interval::new_single(' '));
        spaces.positive();
        spaces.concat(Nfa::new_accepting("space".into()));

        let mut nfa = match_lower.clone();
        nfa.combine(match_aplus);
        nfa.combine(spaces);

        println!("{:?}", nfa);

        let dfa = Dfa::from_nfa(&nfa);

        println!("{:?}", dfa);

        panic!();

        let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("intersecting-intervals.rs");

        let mut out = File::create(outfile).unwrap();

        let mut gen = CodeGen::new();

        gen.set_token_type("String");

        gen.generate(&dfa, &mut out).unwrap();
    }

}

#[cfg(test)]
mod simple {
    include!(concat!(env!("OUT_DIR"), "/simple.rs"));
}

#[test]
fn simple() {
    let mut buf: &[u8] = b"abcbabbababbabc";

    let mut lexer = simple::Lexer::new(&mut buf);

    assert_eq!(lexer.next_token().unwrap(), "regex abc".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "regex (a|b)*abb".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "regex abc".to_owned());

    match lexer.next_token() {
        Err(simple::LexerError::EndOfFile) => (),
        e => panic!("Expected EOF, got {:?}", e),
    }
}

#[cfg(test)]
mod intervals {
    include!(concat!(env!("OUT_DIR"), "/intervals.rs"));
}

#[test]
fn intervals() {
    let mut buf: &[u8] = b"foo bar   aZ _AbC12 a_b_c a0_bc 0invalid";

    let mut lexer = intervals::Lexer::new(&mut buf);

    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());

    match lexer.next_token() {
        Err(intervals::LexerError::NoMatch(32)) => (),
        e => panic!("Expected match error, got {:?}", e),
    }
}

#[cfg(test)]
mod intersecting_intervals {
    include!(concat!(env!("OUT_DIR"), "/intersecting-intervals.rs"));
}

#[test]
fn intersecting_intervals() {
    let mut buf: &[u8] = b"abcz ABCZ AbCz";

    let mut lexer = intersecting_intervals::Lexer::new(&mut buf);

    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "id".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());

    match lexer.next_token() {
        Err(intersecting_intervals::LexerError::NoMatch(32)) => (),
        e => panic!("Expected match error, got {:?}", e),
    }
}
