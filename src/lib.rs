use std::collections::BTreeSet;

pub type SudokuResult = Result<(), SudokuError>;

fn i32_from_char(c: char) -> Option<i32> {
    match c {
        //'0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        _ => None,
    }
}

fn char_from32(v: i32) -> Option<char> {
    match v {
        0 => Some('0'),
        1 => Some('1'),
        2 => Some('2'),
        3 => Some('3'),
        4 => Some('4'),
        5 => Some('5'),
        6 => Some('6'),
        7 => Some('7'),
        8 => Some('8'),
        9 => Some('9'),
        _ => None,
    }
}

#[derive(Debug)]
pub enum SudokuError {
    // A value specified is outside the valid range
    InvalidRange,
    // The solver found this board in not solvable
    NotSolvable,
    // The solver has many options and does not know which one to choose
    TooManyOptions,
    // This option is already known, but we are trying to mark it again
    AlreadyKnown,
    // The board is not fully solved.  It branches and needs help
    NoFullySolved,
    // unknown error
    Unknown,
}

#[derive(Debug, Clone)]
pub enum BoxValue {
    Known(i32),
    Unknown(BTreeSet<i32>),
}

impl BoxValue {
    fn init_unknown() -> crate::BoxValue {
        let ret: BTreeSet<i32> = (1..=9).collect();
        BoxValue::Unknown(ret)
    }
}

#[derive(Clone)]
pub struct Node {
    pub row: usize,
    pub col: usize,
    pub value: BoxValue,
}

impl Node {
    /// Get the square number in the sudoku grid.
    ///
    /// The sudoku grid is split into 3x3 grids:
    ///
    /// The table of values for each column and row:
    ///
    ///   123456789
    ///   ---------
    /// 1|111222333
    /// 2|111222333
    /// 3|111222333
    /// 4|444555666
    /// 5|444555666
    /// 6|444555666
    /// 7|777888999
    /// 8|777888999
    /// 9|777888999
    ///
    fn get_square(&self) -> usize {
        ((self.row - 1) / 3) * 3 + (self.col - 1) / 3 + 1
    }

    fn reverse_square(square_id: usize, idx: usize) -> (usize, usize) {
        if square_id > 9 {
            panic!("this should not happen");
        }
        if idx > 9 {
            panic!("This should not happen");
        }
        let r_mult = (square_id - 1) / 3;
        let c_mult = (square_id - 1) % 3;

        let row = (r_mult * 3) + (idx / 3) + 1;
        let col = (c_mult * 3) + (idx % 3) + 1;
        (row, col)
    }
}

#[derive(Clone)]
pub struct SudokuBoard {
    board: Vec<Vec<Node>>,

    unknown_values: i32,
}

impl SudokuBoard {
    pub fn new() -> SudokuBoard {
        //let mut board = Vec::new();
        let mut board: Vec<Vec<Node>> = (0..9).map(|_| Vec::new()).collect();

        for row in 0..9 {
            for col in 0..9 {
                let node = Node {
                    row: row + 1,
                    col: col + 1,
                    value: BoxValue::init_unknown(),
                };
                board.get_mut(row).unwrap().push(node);
            }
        }

        SudokuBoard {
            board,
            unknown_values: 9 * 9,
        }
    }

    /// Initialize the board given a string.  The string is a sequence of numeric characters.
    /// Non-numeric characters are ignored.  It is filled from top to bottom left to right.
    pub fn fill_board(s: &String) -> Result<SudokuBoard, SudokuError> {
        let mut board = SudokuBoard::new();

        for (i, c) in s
            .chars()
            .filter(|c| {
                *c == '0'
                    || *c == '1'
                    || *c == '2'
                    || *c == '3'
                    || *c == '4'
                    || *c == '5'
                    || *c == '6'
                    || *c == '7'
                    || *c == '8'
                    || *c == '9'
                    || *c == '-'
            })
            .enumerate()
        {
            let row = i / 9 + 1;
            let col = i % 9 + 1;
            let value = i32_from_char(c);
            match value {
                Some(know_value) => board.mark_as_known(row, col, know_value)?,
                None => (),
            }
        }
        Ok(board)
    }

    pub fn print_board(&self) -> String {
        self.board
            .iter()
            .flatten()
            .map(|v| match v.value {
                BoxValue::Known(v) => char_from32(v).unwrap_or('?'),
                BoxValue::Unknown(_) => '-',
            })
            .collect::<String>()
    }

