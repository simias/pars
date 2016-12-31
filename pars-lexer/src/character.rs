use std::fmt;
use std::cmp::{min, max};

/// A character interval matching any character it containis
/// inclusively
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Interval {
    first: u32,
    last: u32,
}

impl Interval {
    /// Create an interval that matches `[first-last]`
    pub fn new(mut first: char, mut last: char) -> Interval {
        if first > last {
            ::std::mem::swap(&mut first, &mut last);
        }

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

    /// Create an interval containing all possible characters.
    pub fn new_any() -> Interval {
        Interval {
            first: 0,
            last: u32::max_value(),
        }
    }

    /// Returns `true` if `other` intersects `self`
    pub fn intersects(&self, other: Interval) -> bool {
        self.first <= other.last && other.first <= self.last
    }

    /// Returns the intersection between two intervals as well as the
    /// leftovers before and after the intersection:
    ///
    /// Or in graphic form:
    /// ```text
    /// a:              [0  1  2  3  4  5]
    /// b:                       [3  4  5  6  7  8]
    /// c:                       [3  4]
    /// d:              [0  1]
    ///
    ///
    /// a.intersect(b): [0  1  2][3  4  5][6  7  8]
    /// a.intersect(c): [0  1  2][3  4][5]
    /// b.intersect(c):        [][3  4][5  6  7  8]
    /// c.intersect(d): [0  1] [][3  4]
    /// ```
    ///
    /// The method returns `(left, intersection, right)`. Any empty
    /// interval is returned as `None`
    pub fn intersect(&self, other: Interval)
                     -> (Option<Interval>, Option<Interval>, Option<Interval>) {

        let (a, b) =
            if *self < other {
                (*self, other)
            } else {
                (other, *self)
            };

        if !a.intersects(b) {
            return (Some(a), None, Some(b))
        }

        let a_f = a.first();
        let b_f = b.first();
        let a_l = a.last();
        let b_l = b.last();

        let left_first = a_f;
        let mid_first = b_f;
        let mid_last = min(a_l, b_l);
        let right_last = max(a_l, b_l);

        let maybe = |first, last| {
            if first <= last {
                Some(Interval {
                    first: first,
                    last: last,
                })
            } else {
                None
            }
        };

        let middle = maybe(mid_first, mid_last);

        let left =
            if mid_first > 0 {
                maybe(left_first, mid_first - 1)
            } else {
                None
            };

        let right =
            if mid_last < u32::max_value() {
                maybe(mid_last + 1, right_last)
            } else {
                None
            };

        (left, middle, right)
    }

    pub fn first(&self) -> u32 {
        self.first
    }

    pub fn last(&self) -> u32 {
        self.last
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "["));
        try!(write_char(f, self.first));

        if self.first != self.last {
            try!(write!(f, "-"));
            try!(write_char(f, self.last));
        }

        write!(f, "]")
    }
}

fn write_char(f: &mut fmt::Formatter, c: u32) -> fmt::Result {
    if c <= 0xff {
        let c = c as u8;

        match c {
            b'!'...b'~' => write!(f, "{}", c as char),
            _ => write!(f, "\\x{:02x}", c)

        }
    } else {
        write!(f, "\\x{{{:x}}}", c)
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

    assert!(a.intersects(a2));
    assert!(!a.intersects(b));


    let az = Interval::new('a', 'z');
    let al = Interval::new('a', 'l');
    let ae = Interval::new('a', 'e');
    let fl = Interval::new('f', 'l');
    let mz = Interval::new('m', 'z');
    let lz = Interval::new('l', 'z');
    let cu = Interval::new('c', 'u');
    let ab = Interval::new('a', 'b');
    let cl = Interval::new('c', 'l');
    let mu = Interval::new('m', 'u');

    let any = Interval::new_any();

    assert!(al.intersects(a));
    assert!(al.intersects(al));
    assert!(al.intersects(lz));
    assert!(al.intersects(b));
    assert!(b.intersects(al));

    assert!(!b.intersects(cu));
    assert!(!ae.intersects(lz));
    assert!(!lz.intersects(ae));

    assert_eq!(az.intersect(fl), (Some(ae), Some(fl), Some(mz)));
    assert_eq!(fl.intersect(az), (Some(ae), Some(fl), Some(mz)));

    assert_eq!(ae.intersect(lz), (Some(ae), None, Some(lz)));
    assert_eq!(lz.intersect(ae), (Some(ae), None, Some(lz)));

    assert_eq!(al.intersect(ae), (None, Some(ae), Some(fl)));
    assert_eq!(ae.intersect(al), (None, Some(ae), Some(fl)));

    assert_eq!(al.intersect(cu), (Some(ab), Some(cl), Some(mu)));
    assert_eq!(cu.intersect(al), (Some(ab), Some(cl), Some(mu)));

    assert_eq!(any.intersect(az), az.intersect(any));

    assert_eq!(az.intersect(az), (None, Some(az), None));
    assert_eq!(any.intersect(any), (None, Some(any), None));
}
