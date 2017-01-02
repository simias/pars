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
    let mut buf: &[u8] = b"abc bcd cde xyz  azerty";

    let mut lexer = intersecting_intervals::Lexer::new(&mut buf);

    assert_eq!(lexer.next_token().unwrap(), "ae".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "bd".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "ae".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "cz".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "space".to_owned());
    assert_eq!(lexer.next_token().unwrap(), "az".to_owned());

    match lexer.next_token() {
        Err(intersecting_intervals::LexerError::EndOfFile) => (),
        e => panic!("Expected EOF, got {:?}", e),
    }
}
