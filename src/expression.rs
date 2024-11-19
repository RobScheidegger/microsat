use core::panic;
use hashbrown::{HashMap, HashSet};
use std::cmp::{max, min, Ordering};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::cnf::{
    negate, to_variable, Action, ActionState, Assignment, Clause, ClauseId, Literal, Variable, CNF,
};
use crate::dimacs_parser::parse_dimacs;
use crate::stack::Stack;

#[derive(Clone, Copy, Debug)]
pub enum SolverHeuristic {
    MostLiteralOccurances,
    MostVariableOccurances,
    MinimizeClauseLength,
}

pub struct Expression {
    clauses: Vec<Clause>,
    variables: HashSet<Variable>,
    actions: Arc<RwLock<Stack<Action>>>,
    assignments: HashMap<Variable, bool>,

    literal_to_clause: HashMap<Literal, HashSet<ClauseId>>,
    unit_clauses: HashSet<ClauseId>,
    pure_literals: HashSet<Literal>,
    num_active_clauses: u16,
    num_empty_clauses: usize,
    max_clause_length: usize,
    pub heuristic: SolverHeuristic,
}

impl Clone for Expression {
    fn clone(&self) -> Self {
        let mut new_expression = Expression::new();
        for clause in &self.clauses {
            new_expression.add_clause(clause.clone());
        }

        new_expression
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self::new()
    }
}

impl Expression {
    pub fn new() -> Expression {
        Expression {
            clauses: Vec::new(),
            variables: HashSet::new(),
            actions: Arc::new(RwLock::new(Stack::new(0))),
            assignments: HashMap::new(),

            literal_to_clause: HashMap::new(),
            unit_clauses: HashSet::new(),
            pure_literals: HashSet::new(),
            num_active_clauses: 0,
            num_empty_clauses: 0,
            max_clause_length: 0,
            heuristic: SolverHeuristic::MostLiteralOccurances,
        }
    }

    pub fn from_clauses(clauses: Vec<Clause>) -> Expression {
        let mut expression = Expression::new();
        for clause in clauses {
            expression.add_clause(clause);
        }

        expression
    }

    pub fn from_cnf_file(file_name: &str) -> Expression {
        return parse_dimacs(file_name);
    }

    pub fn get_clauses(&self) -> Vec<Clause> {
        self.clauses.clone()
    }

    pub fn set_heuristic(&mut self, heuristic: SolverHeuristic) {
        self.heuristic = heuristic;
    }

    /// Softly removes a clause from the expression.
    /// This means that the clause is not actually removed from the expression vector,
    /// but all references to it have been removed from the literals map, so it is unreferenced.
    fn remove_clause(&mut self, clause_id: ClauseId) {
        // Remove all of the literals in the clause from the variable_to_clause map
        for i in 0..self.clauses[clause_id as usize].len() {
            let literal = unsafe { self.clauses.get_unchecked(clause_id as usize).get(i) };
            let literal_clauses = self.literal_to_clause.get_mut(&literal).unwrap();

            // If there are no more clauses that contain the literal, the negation is a pure literal
            if literal_clauses.is_empty() {
                // This literal has no more instances.
                // If its negation has some number of instances, add it to the pure_literals set.
                let negated_literal = negate(literal);
                let negated_literal_clauses = self.literal_to_clause.get_mut(&negated_literal);

                if negated_literal_clauses.is_none() || negated_literal_clauses.unwrap().is_empty()
                {
                    self.pure_literals.insert(negated_literal);
                }
            }
        }

        self.num_active_clauses -= 1;
        self.unit_clauses.remove(&clause_id);
        self.actions
            .write()
            .unwrap()
            .push(Action::RemoveClause(clause_id));
    }

