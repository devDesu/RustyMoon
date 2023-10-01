type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

pub struct LinkedList<T> {
    head: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /**
     * Push front
     * head -> next -> next ... turns into
     * new_head -> prev_head -> next ...
     */
    pub fn push_front(&mut self, value: T) {
        let new_node = Box::new(Node::<T> {
            value,
            next: self.head.take()
        });

        self.head = Some(new_node);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.head.take() {
            None => None,
            Some(old_head) => {
                self.head = old_head.next;
                Some(old_head.value)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(| node | { &node.value })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_ref().map(| node | { &mut node.as_mut().value })
    }

    pub fn offset_peek(&self, offset: usize) -> Option<&T> {
        let mut current = &self.head;
        for _ in 0..offset {
            match current {
                None => { return None; },
                Some(node) => {
                    current = &node.next
                }
            }
        }

        current.as_ref().map( |node| { &node.value } )
    }
} 

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut boxed_node) = current {
            current = boxed_node.next.take();
        }
    }
}