use hexx::Hex;
use regex_automata::dfa::Automaton;
use regex_automata::{hybrid, Anchored};
use std::collections::HashMap;

struct Line {
    start: Hex,
    //    direction: ,
}

struct Crossword {
    //    cells: HashMap<Hex, >
    //    expressions: HashMap<, String>,
}

fn main() {}

#[test]
fn test_dense_dfa() {
    use regex_automata::{
        dfa::{dense, Automaton},
        Input,
    };

    let pattern = r"^.(C|HH)*$";
    let dfa = dense::DFA::new(pattern).unwrap();
    // let dfa = hybrid::dfa::DFA::new(pattern).unwrap();
    let haystack = "CCCHCCHHC";

    let mut state = dfa
        .start_state_reverse(&Input::new(haystack).anchored(Anchored::No))
        .unwrap();

    // let mut cache = dfa.create_cache();
    // let mut state = dfa.start_state_forward(&mut cache, &Input::new(haystack).anchored(Anchored::Yes)).unwrap();

    for &b in haystack.as_bytes().iter().rev() {
        state = dfa.next_state(state, b);
        // state = dfa.next_state(&mut cache, state, b).unwrap();

        let s = state;
        println!(
            "char: {:?}, is_match: {}, is_special: {}, is_dead: {}",
            b as char,
            dfa.is_match_state(s),
            dfa.is_special_state(s),
            dfa.is_dead_state(s)
        );
    }

    state = dfa.next_eoi_state(state);
    println!(
        "is_match: {}, is_special: {}, is_dead: {}",
        dfa.is_match_state(state),
        dfa.is_special_state(state),
        dfa.is_dead_state(state)
    );
}
