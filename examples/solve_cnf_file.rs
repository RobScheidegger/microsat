use microsat::{expression::Expression, solver::solve};

extern crate microsat;

/// Solves the provided CNF file
fn main()
{
    // Load the first argument as the filename
    let filename = std::env::args().nth(1).expect("No filename provided");
    let expression = Expression::from_cnf_file(&filename);

    // Solve the expression
    let result = solve(expression, true, true);
    println!("{:?}", result);
}

