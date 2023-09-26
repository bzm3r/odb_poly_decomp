use std::cell::Cell;
use std::fmt::Debug;
use std::ops::Deref;


/// A vector that can only be pushed to: items cannot be removed.
///
/// Furthermore, items can only be indexed by types that implement
/// [`StableIndex`]. Examples of such indices are numerical indices (`usize`)
/// into a vector.
///
/// Together, `PushOnlyVec` and [`StableIndex`] provide
pub struct PushOnlyVec<T> {
    vec: CellOpt<Vec<T>>,
}

impl<T> PushOnlyVec<T> {
    pub fn push(&self, item: T)
}



pub trait StableIndex:
    Clone + Copy + Debug + Default + Deref<Target = CellOpt<usize>>
{
}

#[derive(Clone, Debug, Default)]
pub struct EdgeIdx(CellOpt<usize>);

impl From<&EdgeIdx> for Option<usize> {
    fn from(index: &EdgeIdx) -> Self {
        index.0.get()
    }
}

#[derive(Clone, Debug, Default)]
pub struct NodeIdx(CellOpt<usize>);

impl From<&NodeIdx> for Option<usize> {
    fn from(index: &NodeIdx) -> Self {
        index.0.get()
    }
}

pub struct CellOpt<T: Clone + Debug> {
    cell: Cell<Option<T>>,
}

impl<T: Clone + Debug + Copy> CellOpt<T> {
    pub fn get(&self) -> Option<T> {
        self.cell.get()
    }
}

impl<T: Clone + Debug> From<T> for CellOpt<T> {
    fn from(value: T) -> Self {
        CellOpt {
            cell: Cell::new(value.into()),
        }
    }
}

impl<T: Clone + Debug> From<Option<T>> for CellOpt<T> {
    fn from(value: Option<T>) -> Self {
        CellOpt { cell: value.into() }
    }
}

impl<T: Clone + Debug> Default for CellOpt<T> {
    fn default() -> Self {
        CellOpt {
            cell: Cell::new(None),
        }
    }
}

impl<T: Clone + Debug> Clone for CellOpt<T> {
    fn clone(&self) -> Self {
        self.restoring_map(|inner| CellOpt::new(inner.clone()))
            .unwrap_or_default()
    }
}

impl<T: Clone + Debug> Debug for CellOpt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.restoring_map(|inner| {
            write!(f, "{}", format!("Option::Some({:?})", inner))
        })
        .unwrap_or_else(|| write!(f, "None"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueError {
    Occupied,
    Empty,
}

pub struct InitializeErr<T> {
    to_insert: T,
    err: ValueError,
}

impl<T: Clone + Debug> CellOpt<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            cell: Cell::new(value.into()),
        }
    }

    #[inline]
    pub fn restoring_map<U, F: Fn(T) -> U>(&self, f: F) -> Option<U> {
        self.take()
            .map(|t| {
                let u = f(t);
                self.replace(t);
                u
            })
            .ok()
    }

    #[inline]
    pub fn replacing_map<F: Fn(T) -> T>(&self, f: F) {
        if let Ok(t) = self.take() {
            self.replace(f(t));
        }
    }

    #[inline]
    pub fn force_take(&self) -> T {
        self.take().unwrap()
    }

    #[inline]
    pub fn initialize(&self, value: T) -> Result<(), InitializeErr<T>> {
        if self.is_occupied() {
            Err(InitializeErr {
                to_insert: value,
                err: ValueError::Occupied,
            })
        } else {
            self.replace(value);
            Ok(())
        }
    }

    #[inline]
    pub fn take(&self) -> Result<T, ValueError> {
        self.cell.take().ok_or(ValueError::Empty)
    }

    #[inline]
    pub fn is_occupied(&self) -> bool {
        if let Ok(value) = self.take() {
            self.replace(value);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn replace(&self, value: impl Into<Option<T>>) {
        self.cell.replace(value.into());
    }
}
