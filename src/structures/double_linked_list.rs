type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
    prev: Link<T>,
}

pub struct DoubleLinkedList<T> {
    pub head: Link<T>,
    pub tail: Link<T>,
}

impl<T> DoubleLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    pub fn push_front(&mut self, value: T) {
        if (self.head.is_none() && self.tail.is_some()) || (self.head.is_some() && self.tail.is_none()) {
            panic!("Linked list consistency broken");
        }

        if self.is_empty() {
            let mut node = Box::new(Node::<T> {
                value,
                next: None,
                prev: None
            });
            self.head = Some(node);
        } else {
            let mut new_head = Box::new(Node::<T> {
                value,
                prev: None,
                next: self.head,
            });

            // if tail was none old head turns into new tail
            if self.tail.is_none() {
                self.tail = self.head;
            }
            self.head = Some(new_head);
        }
    }

    pub fn pop_front(&mut self) -> Option<&T> {
        if !self.is_empty() {
            let old_head = self.head.unwrap();
            let new_head = old_head.next;
            self.head = new_head;
            Some(&old_head.value)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(| node | { &node.value })
    }
} 