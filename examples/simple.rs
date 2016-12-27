extern crate pars;

use pars::nfa::Nfa;
use pars::dfa::Dfa;

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
}
