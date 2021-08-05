pub trait Iterator<T: Clone>: Sized {
    fn iter(&self) -> Iter<Self, T>;
    fn rev_iter(&self) -> Iter<Self, T>;
    fn next(iter: &mut Iter<Self, T>) -> Option<T>;
    fn prev(iter: &mut Iter<Self, T>) -> Option<T>;
}

pub struct Iter<'a, T1: Iterator<T2>, T2: Clone> {
    pub data: &'a T1,
    pub cur: T2,
}

impl<'a, T1: Iterator<T2>, T2: Clone> Iter<'a, T1, T2> {
    pub fn next(&mut self) -> Option<T2> {
        T1::next(self)
    }

    pub fn prev(&mut self) -> Option<T2> {
        T1::prev(self)
    }
}
