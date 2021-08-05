use crate::iter::{Iter, Iterator};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type RcNode<T> = Rc<RefCell<LinkNode<T>>>;

pub type WeakNode<T> = Weak<RefCell<LinkNode<T>>>;

pub struct LinkNode<T> {
    pub val: Option<T>,
    next: Option<RcNode<T>>,
    prev: Option<WeakNode<T>>,
}

impl<T> LinkNode<T> {
    #[inline]
    fn wrap(val: Option<T>) -> RcNode<T> {
        Rc::new(RefCell::new(Self {
            val,
            next: None,
            prev: None,
        }))
    }
}

#[derive(Clone)]
pub struct LinkList<T> {
    len: usize,
    head: RcNode<T>,
    tail: WeakNode<T>,
}

impl<T> LinkList<T> {
    #[inline]
    fn first_node(&self) -> RcNode<T> {
        (*self.head).borrow_mut().next.clone().unwrap()
    }

    #[inline]
    fn last_node(&self) -> RcNode<T> {
        (*self.tail.upgrade().unwrap())
            .borrow_mut()
            .prev
            .as_mut()
            .unwrap()
            .upgrade()
            .unwrap()
    }

    fn insert_between(after: RcNode<T>, before: RcNode<T>, item: RcNode<T>) {
        (*before).borrow_mut().prev = Some(Rc::downgrade(&item));
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(before);
            item.prev = Some(Rc::downgrade(&after));
        }
        (*after).borrow_mut().next = Some(item);
    }

    fn remove(at: RcNode<T>) -> RcNode<T> {
        let prev = (*at).borrow_mut().prev.take().unwrap().upgrade().unwrap();
        let next = (*at).borrow_mut().next.take().unwrap();
        (*next).borrow_mut().prev = Some(Rc::downgrade(&prev));
        (*prev).borrow_mut().next = Some(next);
        at
    }
}

impl<T> LinkList<T> {
    pub fn new() -> Self {
        let head = LinkNode::wrap(None);
        let tail = LinkNode::wrap(None);
        (*tail).borrow_mut().prev = Some(Rc::downgrade(&head));
        let weak_tail = Rc::downgrade(&tail);
        (*head).borrow_mut().next = Some(tail);
        Self {
            len: 0,
            head,
            tail: weak_tail,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push_back_node(&mut self, item: RcNode<T>) -> WeakNode<T> {
        self.len += 1;
        let item_weak = Rc::downgrade(&item);
        Self::insert_between(self.last_node(), self.tail.upgrade().unwrap(), item);
        item_weak
    }

    pub fn push_front_node(&mut self, item: RcNode<T>) -> WeakNode<T> {
        self.len += 1;
        let item_weak = Rc::downgrade(&item);
        Self::insert_between(self.head.clone(), self.first_node(), item);
        item_weak
    }

    pub fn push_back(&mut self, item: T) -> WeakNode<T> {
        Self::push_back_node(self, LinkNode::wrap(Some(item)))
    }

    pub fn push_front(&mut self, item: T) -> WeakNode<T> {
        Self::push_front_node(self, LinkNode::wrap(Some(item)))
    }

    pub fn pop_back_node(&mut self) -> Option<RcNode<T>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(Self::remove(Self::last_node(&self)))
    }

    pub fn pop_front_node(&mut self) -> Option<RcNode<T>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(Self::remove(self.first_node()))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(back) = self.pop_back_node() {
            Some(Rc::try_unwrap(back).ok().unwrap().into_inner().val.unwrap())
        } else {
            None
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(front) = self.pop_front_node() {
            Some(
                Rc::try_unwrap(front)
                    .ok()
                    .unwrap()
                    .into_inner()
                    .val
                    .unwrap(),
            )
        } else {
            None
        }
    }
}

impl<T> Iterator<RcNode<T>> for LinkList<T> {
    fn iter(&self) -> Iter<Self, RcNode<T>> {
        Iter {
            data: self,
            cur: self.head.clone(),
        }
    }

