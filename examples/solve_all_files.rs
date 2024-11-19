use microsat::{expression::Expression, solver::solve};

extern crate microsat;

/// Solves all of the CNF files in the examples/cnf directory
fn main() -> std::io::Result<()>
{
    let paths = std::fs::read_dir("examples/cnf").unwrap();
    for path in paths {
        let path_buf = path.unwrap().path();
        let path = path_buf.to_str().unwrap();

        println!("Solving file: {}", path);

        let expression = Expression::from_cnf_file(&path);

        // Solve the expression
        let result = solve(expression, true, true);
        println!("{:?}", result);
    }

    Ok(())
}
