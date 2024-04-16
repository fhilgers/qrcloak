use bytes::Bytes;

use crate::format::Index;

pub struct Splits {
    data: Bytes,
    quo: usize,
    long_count: usize,
}

impl Splits {
    pub fn new(data: Bytes, splits: usize) -> Self {
        let len = data.len();
        assert!(len >= splits);
        assert_ne!(splits, 0);

        let (quo, rem) = (len / splits, len % splits);

        Self {
            data,
            quo,
            long_count: rem,
        }
    }
}

impl Iterator for Splits {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.long_count > 0 {
            self.long_count -= 1;

            let data = self.data.split_to(self.quo + 1);
            Some(data)
        } else if self.data.len() > 0 {
            let data = self.data.split_to(self.quo);
            Some(data)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexIter {
    index: Index,
}

impl IndexIter {
    pub fn new(size: u32) -> Self {
        IndexIter {
            index: Index {
                id: rand::random(),
                index: 0,
                size,
            },
        }
    }
}

impl Iterator for IndexIter {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.index == self.index.size {
            None
        } else {
            let index = self.index;
            self.index.index += 1;
            Some(index)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index_iter() {
        let size = 4;

        let iter = IndexIter::new(size);

        let mut id = None;

        let mut count = 0;
        for (i, index) in iter.enumerate() {
            if id.is_none() {
                id = Some(index.id);
            }

            count += 1;

            assert_eq!(id, Some(index.id));
            assert_eq!(i, index.index as usize);
            assert_eq!(4, index.size);
        }

        assert_eq!(count, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_with_rem() {
        let data = Bytes::from("hello");
        let mut splits = Splits::new(data, 4);

        assert_eq!(splits.next(), Some(Bytes::from("he")));
        assert_eq!(splits.next(), Some(Bytes::from("l")));
        assert_eq!(splits.next(), Some(Bytes::from("l")));
        assert_eq!(splits.next(), Some(Bytes::from("o")));
        assert_eq!(splits.next(), None);
    }

    #[test]
    fn test_without_rem() {
        let data = Bytes::from("hello");
        let mut splits = Splits::new(data, 5);

        assert_eq!(splits.next(), Some(Bytes::from("h")));
        assert_eq!(splits.next(), Some(Bytes::from("e")));
        assert_eq!(splits.next(), Some(Bytes::from("l")));
        assert_eq!(splits.next(), Some(Bytes::from("l")));
        assert_eq!(splits.next(), Some(Bytes::from("o")));
        assert_eq!(splits.next(), None);
    }
}
