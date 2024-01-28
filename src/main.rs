mod crosswords;

use crate::crosswords::basic_crossword_2;
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

#[derive(Debug, Clone, Default)]
struct PotentialStrings(Vec<String>);

impl PotentialStrings {
    fn insert(&mut self, string: &str) {
        self.0.push(string.to_string());
    }

    // returns one item as an empty string if there are no strings
    fn iter_or_empty(&self) -> Vec<String> {
        if self.0.len() == 0 {
            vec!["".to_string()]
        } else {
            self.0.clone()
        }
    }
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
    permutations: HashMap<Line, PotentialStrings>,
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

    fn hex_border(&self, radius: usize) -> impl ExactSizeIterator<Item = Hex> + '_ {
        let mut cells = HashSet::new();
        for hex in hexagon(Hex::new(0, 0), radius as u32) {
            let distance = hex.distance_to(Hex::new(0, 0));
            if distance != radius as i32 {
                continue;
            }
            cells.insert(hex);
        }

        cells.into_iter()
    }

    fn create_task(&self, radius: usize, hex: &Hex) -> Task {
        dbg!(radius, hex);

        // Find the lines that intersect with the given hex.
        let mut lines = Vec::new();

        for line in self.expressions.keys() {
            if line.at(self.radius - radius) != *hex {
                continue;
            }
            println!("found at radius: {:?} forward", line);

            lines.push(LineTask {
                line: line.clone(),
                search: self.expressions[line].clone(),
                string_permutations: self.permutations[line].clone(),
            });
        }

        debug_assert!(lines.len() > 0, "There should be at least 1 line");

        Task {
            cell: *hex,
            lines,
            index: radius,
        }
    }
}

fn main() {
    let mut crossword = basic_crossword_2();

    for r in (0..=crossword.radius).rev() {
        let cells = crossword.hex_border(r).collect::<Vec<_>>();

        for cell in &cells {
            let task = crossword.create_task(r, cell);
            let changes = permutate(task);
            dbg!(&changes);

            for change in changes {
                crossword.permutations.get_mut(&change.line).unwrap().0 = change.strings;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct TaskNewStrings {
    line: Line,
    strings: Vec<String>,
}

fn permutate(task: Task) -> Vec<TaskNewStrings> {
    let mut first = true;
    let mut common_set = HashSet::new();

    let mut new_words = HashMap::new();

    for line_task in task.lines.iter() {
        println!("line: {:?}", line_task.search);

        let mut successful = Vec::with_capacity(26);
        for prefix in line_task.string_permutations.iter_or_empty() {
            let mut s = prefix.to_string();
            for char in az() {
                s.push(char);
                let good = match line_task.search {
                    Search::Expression(ref expression) => partial_match_forward(expression, &s),
                    Search::Function(ref function) => function(&s),
                };

                if good {
                    successful.push(char);
                    new_words
                        .entry(line_task.line.clone())
                        .or_insert_with(HashMap::new)
                        .entry(char)
                        .or_insert_with(Vec::new)
                        .push(s.clone());
                }

                s.pop();
            }
        }

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

    println!("{:?}", &common_set);

    let mut task_new_strings = Vec::new();
    for line_task in task.lines.iter() {
        let mut new_strings = Vec::new();
        for char in &common_set {
            let mut s = new_words[&line_task.line][char].clone();
            new_strings.append(&mut s);
        }
        task_new_strings.push(TaskNewStrings {
            line: line_task.line.clone(),
            strings: new_strings,
        });
    }

    task_new_strings
}

fn az() -> impl Iterator<Item = char> {
    'A'..='Z'
}

fn partial_match_forward(expression: &str, string: &str) -> bool {
    // println!("expression: {:?}, string: {:?}", expression, string);
    let dfa = dense::DFA::new(expression).unwrap();
    let mut state = dfa.start_state_forward(&Input::new(string)).unwrap();

    for &b in string.as_bytes().iter() {
        state = dfa.next_state(state, b);

        // println!(
        //     "char: {:?}, is_match: {}, is_special: {}, is_dead: {}",
        //     b as char,
        //     dfa.is_match_state(state),
        //     dfa.is_special_state(state),
        //     dfa.is_dead_state(state)
        // );

        if dfa.is_dead_state(state) {
            return false;
        }
    }

    !dfa.is_dead_state(state)
}

#[derive(Clone, Debug)]
struct LineTask {
    line: Line,
    search: Search,
    string_permutations: PotentialStrings,
}

#[derive(Clone)]
struct Task {
    cell: Hex,

    lines: Vec<LineTask>,
    /// Distance from the outer ring.
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetContainer;

    #[test]
    fn string_tree() {
        let mut tree = PotentialStrings::default();
        tree.insert("DOG");
        tree.insert("C");
        tree.insert("CAT");
        tree.insert("CAP");

        let c = tree.iter_or_empty().collect::<Vec<_>>();
        assert!(c.contains(&"C"));
        assert!(c.contains(&"CAT"));
    }

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
