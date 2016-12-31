/// A character interval matching any character it containis
/// inclusively
#[derive(PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Interval {
    first: u32,
    last: u32,
}

impl Interval {
    /// Create an interval that matches `[first-last]`
    pub fn new(first: char, last: char) -> Interval {
        Interval {
            first: first as u32,
            last: last as u32,
        }
    }

    /// Create an interval containing a single character.
    pub fn new_single(c: char) -> Interval {
        Interval {
            first: c as u32,
            last: c as u32,
        }
    }

    /// Returs `true` if `other` intersects `self`
    pub fn intersects(&self, other: &Interval) -> bool {
        self.first <= other.last && self.last >= other.first
    }
}

#[test]
fn ordering() {
    let a = Interval::new_single('a');
    let a2 = Interval::new_single('a');
    let b = Interval::new_single('b');

    assert_eq!(a, a2);
    assert!(a < b);

    let al = Interval::new('a', 'l');
    let ae = Interval::new('a', 'e');
    let cu = Interval::new('c', 'u');
    let lz = Interval::new('l', 'z');

    assert_ne!(a, al);

    assert!(a < al);
    assert!(ae < al);
    assert!(al < b);
    assert!(al < cu);
    assert!(cu < lz);
    assert!(al < lz);
}

#[test]
fn intersection() {
    let a = Interval::new_single('a');
    let a2 = Interval::new_single('a');
    let b = Interval::new_single('b');

    assert!(a.intersects(&a2));
    assert!(!a.intersects(&b));


    let al = Interval::new('a', 'l');
    let ae = Interval::new('a', 'e');
    let cu = Interval::new('c', 'u');
    let lz = Interval::new('l', 'z');

    assert!(al.intersects(&a));
    assert!(al.intersects(&al));
    assert!(al.intersects(&lz));
    assert!(al.intersects(&b));
    assert!(b.intersects(&al));

    assert!(!b.intersects(&cu));
    assert!(!ae.intersects(&lz));
    assert!(!lz.intersects(&ae));
}
