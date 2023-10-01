use super::linked_list;

pub struct Stack<T> {
    size: usize,
    values: linked_list::LinkedList<T>
}

impl<T> Stack<T> {
    pub fn top(&self) -> usize {
        self.size
    }

    pub fn new() -> Self {
        Self {
            size: 0,
            values: linked_list::LinkedList::new(),
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.values.peek()
    }

    pub fn offset_peek(&self, offset: usize) -> Option<&T> {
        self.values.offset_peek(offset)
    }

    pub fn push(&mut self, value: T) {
        self.size += 1;
        self.values.push_front(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.size -= 1;
        self.values.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn empty_stack() {
        let stack = Stack::<usize>::new();
        assert_eq!(stack.size, 0)
    }

    #[test]
    fn push_and_pop() {
        let mut stack = Stack::<i64>::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.peek().map(|v| { *v }), Some(2));
        assert_eq!(stack.size, 2);
        stack.push(33);
        assert_eq!(stack.peek().map(|v| { *v }), Some(33));
        assert_eq!(stack.size, 3);
        assert_eq!(stack.offset_peek(stack.size - 1).map(|v| { *v }), Some(1));

        assert_eq!(stack.pop(), Some(33));
        assert_eq!(stack.size, 2);
        assert_eq!(stack.peek().map(|v| { *v }), Some(2));

        stack.pop();
        stack.pop();
        assert_eq!(stack.size, 0)
    }
}