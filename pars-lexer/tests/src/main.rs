#[cfg(not(test))]
fn main() {
    generate::simple();
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

    pub fn simple() {
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
}

#[cfg(test)]
mod simple {
    include!(concat!(env!("OUT_DIR"), "/simple.rs"));
}

#[test]
fn simple() {
    let mut buf: &[u8] = b"abcbabbababbabcd";

    let mut lexer = simple::Lexer::new(&mut buf);

    assert_eq!(lexer.next_token().unwrap(), "regex abc".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "regex (a|b)*abb".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "regex abc".to_owned());

    match lexer.next_token() {
        Err(simple::LexerError::NoMatch(pos)) => assert_eq!(pos, 15),
        _ => unreachable!(),
    }
}
