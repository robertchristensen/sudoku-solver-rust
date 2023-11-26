use std::env;
use std::fs;

pub fn main() {
    let file_name = "test_sudoku.txt";

    let file_contents = fs::read_to_string(file_name).unwrap();

    let mut collected_lines = 0;
    let mut grid: String = String::new();
    for line in file_contents.split("\n") {
        if line.contains("Grid") {
            println!("{}", line);
        } else {
            collected_lines += 1;
            grid += line;
            if collected_lines == 9 {
                let mut solver = sudoku::SudokuBoard::fill_board(&grid).unwrap();
                solver.solve().unwrap();
                let to_print = solver.print_board();
                for r in 0..9 {
                    println!("{}", &to_print.as_str()[r * 9..(r + 1) * 9]);
                }
                collected_lines = 0;
                grid = String::new();
            }
        }
    }
}
