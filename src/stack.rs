/// A simple stack implementation.
#[derive(Clone)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    /// Initializes a new stack with the given capacity.
    pub fn new(capacity: usize) -> Stack<T> {
        Stack {
            stack: Vec::with_capacity(capacity),
        }
    }

    /// Pushes a new value onto the stack.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.stack.push(value);
    }

    /// Pops the top value from the stack.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    /// Returns the length of the stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Peeks at the top value of the stack.
    #[inline]
    pub fn peek(&self) -> &T {
        self.stack.last().unwrap()
    }

    /// Checks if the stack contains a value.
    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.stack.contains(value)
    }
}

impl Stack<(u16, bool)> {
    pub fn iter(&self) -> std::slice::Iter<(u16, bool)> {
        self.stack.iter()
    }
}

impl IntoIterator for Stack<(u16, bool)> {
    type Item = (u16, bool);
    type IntoIter = std::vec::IntoIter<(u16, bool)>;

    fn into_iter(self) -> Self::IntoIter {
        self.stack.into_iter()
    }
}