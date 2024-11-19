use crate::cnf::{to_variable, Assignment};
use crate::dpll::solve_dpll;
use crate::expression::{self, Expression};
use std::sync::mpsc;

fn verify_assignment(expression: &Expression, assignment: &Assignment) -> bool {
    for clause in expression.get_clauses() {
        let mut satisfied = false;
        for literal in clause.literals() {
            let term = to_variable(*literal);
            let sign = *literal > 0;

            // The clause is satisfied as long as a literal's assignment matches its sign
            // Aka, if the literal is positive, the assignment must be true
            // If the literal is negative, the assignment must be false (making the literal true when it is negated)
            if assignment[&term] == sign {
                satisfied = true;
                break;
            }
        }

        if !satisfied {
            return false;
        }
    }

    return true;
}

pub fn solve(expression: Expression, use_multiple_threads: bool, verify: bool) -> Option<Assignment> {
    let mut expression_max_literals = expression.clone();
    let mut expression_min_clause_len = expression.clone();

    let (send_channel, recv_channel) = mpsc::channel();
    let send_channel_copy = send_channel.clone();

    std::thread::spawn(move || {
        expression_max_literals.optimize();
        expression_max_literals
            .set_heuristic(expression::SolverHeuristic::MostLiteralOccurances);

        let result = solve_dpll(&mut expression_max_literals);
        let _ = send_channel.send(result);
    });

    if use_multiple_threads {
        std::thread::spawn(move || {
            expression_min_clause_len.optimize();
            expression_min_clause_len
                .set_heuristic(expression::SolverHeuristic::MinimizeClauseLength);
    
            let result = solve_dpll(&mut expression_min_clause_len);
            let _ = send_channel_copy.send(result);
        });
    }

    let solution = recv_channel.recv().expect("Could not receive result from solver.");
    if solution.is_some() && verify {
        let assignment = solution.clone().unwrap();
        if !verify_assignment(&expression, &assignment) {
            panic!("Solution is invalid!");
        }
    }

    return solution;
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnf::{Clause, CNF};
    use crate::expression::Expression;

    #[test]
    fn test_verify_assignment() {
        let mut expression = Expression::new();
        let mut clause = Clause::new();
        clause.insert_checked(1);
        clause.insert_checked(-2);
        expression.add_clause(clause);

        let mut assignment = Assignment::new();
        assignment.insert(1, true);
        assignment.insert(2, false);

        assert!(verify_assignment(&expression, &assignment));
    }

    #[test]
    fn test_verify_assignment_unsatisfied() {
        let mut expression = Expression::new();
        let mut clause = Clause::new();
        clause.insert_checked(1);
        clause.insert_checked(2);
        expression.add_clause(clause);

        let mut assignment = Assignment::new();
        assignment.insert(1, false);
        assignment.insert(2, false);

        assert!(!verify_assignment(&expression, &assignment));
    }

    #[test]
    fn test_verify_assignment_unsatisfied_multiple_clauses() {
        let mut expression = Expression::new();
        let mut clause = Clause::new();
        clause.insert_checked(1);
        clause.insert_checked(2);
        expression.add_clause(clause);

        let mut clause = Clause::new();
        clause.insert_checked(-3);
        clause.insert_checked(-4);
        expression.add_clause(clause);

        let mut assignment = Assignment::new();
        assignment.insert(1, false);
        assignment.insert(2, false);
        assignment.insert(3, true);
        assignment.insert(4, true);

        assert!(!verify_assignment(&expression, &assignment));
    }

    #[test]
    fn test_verify_assignment_satisfied_multiple_clauses() {
        let mut expression = Expression::new();
        let mut clause = Clause::new();
        clause.insert_checked(1);
        clause.insert_checked(-2);
        expression.add_clause(clause);

        let mut clause = Clause::new();
        clause.insert_checked(3);
        clause.insert_checked(-4);
        expression.add_clause(clause);

        let mut assignment = Assignment::new();
        assignment.insert(1, true);
        assignment.insert(2, false);
        assignment.insert(3, true);
        assignment.insert(4, false);

        assert!(verify_assignment(&expression, &assignment));
    }

}