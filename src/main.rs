mod crosswords;

use crate::crosswords::basic_crossword;
use bevy::utils::HashMap;
use hexx::shapes::hexagon;
use hexx::{DiagonalDirection, Direction, Hex};
use regex_automata::dfa::Automaton;
use regex_automata::{hybrid, Anchored};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Line {
    start: Hex,
    direction: Direction,
}

impl Line {
    fn cells(&self, radius: usize) -> Vec<Hex> {
        let mut cells = Vec::new();
        let mut current = self.start;

        loop {
            cells.push(current);
            current = current.neighbor(self.direction);

            let distance = current.unsigned_distance_to(Hex::new(0, 0));
            if distance > radius as u32 {
                break;
            }
        }

        cells
    }

    fn at(&self, radius: usize) -> Hex {
        let mut current = self.start;

        for _ in 0..radius {
            current = current.neighbor(self.direction);
        }

        current
    }
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
    permutations: HashMap<Line, StringPermutations>,
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
        self.permutations.insert(line, Default::default());
    }

    fn hexes_at_radius(&self, radius: usize) -> impl ExactSizeIterator<Item = Hex> + '_ {
        hexagon(Hex::new(0, 0), radius as u32)
    }

    fn create_task(&self, radius: usize, hex: &Hex) -> Task {
        dbg!(radius, hex);

        // Find the 3 lines that intersect with the given hex.
        let mut lines = Vec::with_capacity(3);

        for line in self.expressions.keys() {
            // true for forward, false for reverse.
            let mut found = None;
            if line.at(radius) == *hex {
                found = Some(false);
            } else if line.at(self.radius - radius) == *hex {
                found = Some(true);
            }
            let Some(forward) = found else {
                continue;
            };

            lines.push(LineTask {
                expression: self.expressions[line].clone(),
                forward,
                string_permutations: self.permutations[line].clone(),
            });
        }

        debug_assert_eq!(lines.len(), 3, "Expected 3 lines, found {:?}", lines);
        // There should also be at least one forward and one reverse line.
        debug_assert!(lines.iter().any(|line| line.forward));
        debug_assert!(lines.iter().any(|line| !line.forward));

        Task {
            lines: [lines[0].clone(), lines[1].clone(), lines[2].clone()],
            index: radius,
        }
    }
}

fn main() {
    let crossword = basic_crossword();
    println!("{:#?}", crossword);

    let r = crossword.radius;
    let cells = crossword.hexes_at_radius(r).collect::<Vec<_>>();

    dbg!(&cells);

    let task = crossword.create_task(r, &cells[0]);
    dbg!(task);
}

#[derive(Debug, Clone)]
struct LineTask {
    expression: String,
    forward: bool,
    string_permutations: StringPermutations,
}

#[derive(Debug, Clone)]
struct Task {
    lines: [LineTask; 3],
    /// Distance from the outer ring.
    index: usize,
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