    pub fn print_possibility(&self) -> String {
        self.board
            .iter()
            .flatten()
            .map(|v| match &v.value {
                BoxValue::Known(_) => 'K',
                BoxValue::Unknown(v) => {
                    char_from32((v.len() as usize).try_into().unwrap()).unwrap()
                }
            })
            .collect::<String>()
    }

    // if this value has a single item it will mark the known value.
    fn mark_single_option(&mut self, row: usize, col: usize) -> SudokuResult {
        if row > 9 {
            return SudokuResult::Err(SudokuError::InvalidRange);
        }
        if col > 9 {
            return SudokuResult::Err(SudokuError::InvalidRange);
        }

        // get the value we will mark it as known
        let known_value = match &self.board.get(row - 1).unwrap().get(col - 1).unwrap().value {
            BoxValue::Known(_) => return SudokuResult::Err(SudokuError::AlreadyKnown),
            BoxValue::Unknown(v) => {
                if v.is_empty() {
                    return SudokuResult::Err(SudokuError::NotSolvable);
                }
                if v.len() != 1 {
                    return SudokuResult::Err(SudokuError::TooManyOptions);
                }
                // We have checked that there will be exactly one item in the set
                v.first().unwrap().clone()
            }
        };
        self.mark_as_known(row, col, known_value)
    }

    /// When marking an item as known, we first change the state of unknown
    /// to known, then mark everything in the row, column, and square so nothing
    /// else will have the same value.
    ///
    ///
    fn mark_as_known(&mut self, row: usize, col: usize, known_value: i32) -> SudokuResult {
        if row > 9 {
            return SudokuResult::Err(SudokuError::InvalidRange);
        }
        if col > 9 {
            return SudokuResult::Err(SudokuError::InvalidRange);
        }

        self.board
            .get_mut(row - 1)
            .unwrap()
            .get_mut(col - 1)
            .unwrap()
            .value = BoxValue::Known(known_value);

        let square_value = self
            .board
            .get(row - 1)
            .unwrap()
            .get(col - 1)
            .unwrap()
            .get_square();

        // scan the row, column, and square.  Remove the known value as a possibility.
        for i in 0..9 {
            match &mut self
                .board
                .get_mut(row - 1)
                .unwrap()
                .get_mut(i)
                .unwrap()
                .value
            {
                BoxValue::Known(_) => (),
                BoxValue::Unknown(v) => {
                    v.remove(&known_value);
                    if v.is_empty() {
                        return SudokuResult::Err(SudokuError::NotSolvable);
                    }
                }
            }
            match &mut self
                .board
                .get_mut(i)
                .unwrap()
                .get_mut(col - 1)
                .unwrap()
                .value
            {
                BoxValue::Known(_) => (),
                BoxValue::Unknown(v) => {
                    v.remove(&known_value);
                    if v.is_empty() {
                        return SudokuResult::Err(SudokuError::NotSolvable);
                    }
                }
            }
            let (r, c) = Node::reverse_square(square_value, i);
            match &mut self
                .board
                .get_mut(r - 1)
                .unwrap()
                .get_mut(c - 1)
                .unwrap()
                .value
            {
                BoxValue::Known(_) => (),
                BoxValue::Unknown(v) => {
                    v.remove(&known_value);
                    if v.is_empty() {
                        return SudokuResult::Err(SudokuError::NotSolvable);
                    }
                }
            }
        }

        self.unknown_values -= 1;
        Ok(())
    }

