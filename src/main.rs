mod crosswords;

use crate::crosswords::basic_crossword;
use bevy::utils::{HashMap, HashSet};
use hexx::shapes::hexagon;
use hexx::{DiagonalDirection, Direction, Hex};
use regex_automata::dfa::{dense, Automaton};
use regex_automata::{hybrid, Anchored, Input};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

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

// This needs to be a tree I think.
// Each node is a character, and each node has a list of children.
#[derive(Debug, Clone, Default)]
struct PotentialStringTree {
    children: HashMap<char, PotentialStringTree>,
}

#[derive(Clone)]
enum Search {
    Expression(String),
    Function(Arc<dyn Fn(&str) -> bool>),
}

impl Debug for Search {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Search::Expression(s) => write!(f, "Search::Expression({:?})", s),
            Search::Function(_) => write!(f, "Search::Function(...)"),
        }
    }
}

struct Crossword {
    radius: usize,
    expressions: HashMap<Line, Search>,
    permutations: HashMap<Line, PotentialStringTree>,
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
        self.add(line, Search::Expression(expression));
    }

    fn add_function(&mut self, line: Line, function: Arc<dyn Fn(&str) -> bool>) {
        self.add(line, Search::Function(function));
    }

    fn add(&mut self, line: Line, search: Search) {
        self.expressions.insert(line, search);
        self.permutations.insert(line, Default::default());
    }

    fn hexes_at_radius(&self, radius: usize) -> impl ExactSizeIterator<Item = Hex> + '_ {
        hexagon(Hex::new(0, 0), radius as u32)
    }

    fn create_task(&self, radius: usize, hex: &Hex) -> Task {
        dbg!(radius, hex);

        // Find the 2 lines that intersect with the given hex.
        let mut lines = Vec::with_capacity(2);

        for line in self.expressions.keys() {
            println!("\nline: {:?}", line);
            println!("line.at(radius): {:?}", line.at(radius));
            println!(
                "line.at(self.radius - radius): {:?}",
                line.at(self.radius - radius)
            );

            if line.at(self.radius - radius) != *hex {
                continue;
            }
            println!("found at radius: {:?} forward", line);

            lines.push(LineTask {
                search: self.expressions[line].clone(),
                string_permutations: self.permutations[line].clone(),
            });
        }

        debug_assert_eq!(
            lines.len(),
            2,
            "There should be 3 lines. Found: {:?}",
            lines
        );

        Task {
            cell: *hex,
            lines: [lines[0].clone(), lines[1].clone()],
            index: radius,
        }
    }
}

fn main() {
    let crossword = basic_crossword();

    let r = crossword.radius;
    let cells = crossword.hexes_at_radius(r).collect::<Vec<_>>();
    dbg!(&cells);

    let task = crossword.create_task(r, &cells[2]);

    permutate(task);
}

fn permutate(task: Task) {
    // For now just do the "last" permutation.
    let hex = task.cell;

    let mut s = String::new();
    let mut first = true;
    let mut common_set = HashSet::new();

    for line in &task.lines {
        let mut successful = vec![];
        println!("line: {:?}", line.search);
        for char in az() {
            s.push(char);
            match line.search {
                Search::Expression(ref expression) => {
                    if partial_match_forward(expression, &s) {
                        successful.push(s.clone());
                    }
                }

                Search::Function(ref function) => {
                    if function(&s) {
                        successful.push(s.clone());
                    }
                }
            }
            s.pop();
        }
        dbg!(&successful);

        if first {
            common_set = successful.into_iter().collect();
            first = false;
        } else {
            common_set = common_set
                .intersection(&successful.into_iter().collect())
                .cloned()
                .collect();
        }
    }

    dbg!(&common_set);
}

fn az() -> impl Iterator<Item = char> {
    'A'..='Z'
}

fn partial_match_forward(expression: &str, string: &str) -> bool {
    println!("expression: {:?}, string: {:?}", expression, string);
    let dfa = dense::DFA::new(expression).unwrap();
    let mut state = dfa.start_state_forward(&Input::new(string)).unwrap();

    for &b in string.as_bytes().iter() {
        state = dfa.next_state(state, b);

        println!(
            "char: {:?}, is_match: {}, is_special: {}, is_dead: {}",
            b as char,
            dfa.is_match_state(state),
            dfa.is_special_state(state),
            dfa.is_dead_state(state)
        );

        if dfa.is_dead_state(state) {
            return false;
        }
    }

    !dfa.is_dead_state(state)
}

#[derive(Clone, Debug)]
struct LineTask {
    search: Search,
    string_permutations: PotentialStringTree,
}

#[derive(Clone)]
struct Task {
    cell: Hex,

    lines: [LineTask; 2],
    /// Distance from the outer ring.
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward() {
        // expression, matches, non-matches
        let fixtures = vec![
            (
                "^(A|DC)*$",
                vec!["A", "DC", "AAADCD"],
                vec!["ADD", "DCDDC", "Z"],
            ),
            ("^.(A|D)*.$", vec!["A", "AA"], vec!["ADZZ", "DZC"]),
        ];

        for fixture in &fixtures {
            for string in &fixture.1 {
                assert!(
                    partial_match_forward(fixture.0, string),
                    "{:?} {:?}",
                    fixture,
                    string
                );
            }
            for string in &fixture.2 {
                assert!(
                    !partial_match_forward(fixture.0, string),
                    "{:?} {:?}",
                    fixture,
                    string
                );
            }
        }
    }

    #[test]
    fn test_dense_dfa() {
        use regex_automata::{
            dfa::{dense, Automaton},
            Input,
        };

        let pattern = r"^(A|DC)*$";
        let dfa = dense::DFA::new(pattern).unwrap();
        // let dfa = hybrid::dfa::DFA::new(pattern).unwrap();
        let haystack = "AAAAAAADC";
        // let haystack = "DC";

        let mut state = dfa
            .start_state_reverse(&Input::new(&haystack).anchored(Anchored::No))
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
}
