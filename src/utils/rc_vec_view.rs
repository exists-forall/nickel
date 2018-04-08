use std::rc::Rc;
use std::ops::{Range, Deref};
use std::cmp::{PartialEq, Eq};

/// A view into a particular range of a reference-counted vector.
///
/// This facilitates structural sharing when creating reference-counted vectors with common prefixes
/// or suffixes.
///
/// Note the the *entire* underlying vector is retained until the last view of it is dropped -- care
/// should be taken to make sure this does not introduce significant memory leaks!
#[derive(Clone, Debug)]
pub struct RcVecView<T> {
    data: Rc<Vec<T>>,
    range: Range<usize>,
}

impl<T> RcVecView<T> {
    pub fn new(data: Rc<Vec<T>>) -> Self {
        RcVecView {
            range: 0..data.len(),
            data,
        }
    }

    pub fn slice(&self, sub_range: Range<usize>) -> Self {
        let start = (self.range.start + sub_range.start).max(self.range.start);
        let end = (self.range.start + sub_range.end).min(self.range.end);
        RcVecView {
            data: self.data.clone(),
            range: start..end,
        }
    }
}

impl<T> Deref for RcVecView<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data[self.range.clone()]
    }
}

impl<T: PartialEq> PartialEq for RcVecView<T> {
    fn eq(&self, other: &RcVecView<T>) -> bool {
        (&self as &[T]) == (&other as &[T])
    }
}

impl<T: Eq> Eq for RcVecView<T> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_slicing() {
        let view1 = RcVecView::new(Rc::new(vec!["foo", "bar", "baz", "biz", "quux"]));
        assert_eq!(&*view1, &["foo", "bar", "baz", "biz", "quux"]);

        let view2 = view1.slice(0..view1.len());
        assert_eq!(&*view2, &*view1);
        assert_eq!(view2, view1);

        let view3 = view1.slice(1..4);
        assert_eq!(&*view3, &["bar", "baz", "biz"]);

        let view4 = view3.slice(1..3);
        assert_eq!(&*view4, &["baz", "biz"]);

        let view5 = RcVecView::new(Rc::new(vec!["baz", "biz"]));
        assert_eq!(view4, view5);
    }
}