    fn rev_iter(&self) -> Iter<Self, RcNode<T>> {
        Iter {
            data: self,
            cur: self.tail.upgrade().unwrap(),
        }
    }

    fn next(iter: &mut Iter<Self, RcNode<T>>) -> Option<RcNode<T>> {
        let n = (*iter.cur).borrow().next.clone().unwrap();
        if Rc::ptr_eq(&n, &iter.data.tail.upgrade().unwrap()) {
            return None;
        }
        iter.cur = n.clone();
        Some(n)
    }

    fn prev(iter: &mut Iter<Self, RcNode<T>>) -> Option<RcNode<T>> {
        let p = (*iter.cur)
            .borrow()
            .prev
            .clone()
            .unwrap()
            .upgrade()
            .unwrap();
        if Rc::ptr_eq(&p, &iter.data.head) {
            return None;
        }
        iter.cur = p.clone();
        Some(p)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn init_list(v: &[i32]) -> LinkList<i32> {
        let mut list = LinkList::new();
        for n in v {
            list.push_back(*n);
        }
        list
    }

    fn check_list(list: &LinkList<i32>, v: Vec<i32>) {
        assert_eq!(list.len(), v.len());
        let mut prev = list.head.clone();
        for x in v.iter() {
            let n = (*prev).borrow().next.clone().unwrap();
            assert_eq!((*n).borrow().val.unwrap(), *x);
            prev = n;
        }
        assert_eq!(
            Rc::downgrade((*prev).borrow().next.as_ref().unwrap()).as_ptr(),
            list.tail.as_ptr()
        );
        let mut next = list.tail.upgrade().unwrap();
        for x in v.into_iter().rev() {
            let n = (*next).borrow().prev.clone().unwrap().upgrade().unwrap();
            assert_eq!((*n).borrow().val.unwrap(), x);
            next = n;
        }
        assert_eq!(
            (*next).borrow().prev.as_ref().unwrap().as_ptr(),
            Rc::downgrade(&list.head).as_ptr()
        );
    }

    #[test]
    fn test_push_back() {
        let v = vec![1, 2, 3];
        let list = init_list(&v);
        check_list(&list, v);
    }

    #[test]
    fn test_push_front() {
        let mut list = LinkList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);
        check_list(&list, vec![1, 2, 3]);
    }

    #[test]
    fn test_push() {
        let mut list = LinkList::new();
        list.push_front(2);
        list.push_back(3);
        list.push_back(4);
        list.push_front(1);
        check_list(&list, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_pop_back() {
        let mut list = init_list(&[1, 2, 3]);
        assert_eq!(list.pop_back().unwrap(), 3);
        check_list(&list, vec![1, 2]);
        assert_eq!(list.pop_back().unwrap(), 2);
        check_list(&list, vec![1]);
        assert_eq!(list.pop_back().unwrap(), 1);
        check_list(&list, vec![]);
        assert!(list.pop_back().is_none());
    }

    #[test]
    fn test_pop_front() {
        let mut list = init_list(&[1, 2, 3]);
        assert_eq!(list.pop_front().unwrap(), 1);
        check_list(&list, vec![2, 3]);
        assert_eq!(list.pop_front().unwrap(), 2);
        check_list(&list, vec![3]);
        assert_eq!(list.pop_front().unwrap(), 3);
        check_list(&list, vec![]);
        assert!(list.pop_back().is_none());
    }

    #[test]
    fn test_iter() {
        let v = vec![1, 2, 3];
        let list = init_list(&v);
        let mut iter = list.iter();
        let mut idx = 0;
        while let Some(n) = iter.next() {
            let val = n.borrow().val.unwrap();
            assert_eq!(val, v[idx]);
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_rev_iter() {
        let v = vec![1, 2, 3];
        let list = init_list(&v);
        let mut iter = list.rev_iter();
        let mut idx = v.len();
        while let Some(n) = iter.prev() {
            let val = n.borrow().val.unwrap();
            idx -= 1;
            assert_eq!(val, v[idx]);
        }
        assert_eq!(idx, 0);
    }
}
