use crate::{Crossword, Line};
use hexx::DiagonalDirection::*;
use hexx::{DiagonalDirection, Direction, Hex};

pub fn basic_crossword() -> Crossword {
    let expressions = vec![
        (
            Right,
            vec![(0, -1, r".A"), (-1, 0, r"(B|CD)*"), (-1, 1, r"([^A]*)")],
        ),
        (
            BottomLeft,
            vec![(0, -1, r"BC"), (1, -1, r"B.*A"), (1, 0, r".*")],
        ),
        (
            TopLeft,
            vec![(1, 0, r"(A|D)*"), (0, 1, r"C*"), (-1, 1, r"(.)\1")],
        ),
    ];

    let mut crossword = Crossword::new(1);

    for (direction, expressions) in expressions {
        for (x, y, expression) in expressions {
            crossword.add_expression(
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