    /// Attempt to solve the sudoku as much as possible by finding
    /// a square that only has one alternative and marking it as known.
    pub fn solve(&mut self) -> Result<(), SudokuError> {
        // find a node that has unknown value but only has one alternative
        while self.unknown_values > 0 {
            let n = self.board.iter().flatten().find(|v| match &v.value {
                BoxValue::Unknown(v) if v.len() == 1 => true,
                _ => false,
            });
            match n {
                Some(nv) => {
                    self.mark_single_option(nv.row, nv.col)?;
                }
                // This will happen if it can't find an option that has only one option.
                None => {
                    // find the minimum number of alternatives to try
                    let min_alternatives = self
                        .board
                        .iter()
                        .flatten()
                        .map(|v| match &v.value {
                            BoxValue::Unknown(v) => v.len(),
                            _ => 0,
                        })
                        .filter(|v| v > &0)
                        .min()
                        .unwrap();
                    // find a node that has that many alternatives.
                    let n = self.board.iter().flatten().find(|v| match &v.value {
                        BoxValue::Unknown(v) if v.len() == min_alternatives => true,
                        _ => false,
                    });
                    let alt_node = match n {
                        Some(n) => n,
                        None => return Err(SudokuError::Unknown),
                    };
                    let alt_set = match &alt_node.value {
                        BoxValue::Known(_) => return Err(SudokuError::Unknown),
                        BoxValue::Unknown(s) => s,
                    };
                    // we look at each alternative.  Run solve on each alternative until we find a match.
                    for alt_item in alt_set {
                        let mut alt_board = self.clone();
                        let _ = alt_board.mark_as_known(alt_node.row, alt_node.col, *alt_item);
                        match alt_board.solve() {
                            // we found a solution in one of the alternatives.  Return this
                            // alternative right away.
                            Ok(_) => {
                                self.board = alt_board.board;
                                self.unknown_values = alt_board.unknown_values;
                                return Ok(());
                            }
                            // if a solution could not be found, try another alternative
                            Err(_) => (),
                        };
                        // if nothing could be found, report so
                    }
                    return Err(SudokuError::NotSolvable);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::BoxValue;
    use crate::Node;
    use crate::SudokuBoard;
    #[test]
    fn test_square() {
        let mut n = Node {
            row: 1,
            col: 1,
            value: BoxValue::Known(1),
        };
        assert_eq!(n.get_square(), 1);
        n.row = 2;
        assert_eq!(n.get_square(), 1);
        n.row = 3;
        assert_eq!(n.get_square(), 1);

        n.col = 6;
        assert_eq!(n.get_square(), 2);

        n.col = 7;
        assert_eq!(n.get_square(), 3);

        n.row = 6;
        assert_eq!(n.get_square(), 6);

        n.row = 7;
        assert_eq!(n.get_square(), 9);
    }

    #[test]
    fn test_reverse_square() {
        assert_eq!(Node::reverse_square(1, 0), (1, 1));
        assert_eq!(Node::reverse_square(1, 1), (1, 2));
        assert_eq!(Node::reverse_square(1, 2), (1, 3));
        assert_eq!(Node::reverse_square(1, 3), (2, 1));
        assert_eq!(Node::reverse_square(1, 4), (2, 2));
        assert_eq!(Node::reverse_square(1, 5), (2, 3));
        assert_eq!(Node::reverse_square(1, 6), (3, 1));
        assert_eq!(Node::reverse_square(1, 7), (3, 2));
        assert_eq!(Node::reverse_square(1, 8), (3, 3));

        assert_eq!(Node::reverse_square(2, 0), (1, 4));
        assert_eq!(Node::reverse_square(2, 1), (1, 5));
        assert_eq!(Node::reverse_square(2, 2), (1, 6));
        assert_eq!(Node::reverse_square(2, 3), (2, 4));
        assert_eq!(Node::reverse_square(2, 4), (2, 5));
        assert_eq!(Node::reverse_square(2, 5), (2, 6));
        assert_eq!(Node::reverse_square(2, 6), (3, 4));
        assert_eq!(Node::reverse_square(2, 7), (3, 5));
        assert_eq!(Node::reverse_square(2, 8), (3, 6));

        assert_eq!(Node::reverse_square(3, 0), (1, 7));
        assert_eq!(Node::reverse_square(3, 1), (1, 8));
        assert_eq!(Node::reverse_square(3, 2), (1, 9));
        assert_eq!(Node::reverse_square(3, 3), (2, 7));
        assert_eq!(Node::reverse_square(3, 4), (2, 8));
        assert_eq!(Node::reverse_square(3, 5), (2, 9));
        assert_eq!(Node::reverse_square(3, 6), (3, 7));
        assert_eq!(Node::reverse_square(3, 7), (3, 8));
        assert_eq!(Node::reverse_square(3, 8), (3, 9));

        assert_eq!(Node::reverse_square(4, 0), (4, 1));
        assert_eq!(Node::reverse_square(4, 1), (4, 2));
        assert_eq!(Node::reverse_square(4, 2), (4, 3));
        assert_eq!(Node::reverse_square(4, 3), (5, 1));
        assert_eq!(Node::reverse_square(4, 4), (5, 2));
        assert_eq!(Node::reverse_square(4, 5), (5, 3));
        assert_eq!(Node::reverse_square(4, 6), (6, 1));
        assert_eq!(Node::reverse_square(4, 7), (6, 2));
        assert_eq!(Node::reverse_square(4, 8), (6, 3));

        assert_eq!(Node::reverse_square(5, 0), (4, 4));
        assert_eq!(Node::reverse_square(5, 1), (4, 5));
        assert_eq!(Node::reverse_square(5, 2), (4, 6));
        assert_eq!(Node::reverse_square(5, 3), (5, 4));
        assert_eq!(Node::reverse_square(5, 4), (5, 5));
        assert_eq!(Node::reverse_square(5, 5), (5, 6));
        assert_eq!(Node::reverse_square(5, 6), (6, 4));
        assert_eq!(Node::reverse_square(5, 7), (6, 5));
        assert_eq!(Node::reverse_square(5, 8), (6, 6));

        assert_eq!(Node::reverse_square(6, 0), (4, 7));
        assert_eq!(Node::reverse_square(6, 1), (4, 8));
        assert_eq!(Node::reverse_square(6, 2), (4, 9));
        assert_eq!(Node::reverse_square(6, 3), (5, 7));
        assert_eq!(Node::reverse_square(6, 4), (5, 8));
        assert_eq!(Node::reverse_square(6, 5), (5, 9));
        assert_eq!(Node::reverse_square(6, 6), (6, 7));
        assert_eq!(Node::reverse_square(6, 7), (6, 8));
        assert_eq!(Node::reverse_square(6, 8), (6, 9));

        assert_eq!(Node::reverse_square(7, 0), (7, 1));
        assert_eq!(Node::reverse_square(7, 1), (7, 2));
        assert_eq!(Node::reverse_square(7, 2), (7, 3));
        assert_eq!(Node::reverse_square(7, 3), (8, 1));
        assert_eq!(Node::reverse_square(7, 4), (8, 2));
        assert_eq!(Node::reverse_square(7, 5), (8, 3));
        assert_eq!(Node::reverse_square(7, 6), (9, 1));
        assert_eq!(Node::reverse_square(7, 7), (9, 2));
        assert_eq!(Node::reverse_square(7, 8), (9, 3));

        assert_eq!(Node::reverse_square(8, 0), (7, 4));
        assert_eq!(Node::reverse_square(8, 1), (7, 5));
        assert_eq!(Node::reverse_square(8, 2), (7, 6));
        assert_eq!(Node::reverse_square(8, 3), (8, 4));
        assert_eq!(Node::reverse_square(8, 4), (8, 5));
        assert_eq!(Node::reverse_square(8, 5), (8, 6));
        assert_eq!(Node::reverse_square(8, 6), (9, 4));
        assert_eq!(Node::reverse_square(8, 7), (9, 5));
        assert_eq!(Node::reverse_square(8, 8), (9, 6));

        assert_eq!(Node::reverse_square(9, 0), (7, 7));
        assert_eq!(Node::reverse_square(9, 1), (7, 8));
        assert_eq!(Node::reverse_square(9, 2), (7, 9));
        assert_eq!(Node::reverse_square(9, 3), (8, 7));
        assert_eq!(Node::reverse_square(9, 4), (8, 8));
        assert_eq!(Node::reverse_square(9, 5), (8, 9));
        assert_eq!(Node::reverse_square(9, 6), (9, 7));
        assert_eq!(Node::reverse_square(9, 7), (9, 8));
        assert_eq!(Node::reverse_square(9, 8), (9, 9));
    }

    #[test]
    fn test_board() {
        let mut sboard = SudokuBoard::new();

        let _ = sboard.mark_as_known(1, 1, 1);
        let _ = sboard.mark_as_known(9, 9, 9);
    }

    #[test]
    fn test_fill_board() {
        let s = concat!(
            "4----8---",
            "----91-8-",
            "-865-2-3-",
            "-2-4--9--",
            "-1-2----6",
            "367-59---",
            "-----5---",
            "7--8---24",
            "2--93--7-"
        )
        .to_string();
        let mut sboard = SudokuBoard::fill_board(&s).unwrap();
        let result = sboard.print_board();
        assert_eq!(s, result);
        print!("{}", result);
    }

    #[test]
    fn test_solve() {
        let s = concat!(
            "500300600",
            "004001750",
            "000059100",
            "403200070",
            "006000000",
            "000000904",
            "700090315",
            "035000806",
            "619080000"
        )
        .to_string();
        let solution = concat!(
            "581327649",
            "924861753",
            "367459182",
            "493216578",
            "876945231",
            "152738964",
            "748692315",
            "235174896",
            "619583427",
        )
        .to_string();
        let mut sboard = SudokuBoard::fill_board(&s).unwrap();
        sboard.solve();
        let result = sboard.print_board();
        assert_eq!(result, solution);
    }
}