    /// Re-enables a clause that had been softly removed, so all of its literals are still present in the vector.
    fn enable_clause(&mut self, clause_id: ClauseId) {
        self.num_active_clauses += 1;

        let clause = unsafe { &self.clauses.get_unchecked(clause_id as usize) };
        if clause.len() == 1 {
            self.unit_clauses.insert(clause_id);
        }

        for i in 0..clause.len() {
            let literal = unsafe { self.clauses.get_unchecked(clause_id as usize).get(i) };
            let should_check_pure_literal;
            {
                let literal_clauses = self.literal_to_clause.get_mut(&literal).unwrap();
                literal_clauses.insert(clause_id);
                should_check_pure_literal = literal_clauses.len() == 1;
            }

            if should_check_pure_literal {
                // TODO: Can we avoid doing this check again? Does it do too much?
                self.check_pure_literal(literal);
            }
        }
    }

    /// Removes a literal from all of the clauses that it is in
    fn remove_literal_from_clauses(&mut self, literal: Literal) {
        let clauses_result = self.literal_to_clause.get(&literal);
        if clauses_result.is_none() {
            return;
        }

        let actions = self.actions.clone();
        let mut actions = actions.write().unwrap();

        actions.push(Action::RemoveLiteralFromClausesStart());

        let literal_clauses = clauses_result.unwrap();
        for clause_id in literal_clauses {
            let clause = &mut self.clauses[*clause_id as usize];
            clause.remove(literal);

            if clause.len() == 1 {
                self.unit_clauses.insert(*clause_id);
            }

            if clause.is_empty() {
                self.num_empty_clauses += 1;
                self.unit_clauses.remove(clause_id);
            }

            actions.push(Action::RemoveLiteralFromClause(*clause_id));
        }

        actions.push(Action::RemoveLiteralFromClausesEnd(literal));
    }

    /// Removes all clauses with the specified literal.
    fn remove_clauses_with_literal(&mut self, literal: Literal) {
        let literal_clauses;
        {
            let literal_clauses_ref = self.literal_to_clause.get(&literal);
            if literal_clauses_ref.is_none() {
                return;
            }
            // TODO: Prevent cloning
            literal_clauses = literal_clauses_ref.unwrap().clone();
        }

        for clause_id in literal_clauses {
            self.remove_clause(clause_id);
        }
    }

    fn check_pure_literal(&mut self, literal: Literal) {
        let negated_literal = negate(literal);
        let literal_clauses = self.literal_to_clause.get(&literal);
        let has_instances = literal_clauses.is_some() && !literal_clauses.unwrap().is_empty();

        let negated_literal_clauses = self.literal_to_clause.get(&negated_literal);
        let negated_has_instances =
            negated_literal_clauses.is_some() && !negated_literal_clauses.unwrap().is_empty();

        if has_instances && !negated_has_instances {
            self.pure_literals.insert(literal);
            self.pure_literals.remove(&negated_literal);
        } else if !has_instances && negated_has_instances {
            self.pure_literals.insert(negated_literal);
            self.pure_literals.remove(&literal);
        } else {
            self.pure_literals.remove(&literal);
            self.pure_literals.remove(&negated_literal);
        }
    }

    fn assign_variable(&mut self, variable: Variable, value: bool) {

        self.assignments.insert(variable, value);
        self.actions
            .write()
            .unwrap()
            .push(Action::AssignVariable(variable));
        let literal = if value {
            variable as Literal
        } else {
            -(variable as Literal)
        };
        let negated_literal = negate(literal);
        self.remove_clauses_with_literal(literal);
        self.remove_literal_from_clauses(negated_literal);

        self.pure_literals.remove(&literal);
        self.pure_literals.remove(&negated_literal);
    }

    fn unassign_variable(&mut self, variable: Variable) {
        self.assignments.remove(&variable);
    }

    pub fn optimize(&mut self) {
        // Remove all of the empty clauses
        self.actions = Arc::new(RwLock::new(Stack::new(
            self.clauses.len() * self.max_clause_length,
        ))); // Pre-allocate a reasonable amount of space
    }

    pub fn is_satisfied_by(&self, assignment: &Assignment) -> bool {
        for clause in &self.clauses {
            let mut satisfied = false;
            for literal in clause.literals() {
                let variable = to_variable(*literal);
                let value = assignment.get(&variable);
                if value.is_none() {
                    continue;
                }

                if *value.unwrap() == (*literal > 0) {
                    satisfied = true;
                    break;
                }
            }

            if !satisfied {
                return false;
            }
        }

        true
    }

