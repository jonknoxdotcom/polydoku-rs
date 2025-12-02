// Sudoku solver
// with Hexdoku capability
// Jon Knox, 2025-11

// Goals
// 1. Define structures/implementations successfully, print name of cell
// 2. Be able to print the grid by fmt:Display and fmt:Debug
// 3. Apply functions to grid for cell status updates...
//    a) tritri application of not-allowed
//    b) column application of not-allowed
//    c) row application of not-allowed
// 5. Have CLI loop which has 'read', 'tritri', 'column', 'row' and 'show' functions
// 6. IO stuff
//    a) settle on file format, write test files
//    b) implement a 'write' function
//    c) implement a 'read' function
// 7. Perform 'next' = one-step fill in a next "obvious" solution and display with terminal highlight
// 8. Detect dependent pair in tri-tri
// 9. Implement cell status updates in ARM assembly language
// 10. Implement heuristic function for solving
//    a) single fn thread from next free cell
//    b) implement multiple threads (on M4 cpu)
//    c) select best-case starting point
// 11. Change dimensions from 9x9 of 3x3 3x3s of 1..9 to 16x16 of 4x4 of 4x4 of 0..9,a..f

// Notes
// vector alternatives described here:
// Re: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2018&gist=174c2ddb88ce053af6206927890d3591
//

use core::panic;
// Imports
use std::{arch::aarch64, fmt};
//use std::error::Error;
//use colored_text::Colorize;
// Re: https://github.com/seapagan/colored_text/blob/main/examples/basic.rs

use colored::Colorize;

// sudoku size (for 'classic', this is 9 states/cell, grid of 9 wide, 9 high)
const MAXSTATES: usize = 9; // max number of diff states a cell can have
const MAXROOTS: usize = 3; // max number of block size (isqrt of MAXSTATES)
const WIDTH: u8 = 9;
const HEIGHT: u8 = 9;

// sudoku number
type Snumb = u8; // holds values 1..9 or 0 for unknown

/// cell is a single element that holds a solution number (snumb)
// uses value 0 if unsolved
// the disallowed vector is an array [1..9] of known disallowed values
#[derive(Clone)]
struct Cell {
    solved: bool,    // whether the cell is solved
    solution: Snumb, // solved value of cell (only when self.solved==true)
    possible: Vec<bool>,
    disallowed: Vec<bool>,
    ispaired: bool,   //unused
    paired: (u8, u8), // unused
    highlight: u8,
}

/// Status of a Grid
#[derive(PartialEq)]
enum GridStatus {
    Solved,     // All cells are complete and logically correct
    Incomplete, // Puzzle is incomplete, number of cells remaining unsolved (include empty)
    Invalid,    // There are logic errors
    Unsolvable, // Nonspecific - has multiple solutions
    Empty,      // Empty grid, ready to load
    NotSquare,  // States count not a square number
}

/// grid consists of 9x9 cells
struct Grid {
    name: String,       // e.g. "Dummy Sudoku"
    state_dict: String, // e.g. "123456789"
    status: GridStatus,
    states: u8, // states is also width is also height
    isqrt: u8,  // integer sq root of states count
    size: u8,   // number of cells = states^2
    cells: Vec<Cell>,
    symbols: Vec<char>,
}

// Implement display trait
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // calculate solved cell count
        let mut used: u32 = 0;
        let total: usize = self.cells.len();
        for i in 0..total {
            if self.cells[i].solved {
                used += 1;
            }
        }
        write!(f, "{}: {}/{}", self.name, used, total)
    }
}

// Implement associated functions

impl Cell {
    fn empty(states: u8) -> Cell {
        Cell {
            solved: false,                            // whether the cell is solved
            solution: 0,                              // don't care
            possible: vec![false; states as usize],   // not computed
            disallowed: vec![false; states as usize], // not computed
            ispaired: false,
            paired: (0, 0),
            highlight: 0,
        }
    }
}

