extern crate pars;

use pars::nfa::Nfa;
use pars::dfa::Dfa;

fn main() {
    // (a|b)c*d
    let mut nfa = Nfa::new('a');
    nfa.union(Nfa::new('b'));

    let mut c_star = Nfa::new('c');
    c_star.star();

    nfa.concat(c_star);
    nfa.concat(Nfa::new('d'));

    let dfa = Dfa::from_nfa(&nfa);

    println!("{:?}", nfa);
}
