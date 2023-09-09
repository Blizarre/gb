/// Vector Iterator which keep track of where it is in the original structure.
/// The `index()` method will return the index of the next element inside the original vec
/// at the time of the call.
pub struct IndexedIter<T> {
    it: std::vec::IntoIter<T>,
    index: usize,
}

impl<T> Iterator for IndexedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it.next() {
            Some(x) => {
                self.index += 1;
                Some(x)
            }
            None => None,
        }
    }
}

impl<T> IndexedIter<T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            it: vec.into_iter(),
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexedIter;

    #[test]
    fn test_indexed_iter_empty_vec() {
        let mut iter = IndexedIter::from_vec(Vec::<u8>::new());
        assert_eq!(iter.index(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.index(), 0);
    }

    #[test]
    fn test_indexed_iter_single_element_vec() {
        let mut iter = IndexedIter::from_vec(vec![1]);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.index(), 1);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.index(), 1);
    }

    #[test]
    fn test_indexed_iter_multi_element_vec() {
        let mut iter = IndexedIter::from_vec(vec![1, 2, 3]);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.index(), 1);

        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.index(), 2);

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.index(), 3);

        assert_eq!(iter.next(), None);
        assert_eq!(iter.index(), 3);
    }
}
