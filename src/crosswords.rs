use crate::{Crossword, Line, Search};
use hexx::DiagonalDirection::*;
use hexx::{DiagonalDirection, Direction, Hex};
use std::sync::Arc;

fn exp(s: &str) -> Search {
    let s = format!("^{}$", s);
    Search::Expression(s.into())
}

fn fun(f: impl Fn(&str) -> bool + 'static) -> Search {
    Search::Function(Arc::new(f))
}

pub fn basic_crossword_1() -> Crossword {
    let expressions = vec![
        (
            Right,
            vec![
                (0, -1, exp(r".A")),
                (-1, 0, exp(r"(B|CD)*")),
                (-1, 1, exp(r"([^A]*)")),
            ],
        ),
        (
            BottomLeft,
            vec![
                (0, -1, exp(r"CB")),
                (1, -1, exp(r"A.*B")),
                (1, 0, exp(r".*")),
            ],
        ),
        (
            TopLeft,
            vec![
                (1, 0, exp(r"(A|D)*")),
                (0, 1, exp(r"C*")),
                (-1, 1, fun(backreference_two_same_chars)),
            ],
        ),
    ];

    let mut crossword = Crossword::new(1);
    create(&mut crossword, expressions);

    crossword
}

pub fn basic_crossword_2() -> Crossword {
    let expressions = vec![
        (
            Right,
            vec![
                (0, -2, exp(r"(AB|C)*")),
                (-1, -1, exp(r"D.*F.*")),
                (-2, 0, exp(r".*(K|L)*.*")),
                (-2, 1, exp(r"(M|N|O|P)*")),
                (-2, 2, fun(backreference_three_same_chars)),
            ],
        ),
        (
            BottomLeft,
            vec![
                (2, 0, exp(r"LPQ")),
                (2, -1, exp(r"GK.*")),
                (2, -2, exp(r".*FJN.*")),
                (1, -2, exp(r"BEIM")),
                (0, -2, exp(r"ADH")),
            ],
        ),
        (
            TopLeft,
            vec![
                (-2, 2, exp(r".*")),
                (-1, 2, exp(r".*")),
                (0, 2, exp(r".*")),
                (1, 1, exp(r".*")),
                (2, 0, exp(r".*")),
            ],
        ),
    ];

    let mut crossword = Crossword::new(2);
    create(&mut crossword, expressions);

    crossword
}

fn create(
    crossword: &mut Crossword,
    expressions: Vec<(DiagonalDirection, Vec<(i32, i32, Search)>)>,
) {
    for (direction, expressions) in expressions {
        for (x, y, expression) in expressions {
            crossword.add(
                Line {
                    start: Hex::new(x, y),
                    direction: direction.direction_cw(),
                },
                expression.into(),
            );
        }
    }
}

/// (.)\1
///
/// return true if only one char.
fn backreference_two_same_chars(input: &str) -> bool {
    let mut chars = input.chars();
    let first = chars.next().unwrap();
    let Some(second) = chars.next() else {
        return true;
    };

    first == second
}

/// (.)\1\1
///
/// Allow zero or one char.
fn backreference_three_same_chars(input: &str) -> bool {
    let mut chars = input.chars();
    let first = chars.next().unwrap();

    if let Some(second) = chars.next() {
        if first == second {
            if let Some(third) = chars.next() {
                return first == third;
            }
        }
    }

    true
}
