use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct LinkNode<T> {
    pub val: Option<T>,
    next: Option<Rc<RefCell<LinkNode<T>>>>,
    prev: Option<Weak<RefCell<LinkNode<T>>>>,
}

pub struct LinkList<T> {
    len: usize,
    head: Rc<RefCell<LinkNode<T>>>,
    tail: Weak<RefCell<LinkNode<T>>>,
}

impl<T> LinkList<T> {
    fn insert_before(at: Rc<RefCell<LinkNode<T>>>, item: Rc<RefCell<LinkNode<T>>>) {
        let prev = (*at)
            .borrow_mut()
            .prev
            .replace(Rc::downgrade(&item))
            .unwrap();
        let prev_strong = Weak::upgrade(&prev).unwrap();
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(at);
            item.prev = Some(prev);
        }
        (*prev_strong).borrow_mut().next = Some(item);
    }

    fn insert_after(at: Rc<RefCell<LinkNode<T>>>, item: Rc<RefCell<LinkNode<T>>>) {
        let next = (*at).borrow_mut().next.replace(Rc::clone(&item)).unwrap();
        (*next).borrow_mut().prev = Some(Rc::downgrade(&item));
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(next);
            item.prev = Some(Rc::downgrade(&at));
        }
    }
}

impl<T> LinkList<T> {
    pub fn new() -> Self {
        let head = Rc::new(RefCell::new(LinkNode {
            val: None,
            next: None,
            prev: None,
        }));
        let tail = Rc::new(RefCell::new(LinkNode {
            val: None,
            next: None,
            prev: None,
        }));
        (*tail).borrow_mut().prev = Some(Rc::downgrade(&head));
        let weak_tail = Rc::downgrade(&tail);
        (*head).borrow_mut().next = Some(tail);
        Self {
            len: 0,
            head,
            tail: weak_tail,
        }
    }

    pub fn push_back(&mut self, item: T) -> Weak<RefCell<LinkNode<T>>> {
        let item = Rc::new(RefCell::new(LinkNode {
            val: Some(item),
            next: None,
            prev: None,
        }));
        let item_weak = Rc::downgrade(&item);
        Self::insert_before(Weak::upgrade(&self.tail).unwrap(), item);
        item_weak
    }

    pub fn push_front(&mut self, item: T) -> Weak<RefCell<LinkNode<T>>> {
        let item = Rc::new(RefCell::new(LinkNode {
            val: Some(item),
            next: None,
            prev: None,
        }));
        let item_weak = Rc::downgrade(&item);
        Self::insert_after(Rc::clone(&self.head), item);
        item_weak
    }
}
