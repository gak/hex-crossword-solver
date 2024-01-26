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

pub fn basic_crossword() -> Crossword {
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

    crossword
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
