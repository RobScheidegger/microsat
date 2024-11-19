use crate::cnf::{Clause, Literal, CNF};
use crate::expression::Expression;

pub fn parse_dimacs(filename: &str) -> Expression {

    // Read the file from disk
    let mut cnf = Expression::new();
    let file = std::fs::read_to_string(filename).unwrap();

    // Read each line of the file
    for line in file.lines() {
        // If the line starts with 'c', then it is a comment, so skip it
        if line.starts_with('c') || line.is_empty() {
            continue;
        }

        // If the line starts with 'p', then it is a problem line
        if line.starts_with('p') {
            let mut parts = line.split_whitespace();
            let _ = parts.next(); // Skip the 'p'
            let _ = parts.next(); // Skip the 'cnf'
            continue;
        }

        // Otherwise, the line is a clause
        let mut clause = Clause::new();
        for literal in line.split_whitespace() {
            let value = literal.parse::<Literal>().unwrap();
            if value == 0 {
                break;
            }
            clause.insert_checked(value);
        }

        cnf.add_clause(clause);
    }

    cnf
}