impl Grid {
    /// new - create an empty grid
    fn new(states: &str) -> Grid {
        let nstates = states.len() as u8;
        let int_sq_root = (nstates as f64).sqrt() as u32;

        let mut s = GridStatus::Empty;
        if nstates as u32 != int_sq_root * int_sq_root {
            s = GridStatus::NotSquare;
        }

        Grid {
            name: format!("Empty grid for {}", states),
            state_dict: states.to_owned(),
            status: s,
            states: nstates,
            isqrt: int_sq_root as u8,
            size: nstates * nstates,
            cells: vec![Cell::empty(nstates); (nstates * nstates) as usize],
            symbols: vec!['1', '2', '3', '4', '5', '6', '7', '8', '9'],
        }
    }
}




// Implement grid methods
impl Grid {
    // isempty - is an initialised empty grid
    fn isempty(&self) -> bool {
        self.status == GridStatus::Empty
    }

    fn bodge(&mut self, title: String, arr: Vec<u8>) -> Result<u32, &'static str> {
        self.name = title;

        if self.size != arr.len() as u8 {
            return Err("wrong length vector provided");
        }

        // populate grid with empty cells, unless solved (in 1...9)
        let mut used: u32 = 0;
        for i in 0..arr.len() {
            if arr[i] > 0 {
                self.cells[i].solved = true;
                self.cells[i].solution = arr[i] - 1;
                used += 1;
            }
        }

        Ok(used)
    }

    // load - get grid from file
    fn load(&self, filename: String) -> bool {
        println!("Loading file");
        true
    }

    // save - save current grid to file
    fn save(&self, filename: String) -> bool {
        println!("Saving file");
        true
    }

    // validate - check logic of current grid
    fn validate(&mut self) -> bool {
        println!("{}","Validating grid".underline());

        let mut valid = true;
        let mut ticked = [false; MAXSTATES];
        let mut start;

        // check horizontals
        for y in 0..self.states { // per row
            for el in 0..self.states {
                ticked[el as usize] = false;
            }

            start = y * self.states;
            for x in 0..self.states {
               //print!(" {}", x);
                let address = (start + x) as usize;
                if self.cells[address].solved {
                    let sol = self.cells[address].solution as usize;
                    if ticked[sol] {
                        // this solution already used on this line
                        self.cells[address].highlight = 2;
                        println!("Bad cell - horizontally repeated '{}' in ({},{})", self.symbols[sol], x+1,y+1);
                        valid = false;
                    }
                    ticked[sol] = true;
                }
            }
        }

        // check verticals
        for x in 0..self.states { // per col
            for el in 0..self.states {
                ticked[el as usize] = false;
            }
            
            for y in 0..self.states {
                let address = (x + y*self.states) as usize;
                if self.cells[address].solved {
                    let sol = self.cells[address].solution as usize;
                    if ticked[sol] {
                        // this solution already used on this line
                        self.cells[address].highlight = 2;
                        println!("Bad cell - vertically repeated '{}' in ({},{})", self.symbols[sol], x+1,y+1);
                        valid = false;
                    }
                    ticked[sol] = true;
                }
            }
        }

        // check blocks
        //println!("i={} s= {}", self.isqrt, self.states);
        for b in 0..self.states { // per block
            for el in 0..self.states {
                ticked[el as usize] = false;
            }

            let bx = (b % self.isqrt) * self.isqrt;
            let by = (b / self.isqrt) * self.isqrt * self.states;
            // println!("{} for {}+{}", b, bx, by);

            for y in 0..self.isqrt {
                for x in 0..self.isqrt {
                    let address = (bx+by+x+y*self.states) as usize;
                    //println!("a={} ", address);
                    if self.cells[address].solved {
                        let sol = self.cells[address].solution as usize;
                        if ticked[sol] {
                            // this solution already used in this block
                            self.cells[address].highlight = 2;
                            println!("Bad block - repeated '{}' in block {}", self.symbols[sol], b+1);
                            valid = false;
                        }
                        ticked[sol] = true;
                    }
                }
            }
        }

        valid
    }


