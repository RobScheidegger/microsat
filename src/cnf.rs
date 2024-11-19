use hashbrown::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    RemoveClause(ClauseId),
    RemoveLiteralFromClausesStart(),
    RemoveLiteralFromClause(ClauseId),
    RemoveLiteralFromClausesEnd(Literal),
    AssignVariable(Variable),
}

pub type Assignment = HashMap<Variable, bool>;
pub type ClauseId = u16;
pub type Literal = i16;
pub type Variable = u16;
pub type ActionState = usize;

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Clause {
    variables: Vec<Literal>,
}

pub trait CNF {
    /// Adds a new clause to the CNF representation.
    fn add_clause(&mut self, clause: Clause);

    /// Removes a unit clause (if it exists) from the CNF and returns it.
    fn remove_unit_clause(&mut self) -> Option<ClauseId>;

    /// Removes a pure literal (if it exists) from the CNF and returns it.
    fn remove_pure_literal(&mut self) -> Option<Literal>;

    /// Constructs an assignment from the current state of the CNF.
    /// This is only valid if the CNF is satisfiable.
    fn construct_assignment(&mut self) -> Assignment;

    /// Returns true if the CNF is satisfiable.
    fn is_satisfied(&self) -> bool;

    fn is_unsatisfiable(&self) -> bool;

    fn get_action_state(&self) -> ActionState;

    fn restore_action_state(&mut self, state: ActionState);

    fn is_inference_possible(&self) -> bool;

    fn get_branch_variable(&self) -> (Variable, bool);

    fn branch_variable(&mut self, variable: Variable, value: bool);
}

impl Clause {
    pub fn new() -> Clause {
        Clause {
            variables: Vec::new(),
        }
    }

    #[inline]
    pub fn insert_checked(&mut self, variable: Literal) {
        if !self.variables.contains(&variable) {
            self.variables.push(variable);
        }
    }

    #[inline]
    pub fn insert(&mut self, variable: Literal) {
        self.variables.push(variable);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    #[inline]
    pub fn contains(&self, variable: Literal) -> bool {
        self.variables.contains(&variable)
    }

    #[inline]
    pub fn literals(&self) -> &Vec<Literal> {
        &self.variables
    }

    #[inline]
    pub fn get(&self, index: usize) -> Literal {
        unsafe {
            return *self.variables.get_unchecked(index);
        }
    }

    /// Efficient remove for a clause set that uses constant time by swapping
    /// the last element with the removed one.
    pub fn remove(&mut self, variable: Literal) {
        for i in 0..self.variables.len() {
            if self.variables[i] == variable {
                self.variables.swap_remove(i);
                return;
            }
        }
    }
}

#[inline]
pub fn to_variable(literal: Literal) -> Variable {
    literal.unsigned_abs()
}

#[inline]
pub fn negate(variable: Literal) -> Literal {
    -variable
}

#[inline]
pub fn to_positive(variable: Literal) -> Literal {
    if variable > 0 {
        variable
    } else {
        -variable
    }
}