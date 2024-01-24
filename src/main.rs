mod crosswords;

use crate::crosswords::basic_crossword;
use bevy::utils::HashMap;
use hexx::{DiagonalDirection, Direction, Hex};
use regex_automata::dfa::Automaton;
use regex_automata::{hybrid, Anchored};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Line {
    start: Hex,
    direction: Direction,
}

#[derive(Debug, Clone, Default)]
struct CharacterPermutations {
    bitfield: u32,
}

impl CharacterPermutations {
    fn new() -> Self {
        Self { bitfield: 0 }
    }

    fn add(&mut self, c: char) {
        debug_assert!(
            c.is_ascii_uppercase() && c >= 'A' && c <= 'Z',
            "Invalid character: {}",
            c
        );
        let index = (c as u32) - ('A' as u32);
        self.bitfield |= 1 << index;
    }

    fn contains(&self, c: char) -> bool {
        debug_assert!(
            c.is_ascii_uppercase() && c >= 'A' && c <= 'Z',
            "Invalid character: {}",
            c
        );
        let index = (c as u32) - ('A' as u32);
        (self.bitfield & (1 << index)) != 0
    }
}

#[derive(Debug, Clone, Default)]
struct StringPermutations(Vec<CharacterPermutations>);

#[derive(Debug, Clone)]
struct Crossword {
    radius: usize,
    expressions: HashMap<Line, String>,
    permutations: HashMap<Line, CharacterPermutations>,
}

impl Crossword {
    fn new(radius: usize) -> Self {
        Self {
            radius,
            expressions: HashMap::default(),
            permutations: Default::default(),
        }
    }

    fn add_expression(&mut self, line: Line, expression: String) {
        self.expressions.insert(line, expression);
    }
}

fn main() {
    let crossword = basic_crossword();
    println!("{:#?}", crossword);
}

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