//  I think I want to be doing this:
//     fn example(width: usize, height: usize) {
//         // Base 1d array
//         let mut grid_raw = vec![0; width * height];
//
//         // Vector of 'width' elements slices
//         let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_mut(width).collect();
//
//         // Final 2d array `&mut [&mut [_]]`
//         let grid = grid_base.as_mut_slice();
//
//         // Accessing data
//         grid[0][0] = 4;
//     }
// Re: https://stackoverflow.com/questions/13212212/creating-two-dimensional-arrays-in-rust



    // claim(r,c,state) - set a blank to a solution at (row,col)
    fn claim_rc(&mut self, row: usize, col: usize, sol: Snumb) {
        let mut address = (row * self.states as usize + col);
        self.claim_a(address, sol);
    }

    // claim_a(a,state) - set a blank to a solution at addr=a
    fn claim_a(&mut self, address: usize, sol: Snumb) {
        if self.cells[address].solved {
            println!("\nClaim a={} as {} failed",address, sol);
            panic!();
        }
        self.cells[address].solved = true;
        self.cells[address].solution = sol;
        self.cells[address].highlight = 1;
    }

    
    // validate - check logic of current grid
    fn solve_next(&mut self) -> u8 {
        println!("{}","Running solve_next".underline());

        // return value is number of cells added
        // (this is used to re-call the fn until exhaustion)
        let mut added = 0u8;

        // set up arrays row x state, and col x state
        println!("Computing row+col 'state claimed' boolmap");
        let mut rticked = [[false; MAXSTATES]; MAXSTATES];
        let mut cticked = [[false; MAXSTATES]; MAXSTATES];

        // variables which simplifies expressions/readability
        let n = self.states as usize;    // n = number of states (9 for Sudoku)
        let bw = self.isqrt as usize;    // bw = box width (3 for Sudoku)

        // a) do one-off walk over grid to set row/col boolmaps
        println!("{}","a) Set boolmaps".italic());
        for row in 0..n {
            for col in 0..n {
                let address = row * n + col;
                if self.cells[address].solved {
                    let sol = self.cells[address].solution as usize;
                    if rticked[row][sol] || cticked[col][sol] {
                        panic!()
                    }
                    rticked[row][sol] = true;
                    cticked[col][sol] = true;                    
                    //println!("- {row},{col} occupied by {sol}");
                }
            }
        }

        // b) do row based 'triple' rationalise (actually divided by isqrt)
        // we do this by processing blocks across, then separately down
        // a row is a candidate for completion of a state if it is in two blocks
        // if so, we need to identify the target row of the missing block
        // each of the slots in the row need to be checked for viability
        // if only one is viable, then this cell can be claimed
        println!("{}","b) 'triple' finder (under dev)".italic());
        for grid_block in 0..bw {    // three of these (of 3) in Sudoku = 0,1,2
            for state in 0..n {

                // compile these values for state
                let mut used = 0;
                let mut rowusage = [false; MAXSTATES];
                let mut slugmap = [[false; MAXROOTS]; MAXROOTS];

                for row in 0..bw {  // row means a row element of blocks on horiz row i
                    let start = (grid_block * bw + row) * n;
                    //println!("Row {} ({})", row, start);
                    for el in 0..n {    // across all cols whole row
                        if self.cells[start+el].solved  && self.cells[start+el].solution==state as u8 {
                            used += 1;
                            rowusage[row]=true;
                            let slug=el/bw; // integer divide
                            slugmap[row][slug]=true;
                        }
                    }
                }

                // we are only interested in rows where n-1 blocks are already populated
                //println!("Blk{} / state={} / used={} / slugs={:?}", grid_block, state,used,slugmap);
                if used==2 {
                    // find which row
                    let mut target_row: usize = 0;
                    for row in 0..bw {
                        if !rowusage[row] {
                            target_row=row;
                            break;
                        }
                    }
                    // find which block
                    let mut target_block: usize = 0;
                    for row in 0..bw {
                        //println!("--check {:?}", slugmap[row]);
                        if slugmap[row] == [false,false,false] {
                            target_block = row;
                            break;
                        }
                    }

                    println!("Grid block #{} / State {} x 2 + none on row {} block {}", grid_block, state, target_row, target_block);
                    // println!("   use={:?}",rowusage);
                    // println!(" slugs={:?}",slugmap);
               }
            }
        
        }


//panic!();


        // c) check boolmaps for '8/9' used ... by row
        println!("{}","c) check boolmaps for '8/9' used ... by row".italic());
        let mut used:usize = 0;
        for row in 0..n {
            used = 0;
            print!("R{:2}: ",self.symbols[row]);
            for sol in 0..n {
                if rticked[row][sol] {
                    print!("{}",self.symbols[sol]);
                    used += 1;
                }
            }
            // - see if can make immediate claim
            if used == n-1 {
                // which state is missing?
                let mut missed : Snumb = 0;  // might actually be 0
                for state in 0..n {
                   if !rticked[row][state] {
                        missed = state as Snumb;
                        break;
                   }
                }
                print!(" ... CLAIM - add {}\n",self.symbols[missed as usize]);
                // where is the gap?
                let mut address = 0; // init for compile
                for col in 0..n {
                    address = (row * n + col);
                    if !self.cells[address].solved {
                        self.claim_a(address, missed);
                        return 1;
                    }
                }
            }
            println!()
        }

        // d) check boolmaps for '8/9' used ... by column
        println!("{}","d) check boolmaps for '8/9' used ... by column".italic());
        for col in 0..n {
            used = 0;
            print!("C{:2}: ",self.symbols[col]);
            for sol in 0..n {
                if cticked[col][sol] {
                    print!("{}",self.symbols[sol]);
                    used += 1;
                }
            }
            // - see if can male immediate claim
            if used == n-1 {
                // which state is missing?
                let mut missed : Snumb = 0;  // might actually be 0
                for state in 0..n {
                   if !cticked[col][state] {
                        missed = state as Snumb;
                        break;
                   }
                }
                print!(" ... CLAIM - add {}\n",self.symbols[missed as usize]);
                // where is the gap?
                let mut address = 0; // init for compile
                for row in 0..n {
                    address = row * n + col;
                    if !self.cells[address].solved {
                        self.claim_rc(row, col, missed);
                        return 1;
                    }
                }
            } 
            println!()
        }

        // e) do block by block scan for 8/9 solved
        println!("{}","e) do block by block scan for 8/9 solved".italic());
        let mut bticked= [false; MAXSTATES];
        let mut used:usize = 0;
        for b in 0..n { // per block
            for el in 0..n {
                bticked[el as usize] = false;
            }

            // relative block offset
            let bx = (b % bw) * bw;
            let by = (b / bw) * bw * n;
            //println!("\nblock {} for {}+{}", b, bx, by);

            let  mut memx: usize = 0;   // (x,y) of last free cell
            let  mut memy: usize = 0;
            let  mut mema: usize = 0;
            for y in 0..bw {
                for x in 0..bw {
                    let address = (bx + by + x + y*n) as usize;
                    //print!("a={} ", address);
                    if self.cells[address].solved {
                        let sol = self.cells[address].solution as usize;
                        if bticked[sol] {
                            // should not be possible - would mean dup solution
                            panic!()
                        }
                        else {                     
                            bticked[sol] = true;
                        }
                    } else {
                        mema = address;
                        //print!("[save {}] ",address)
                    }
                }
            }

            used = 0;
            for el in 0..n {
                if bticked[el] {
                    used += 1;
                }
            }
            if used == n-1 {
                // which state is missing?
                let mut missed : Snumb = 0;  // might actually be 0
                for state in 0..n {
                   if !bticked[state] {
                        missed = state as Snumb;
                        break;
                   }
                }
                print!("CLAIM - add {} to {}\n",self.symbols[missed as usize], mema);
                //panic!();
            
                if !self.cells[mema].solved {
                    self.claim_a(mema, missed);
                    return 1;
                } else {
                    panic!()
                }

            }


        }






        added
    }








    // print - write grid to stdout
    fn print(&self, write_header:bool) {
        // calculate solved cell count
        let mut used: u32 = 0;
        let total: usize = self.cells.len();
        for i in 0..total {
            if self.cells[i].solved {
                used += 1;
            }
        }

        // write out header
        if write_header {
            println!("{} {}/{}", self.name, used, total);
        }

        // write out cells
        for i in 0..total {
            if i != 0 {
                if i % self.states as usize == 0 as usize {
                    println!("");
                    if i % (self.isqrt * self.states) as usize == 0 as usize {
                        println!("");
                    }
                } else if i % self.isqrt as usize == 0 as usize {
                    print!("   ");
                }
            }

            if self.cells[i].solved {
                // trying symbols rather than .chars().nth()  [still messy]
                //print!(" {} ", self.state_dict.chars().nth(self.cells[i].solution as usize).unwrap());
                let sym = format!("{}", self.symbols[(self.cells[i].solution) as usize]);
                match self.cells[i].highlight {
                1 => print!(" {} ", sym.green().bold()),
                2 => print!(" {} ", sym.red().bold()),
                _ => print!(" {} ", sym),
                }
                //print!(" {} ", sym.green().bold());
                //print!(" {} ", sym);
            } else {
                print!(" - ");
            }
        }
        println!()
    }

    // tab - write grid to stdout as tabs in 3-line format
    fn tab(&self) {
        // calculate solved cell count
        let mut used: u32 = 0;
        let total: usize = self.cells.len();
        for i in 0..total {
            if self.cells[i].solved {
                used += 1;
            }
        }
        println!("{} {}/{}", self.name, used, total);

        let row1 = "".to_string(); // possible row
        let row2 = "".to_string(); // solution row
        let row3 = "".to_string(); // disallowed row

        for i in 0..total {
            if i != 0 {
                if i % self.states as usize == 0 as usize {
                    println!("");
                    if i % (self.isqrt * self.states) as usize == 0 as usize {
                        println!("");
                    }
                } else if i % self.isqrt as usize == 0 as usize {
                    print!("   ");
                }
            }

            if self.cells[i].solved {
                print!(
                    " {} ",
                    self.state_dict
                        .chars()
                        .nth(self.cells[i].solution as usize)
                        .unwrap()
                );
            } else {
                print!(" - ");
            }
        }
        println!()
    }
}

