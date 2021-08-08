use rand::prelude::*;
use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt::Debug,
    rc::{Rc, Weak},
};

type RcNode<T> = Rc<RefCell<SkipNode<T>>>;
type WeakNode<T> = Weak<RefCell<SkipNode<T>>>;

pub struct SkipNode<T> {
    pub val: Option<T>,
    next: Option<RcNode<T>>,
    prev: Option<WeakNode<T>>,
    below: Option<RcNode<T>>,
    above: Option<WeakNode<T>>,
}

impl<T> SkipNode<T> {
    fn new(val: Option<T>) -> Self {
        Self {
            val,
            next: None,
            prev: None,
            below: None,
            above: None,
        }
    }
}

pub struct SkipList<T: Ord + Clone + Debug> {
    head: RcNode<T>,
    rng: ThreadRng,
    len: usize,
}

impl<T: Ord + Clone + Debug> SkipList<T> {
    fn find(&self, item: &T) -> Option<RcNode<T>> {
        let mut cur = self.head.clone();
        loop {
            let curb = cur.borrow();
            if let Some(next) = curb.next.clone() {
                let nextb = next.borrow();
                match nextb.val.as_ref().unwrap().cmp(item) {
                    Ordering::Less => {
                        drop(nextb);
                        drop(curb);
                        cur = next;
                        continue;
                    }
                    Ordering::Equal => {
                        drop(nextb);
                        return Some(next);
                    }
                    _ => (),
                }
            }
            if let Some(below) = curb.below.clone() {
                drop(curb);
                cur = below;
            } else {
                return None;
            }
        }
    }

    fn insert_after_above(prev: RcNode<T>, below: Option<RcNode<T>>, item: RcNode<T>) {
        if let Some(below) = below {
            below.borrow_mut().above = Some(Rc::downgrade(&item));
            item.borrow_mut().below = Some(below);
        }
        if let Some(next) = prev.borrow().next.clone() {
            next.borrow_mut().prev = Some(Rc::downgrade(&item));
            item.borrow_mut().next = Some(next);
        }
        item.borrow_mut().prev = Some(Rc::downgrade(&prev));
        prev.borrow_mut().next = Some(item);
    }

    fn remove_node(at: RcNode<T>) -> Option<RcNode<T>> {
        let prev = at.borrow().prev.as_ref().unwrap().upgrade().unwrap();
        if let Some(next) = at.borrow().next.clone() {
            next.borrow_mut().prev = Some(Rc::downgrade(&prev));
            prev.borrow_mut().next = Some(next);
        } else {
            prev.borrow_mut().next = None;
        }
        at.borrow().below.clone()
    }
}

impl<T: Ord + Clone + Debug> SkipList<T> {
    pub fn new() -> Self {
        let rng = rand::thread_rng();
        let head = Rc::new(RefCell::new(SkipNode::new(None)));
        Self { rng, head, len: 0 }
    }

    pub fn get(&self, item: &T) -> Option<RcNode<T>> {
        self.find(item)
    }

    pub fn insert(&mut self, item: T) {
        let mut route = vec![];
        let mut cur = self.head.clone();
        loop {
            let curb = cur.borrow();
            if let Some(next) = curb.next.clone() {
                if next.borrow().val.as_ref().unwrap() < &item {
                    drop(curb);
                    cur = next;
                    continue;
                }
            }
            if let Some(below) = curb.below.clone() {
                drop(curb);
                route.push(cur);
                cur = below;
            } else {
                break;
            }
        }
        let prev = cur;
        let node = Rc::new(RefCell::new(SkipNode::new(Some(item.clone()))));
        Self::insert_after_above(prev, None, node.clone());
        let mut below = node;
        while self.rng.gen() {
            let node = Rc::new(RefCell::new(SkipNode::new(Some(item.clone()))));
            let prev = if let Some(prev) = route.pop() {
                prev
            } else {
                let head = Rc::new(RefCell::new(SkipNode::new(None)));
                self.head.borrow_mut().above = Some(Rc::downgrade(&head));
                head.borrow_mut().below = Some(self.head.clone());
                self.head = head.clone();
                head
            };
            Self::insert_after_above(prev, Some(below), node.clone());
            below = node;
        }
        self.len += 1;
    }

    pub fn remove(&mut self, item: &T) -> bool {
        let node = self.find(item);
        if let Some(mut node) = node {
            while let Some(below) = Self::remove_node(node) {
                node = below;
            }
            self.len -= 1;
            true
        } else {
            false
        }
    }
}
