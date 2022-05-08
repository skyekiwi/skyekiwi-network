use std::{fmt::Debug};
use std::ops::{Index, IndexMut};

#[derive(Debug, PartialEq)]
pub enum DiffOp<'a> {
    Equal(WrappedBytes<'a>, WrappedBytes<'a>),
    Insert(WrappedBytes<'a>),
    Delete(WrappedBytes<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConsolidatedDiffOp {
    Equal(usize, usize),
    Insert(Vec<u8>),
    Delete(usize, usize),
}

#[derive(Debug, PartialEq)]
pub struct WrappedBytes<'a> {
    inner: &'a [u8],
    offset: usize,
    len: usize,
}

impl Copy for WrappedBytes<'_> {}

impl Clone for WrappedBytes<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <'a> WrappedBytes <'a> {

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn new(inner: &'a [u8], bounds: impl RangeBounds) -> Self {
        let (offset, len) = bounds.index(inner.len());
        WrappedBytes { inner, offset, len, }
    }

    pub fn slice(&self, bounds: impl RangeBounds) -> Self {
        let (offset, len) = bounds.index(self.len);
        WrappedBytes {
            inner: self.inner,
            offset: self.offset + offset, len
        }
    }

    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        (self.slice(..mid), self.slice(mid..))
    }

    pub fn get(&self, bounds: impl RangeBounds) -> Option<Self> {
        let (offset, len) = bounds.try_index(self.len)?;
        Some(WrappedBytes {
            inner: self.inner,
            offset: self.offset + offset, len
        })
    }

    pub fn inner_get(&self, bounds: impl RangeBounds) -> Vec<u8> {
        let (offset, len) = bounds.try_index(self.len).expect("wrong range");
        self.inner[self.offset + offset..self.offset + offset + len].to_vec()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn dump(&self) -> Vec<u8> {
        self.inner[self.offset..self.offset + self.len].to_vec()
    }
}

pub trait RangeBounds: Sized + Clone + Debug {
    // Returns (offset, len).
    fn try_index(self, len: usize) -> Option<(usize, usize)>;

    fn index(self, len: usize) -> (usize, usize) {
        match self.clone().try_index(len) {
            Some(range) => range,
            None => panic!("index out of range, index={:?}, len={}", self, len),
        }
    }
}

impl RangeBounds for std::ops::Range<usize> {
    fn try_index(self, len: usize) -> Option<(usize, usize)> {
        if self.start <= self.end && self.end <= len {
            Some((self.start, self.end - self.start))
        } else {
            None
        }
    }
}

impl RangeBounds for std::ops::RangeFrom<usize> {
    fn try_index(self, len: usize) -> Option<(usize, usize)> {
        if self.start <= len {
            Some((self.start, len - self.start))
        } else {
            None
        }
    }
}

impl RangeBounds for std::ops::RangeTo<usize> {
    fn try_index(self, len: usize) -> Option<(usize, usize)> {
        if self.end <= len {
            Some((0, self.end))
        } else {
            None
        }
    }
}

impl RangeBounds for std::ops::RangeFull {
    fn try_index(self, len: usize) -> Option<(usize, usize)> {
        Some((0, len))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct V {
    offset: isize,
    v: Vec<usize>, // Look into initializing this to -1 and storing isize
}

impl V {
    pub(crate) fn new(max_d: usize) -> Self {
        Self {
            offset: max_d as isize,
            v: vec![0; 2 * max_d],
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.v.len()
    }
}

impl Index<isize> for V {
    type Output = usize;

    fn index(&self, index: isize) -> &Self::Output {
        &self.v[(index + self.offset) as usize]
    }
}

impl IndexMut<isize> for V {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        &mut self.v[(index + self.offset) as usize]
    }
}