fn main() {
    // Create an empty grid
    let mut g = Grid::new("123456789");
    if !g.isempty() {
        println!("Unable to configure");
        std::process::exit(1)
    }

    // Load grid with test data
    #[rustfmt::skip]
    let demox = vec![
        7, 0, 0,  0, 0, 0,  0, 0, 3, 
        0, 0, 0,  5, 7, 0,  0, 0, 0, 
        0, 6, 0,  0, 3, 1,  0, 0, 0, 

        0, 0, 0,  7, 5, 0,  8, 0, 0, 
        8, 9, 0,  0, 0, 4,  6, 0, 0, 
        0, 0, 0,  0, 0, 3,  0, 9, 0, 

        3, 7, 0,  6, 0, 0,  0, 0, 0, 
        1, 0, 2,  0, 9, 0,  5, 0, 0, 
        0, 5, 6,  0, 1, 0,  0, 0, 7u8,
    ];

    #[rustfmt::skip]
    let demo = vec![
        2, 0, 0,  0, 0, 0,  0, 0, 0, 
        3, 0, 0,  0, 0, 0,  0, 0, 4, 
        4, 0, 0,  0, 0, 0,  0, 0, 0, 

        5, 0, 0,  0, 0, 0,  1, 2, 3, 
        6, 0, 0,  0, 0, 0,  5, 0, 7, 
        7, 0, 0,  0, 0, 0,  6, 9, 8, 

        8, 0, 0,  0, 0, 0,  0, 0, 0, 
        0, 0, 0,  0, 0, 0,  0, 0, 0, 
        0, 2, 3,  4, 5, 6,  7, 8, 9_u8,
    ];
    println!("Length of test example = {}", demo.len());

    let result = g.bodge("2025-04-22_Hard".to_owned(), demo);
    match result {
        Ok(n) => println!("There were {}/81 solved cells", n),
        Err(e) => println!("Error: {}", e),
    }
    //println!("There were {}/81 solved cells", n);

    // use the fmt:Display of g to print the current grid
    println!("{}", g);

    // use my own print
    g.print(true);

    // do a validate
    if (!g.validate()) {
        println!("Grid is not valid");
        g.print(false);
        std::process::exit(1)
    }
    else {
        println!("It's fine");
    }

    while g.solve_next() >0 {
        g.print(false);
    }
    
    // g.load("test2a.sud".to_owned());
    // g.validate();
    // g.save("test2aupd.sud".to_owned());

    // g.load("test2b.sud".to_owned());
    // g.validate();
    // g.save("test2bupd.sud".to_owned());

    // println!("{}", g);

}