    fn get_most_literal_occurances(&self) -> (Variable, bool) {
        let mut max_occurances = 0;
        let mut best_literal = 0;

        for literal_clause in &self.literal_to_clause {
            let literal = literal_clause.0;
            if literal_clause.1.is_empty() || self.assignments.contains_key(&to_variable(*literal))
            {
                continue;
            }

            let occurances = literal_clause.1.len();
            if occurances > max_occurances {
                max_occurances = occurances;
                best_literal = *literal;
            }
        }

        if best_literal != 0 {
            return (to_variable(best_literal), best_literal > 0);
        }

        panic!("No branch variable found");
    }

    fn get_most_variable_occurances(&self) -> (Variable, bool) {
        let mut max_occurances = 0;
        let mut best_variable = 0;

        for variable in &self.variables {
            let positive_literal = *variable as Literal;
            let negative_literal = -positive_literal;

            if self.assignments.contains_key(variable) {
                continue;
            }

            let positive_occurances = self.literal_to_clause.get(&positive_literal).unwrap().len();
            let negative_occurances = self.literal_to_clause.get(&negative_literal).unwrap().len();

            let occurances = positive_occurances + negative_occurances;
            if occurances > max_occurances {
                max_occurances = occurances;
                best_variable = *variable;
            }
        }

        if best_variable != 0 {
            return (best_variable, true);
        }

        panic!("No branch variable found");
    }

    const ALPHA: usize = 1;
    const BETA: usize = 1;
    fn get_lexicographically_maximizing_literal(&self) -> (Variable, bool) {
        let mut best_variables = self
            .variables
            .iter()
            .filter(|x| !self.assignments.contains_key(*x))
            .collect::<Vec<&Variable>>();

        for clause_size in 2..5 {
            let mut best_heuristic_value = 0;
            let mut new_best_variables: Vec<&Variable> = Vec::new();

            for variable in best_variables {
                let positive_literal = *variable as Literal;
                let negative_literal = -positive_literal;

                let positive_occurrences = self
                    .literal_to_clause
                    .get(&positive_literal)
                    .unwrap()
                    .iter()
                    .filter(|clause_id| self.clauses[**clause_id as usize].len() == clause_size)
                    .count();
                let negative_occurences = self
                    .literal_to_clause
                    .get(&negative_literal)
                    .unwrap()
                    .iter()
                    .filter(|clause_id| self.clauses[**clause_id as usize].len() == clause_size)
                    .count();

                let heuristic_value = Self::ALPHA * max(positive_occurrences, negative_occurences)
                    + Self::BETA * min(positive_occurrences, negative_occurences);

                match heuristic_value.cmp(&best_heuristic_value) {
                    Ordering::Greater => {
                        best_heuristic_value = heuristic_value;
                        new_best_variables.clear();
                        new_best_variables.push(variable);
                    }
                    Ordering::Equal => {
                        new_best_variables.push(variable);
                    }
                    _ => {}
                }
            }

            best_variables = new_best_variables;

            if best_variables.len() == 1 {
                break;
            }
        }

        let variable = *best_variables[0];
        let positive_literal = variable as Literal;
        let negative_literal = -positive_literal;

        let positive_occurrences = self.literal_to_clause.get(&positive_literal).unwrap().len();
        let negative_occurrences = self.literal_to_clause.get(&negative_literal).unwrap().len();

        (variable, positive_occurrences > negative_occurrences)
    }
}

impl CNF for Expression {
    fn add_clause(&mut self, clause: Clause) {
        let clause_id = self.clauses.len() as ClauseId;

        for literal in clause.literals() {
            {
                let variable: Variable = to_variable(*literal);
                self.variables.insert(variable);

                if !self.literal_to_clause.contains_key(literal) {
                    self.literal_to_clause.insert(*literal, HashSet::new());
                }

                if !self.literal_to_clause.contains_key(&negate(*literal)) {
                    self.literal_to_clause
                        .insert(negate(*literal), HashSet::new());
                }

                let literal_clauses = self.literal_to_clause.get_mut(literal).unwrap();
                literal_clauses.insert(clause_id);
            }
            // Check if the literal is a pure literal
            self.check_pure_literal(*literal);
        }

        // Make sure we add it if it is a unit clause
        if clause.len() == 1 {
            self.unit_clauses.insert(clause_id);
        }

        if clause.len() > self.max_clause_length {
            self.max_clause_length = clause.len();
        }

        self.clauses.push(clause);
        self.num_active_clauses += 1;
    }

