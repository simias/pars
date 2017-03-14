mod simple {
    include!(concat!(env!("OUT_DIR"), "/simple.rs"));

    #[derive(Debug, PartialEq, Eq)]
    pub enum Token {
        /// (a|b)*abb
        Abaab(String),
        /// abc
        Abc(String),
    }

    #[test]
    fn lex() {
        let mut buf: &[u8] = b"abcbabbababbabc";

        let mut lexer = Lexer::new(&mut buf);

        assert_eq!(lexer.next_token().unwrap(),
                   Some(Token::Abc("abc".into())));
        assert_eq!(lexer.next_token().unwrap(),
                   Some(Token::Abaab("babbababb".into())));
        assert_eq!(lexer.next_token().unwrap(),
                   Some(Token::Abc("abc".into())));

        assert!(lexer.next_token().unwrap().is_none());
    }
}

mod intervals {
    include!(concat!(env!("OUT_DIR"), "/intervals.rs"));

    #[test]
    fn lex() {
        let mut buf: &[u8] = b"foo bar   aZ _AbC12 a_b_c a0_bc 0invalid";

        let mut lexer = Lexer::new(&mut buf);

        let expected = [
            "foo", "bar", "aZ", "_AbC12", "a_b_c", "a0_bc",
        ];

        for &id in &expected {
            assert_eq!(lexer.next_token().unwrap(),
                       Some(id.to_owned()));
        }

        match lexer.next_token() {
            Err(LexerError::NoMatch(32)) => (),
            e => panic!("Expected match error, got {:?}", e),
        }
    }
}

#[cfg(test)]
mod intersecting_intervals {
    include!(concat!(env!("OUT_DIR"), "/intersecting-intervals.rs"));

    #[derive(Debug, PartialEq, Eq)]
    pub enum Token {
        Ae,
        Bd,
        Cz,
        Az
    }

    #[test]
    fn lex() {
        let mut buf: &[u8] = b"abc bcd cde xyz  azerty";

        let mut lexer = Lexer::new(&mut buf);

        assert_eq!(lexer.next_token().unwrap(), Some(Token::Ae));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Bd));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Ae));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Cz));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Az));

        assert!(lexer.next_token().unwrap().is_none());
    }
}

#[cfg(test)]
mod utf8 {
    include!(concat!(env!("OUT_DIR"), "/utf8.rs"));

    #[derive(Debug, PartialEq, Eq)]
    pub enum Token {
        English,
        Russian,
    }

    #[test]
    fn lex() {
        let mut buf: &[u8] = "hello привет".as_bytes();

        let mut lexer = Lexer::new(&mut buf);

        assert_eq!(lexer.next_token().unwrap(), Some(Token::English));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Russian));

        assert!(lexer.next_token().unwrap().is_none());
    }
}

mod c_basic {
    include!(concat!(env!("OUT_DIR"), "/c-basic.rs"));

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Token {
        Id(String),
        Symbol(String),
        Number(i64),
    }

    #[test]
    fn lex() {
        use self::Token::*;

        let mut buf: &[u8] =
            b"int main() {                                  \
                int i;                                      \
                                                            \
                for (i = 0; i <= 10; i++) {                 \
                  printf(FMT, i + 2);                       \
                }                                           \
              }";

        let expected = [
            Symbol("int".into()),
            Id("main".into()),
            Symbol("(".into()),
            Symbol(")".into()),
            Symbol("{".into()),
            Symbol("int".into()),
            Id("i".into()),
            Symbol(";".into()),
            Symbol("for".into()),
            Symbol("(".into()),
            Id("i".into()),
            Symbol("=".into()),
            Number(0),
            Symbol(";".into()),
            Id("i".into()),
            Symbol("<=".into()),
            Number(10),
            Symbol(";".into()),
            Id("i".into()),
            Symbol("++".into()),
            Symbol(")".into()),
            Symbol("{".into()),
            Id("printf".into()),
            Symbol("(".into()),
            Id("FMT".into()),
            Symbol(",".into()),
            Id("i".into()),
            Symbol("+".into()),
            Number(2),
            Symbol(")".into()),
            Symbol(";".into()),
            Symbol("}".into()),
            Symbol("}".into()),
        ];

        let mut lexer = Lexer::new(&mut buf);

        for t in expected.iter() {
            assert_eq!(lexer.next_token().unwrap(), Some(t.clone()));
        }

        assert!(lexer.next_token().unwrap().is_none());
    }
}
