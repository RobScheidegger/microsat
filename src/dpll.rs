use crate::{
    cnf::{ActionState, Assignment, CNF},
    expression::Expression,
};

pub fn solve_dpll(cnf: &mut Expression) -> Option<Assignment> {
    // Track where we are in the action stack
    let action_state: ActionState = cnf.get_action_state();

    // Try to do as much inference as we can before branching
    while cnf.is_inference_possible() {
        // Next, remove all of the unit clauses
        while cnf.remove_unit_clause().is_some() {}

        // If the CNF is satisfied, then we are done
        if cnf.is_unsatisfiable() {
            // Restore the action state (undo branching)
            cnf.restore_action_state(action_state);
            return None;
        }

        while cnf.remove_pure_literal().is_some() {}
    }

    if cnf.is_satisfied() {
        return Some(cnf.construct_assignment());
    }

    if cnf.is_unsatisfiable() {
        cnf.restore_action_state(action_state);
        return None;
    }

    // Pick some variable to branch on ("guess") to keep searching
    let branch_action_state = cnf.get_action_state();
    let (branch_variable, branch_value) = cnf.get_branch_variable();

    // Try the first branch value
    cnf.branch_variable(branch_variable, branch_value);

    let branch_result = solve_dpll(cnf);
    if branch_result.is_some() {
        return branch_result;
    }

    cnf.restore_action_state(branch_action_state);

    // Try the other branch value
    cnf.branch_variable(branch_variable, !branch_value);

    let branch_result = solve_dpll(cnf);
    if branch_result.is_some() {
        return branch_result;
    }

    cnf.restore_action_state(action_state);
    None
}