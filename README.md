# sudoku-solver-rust
A basic sudoku solver written in Rust to practice writing things in Rust.

## Note

I wrote this to learn things about Rust.  The solution may not be ideal but
I was able to learn a lot about writing thins in Rust.  If you want to fix
things or improve things feel free do to it.  I appreciate learning new things
from people.

## The game of Sudoku

Sudoku is a puzzle of a 9x9 grid.  Each square on the grid can be a value (1,9).
but a value can only appear once per column, row, and 3x3 squares across the board.
A game should only have exactly one solution.

## Solver

The sudoku solver follows the basic rules of sudoku.  Each square in the grid
is either in a known state or unknown state.  When in an unknown a list of all
possibilities are kept.  When an item changes from the unknown state to the known
state the known value is removed from the list of possibilities on the row, column,
and 3x3 grid.

When a game board is first created known values are inserted based on the input.
This updates the unknown lists across the board.  When the solver is called it will
find a square with an unknown list that has only one item and change it from unknown to
known.  The solver will do this until the board is solved or no more unknown lists
have exactly one item.  If the board is not solved it will find an square with the shortest
unknown list and will attempt to solve with that guess.

## Running

The main function takes an input file containing one or more puzzles.  Unknown squares
are marked by `-` or `0`.  The file will be processed and the solutions will be printed
to standard out.

```
Grid 1
120005004
600810500
800060193
403070250
910000830
700200941
078109005
094000000
060080420
```