    fn remove_unit_clause(&mut self) -> Option<ClauseId> {
        if self.unit_clauses.is_empty() {
            return None;
        }

        let clause_id: ClauseId = *self.unit_clauses.iter().next().unwrap();

        let literal = unsafe { self.clauses.get_unchecked(clause_id as usize).literals()[0] };

        self.assign_variable(to_variable(literal), literal > 0);
        Some(clause_id)
    }

    fn remove_pure_literal(&mut self) -> Option<Literal> {
        if self.pure_literals.is_empty() {
            return None;
        }

        let literal: Literal = *self.pure_literals.iter().next().unwrap();

        self.assign_variable(to_variable(literal), literal > 0);
        Some(literal)
    }

    fn construct_assignment(&mut self) -> Assignment {
        let mut assignments = HashMap::new();

        // Copy the existing assignments array to another one
        for (k, v) in self.assignments.iter() {
            assignments.insert(*k, *v);
        }

        // Assign all of the remaining variables to true
        for variable in &self.variables {
            if !assignments.contains_key(variable) {
                assignments.insert(*variable, true);
            }
        }
        assignments
    }

    #[inline]
    fn is_satisfied(&self) -> bool {
        self.num_active_clauses == 0
    }

    #[inline]
    fn is_unsatisfiable(&self) -> bool {
        self.num_empty_clauses > 0
    }

    fn get_branch_variable(&self) -> (Variable, bool) {
        match self.heuristic {
            SolverHeuristic::MostLiteralOccurances => self.get_most_literal_occurances(),
            SolverHeuristic::MostVariableOccurances => self.get_most_variable_occurances(),
            SolverHeuristic::MinimizeClauseLength => {
                self.get_lexicographically_maximizing_literal()
            }
        }
    }

    fn branch_variable(&mut self, variable: Variable, value: bool) {
        self.assign_variable(variable, value);
    }

    fn get_action_state(&self) -> ActionState {
        return self.actions.read().unwrap().len();
    }

    fn restore_action_state(&mut self, state: ActionState) {
        let actions = self.actions.clone();
        let mut actions = actions.write().unwrap();
        while actions.len() > state {
            let action = actions.pop().unwrap();
            match action {
                Action::RemoveClause(clause_id) => self.enable_clause(clause_id),
                Action::RemoveLiteralFromClausesEnd(literal) => {
                    let removing_literal_clauses =
                        self.literal_to_clause.get_mut(&literal).unwrap();

                    let mut should_exit = false;

                    while !should_exit {
                        let next_action = actions.pop().unwrap();
                        match next_action {
                            Action::RemoveLiteralFromClause(clause_id) => {
                                let clause =
                                    unsafe { self.clauses.get_unchecked_mut(clause_id as usize) };
                                clause.insert(literal);
                                if clause.len() == 1 {
                                    self.num_empty_clauses -= 1;
                                    self.unit_clauses.insert(clause_id);
                                } else if clause.len() == 2 {
                                    self.unit_clauses.remove(&clause_id);
                                }

                                removing_literal_clauses.insert(clause_id);
                            }
                            Action::RemoveLiteralFromClausesStart() => {
                                should_exit = true;
                            }
                            _ => panic!("Did not encounter a start literal!"),
                        }
                    }
                }
                Action::AssignVariable(variable) => {
                    self.unassign_variable(variable);
                }
                _ => break,
            }
        }
    }

    /// Inference is possibly when there are some "Active" clauses, 
    /// and either pure literals or unit clauses.
    fn is_inference_possible(&self) -> bool {
        self.num_empty_clauses == 0
            && self.num_active_clauses > 0
            && (!self.pure_literals.is_empty() || !self.unit_clauses.is_empty())
    }
}