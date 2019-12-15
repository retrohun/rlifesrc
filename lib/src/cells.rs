//! Cells in the cellular automaton.

use crate::rule::Desc;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    ops::{Deref, Not},
};
pub use State::{Alive, Dead};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Possible states of a known cell.
///
/// During the search, the state of a cell is represented by `Option<State>`,
/// where `None` means that the state of the cell is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum State {
    Alive = 0b01,
    Dead = 0b10,
}

/// Flips the state.
impl Not for State {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Alive => Dead,
            Dead => Alive,
        }
    }
}

/// Randomly chooses between `Alive` and `Dead`.
///
/// The probability of either state is 1/2.
impl Distribution<State> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State {
        match rng.gen_range(0, 2) {
            0 => Dead,
            _ => Alive,
        }
    }
}

/// The coordinates of a cell.
///
/// `(x-coordinate, y-coordinate, time)`.
/// All three coordinates are 0-indexed.
pub type Coord = (isize, isize, isize);

/// A cell in the cellular automaton.
///
/// The name `LifeCell` is chosen to avoid ambiguity with
/// [`std::cell::Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html).
pub struct LifeCell<'a> {
    /// The index of the cell in the world.
    pub(crate) id: usize,

    /// The coordinates of a cell.
    pub coord: Coord,

    /// The background state of the cell.
    ///
    /// For rules without `B0`, it is always dead.
    /// For rules with `B0`, it is dead on even generations,
    /// alive on odd generations.
    pub(crate) background: State,

    /// The state of the cell.
    ///
    /// `None` means that the state of the cell is unknown.
    pub(crate) state: Cell<Option<State>>,

    /// The “neighborhood descriptors” of the cell.
    ///
    /// It describes the states of the cell itself, its neighbors,
    /// and its successor.
    pub(crate) desc: Cell<Desc>,

    /// The predecessor of the cell.
    ///
    /// The cell in the last generation at the same position.
    pub(crate) pred: Option<CellRef<'a>>,
    /// The successor of the cell.
    ///
    /// The cell in the next generation at the same position.
    pub(crate) succ: Option<CellRef<'a>>,
    /// The eight cells in the neighborhood.
    pub(crate) nbhd: [Option<CellRef<'a>>; 8],
    /// The cells in the same generation that must has the same state
    /// with this cell because of the symmetry.
    pub(crate) sym: Vec<CellRef<'a>>,

    /// Whether the cell is on the first row or column.
    ///
    /// Here the choice of row or column depends on the search order.
    pub(crate) is_front: bool,

    /// The decision level for assigning the cell state.
    pub(crate) level: Cell<Option<usize>>,

    /// Whether the cell has been seen in the analysis.
    pub(crate) seen: Cell<bool>,
}

impl<'a> LifeCell<'a> {
    /// Generates a new cell with state `state`, such that its neighborhood
    /// descriptor says that all neighboring cells also have the same state.
    ///
    /// `first_gen` and `first_col` are set to `false`.
    pub(crate) fn new(id: usize, coord: Coord, background: State, b0: bool) -> Self {
        let succ_state = if b0 { !background } else { background };
        LifeCell {
            id,
            coord,
            background,
            state: Cell::new(Some(background)),
            desc: Cell::new(Desc::new(background, succ_state)),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            is_front: false,
            level: Cell::new(None),
            seen: Cell::new(false),
        }
    }

    /// Returns a `CellRef` from a `LifeCell`.
    pub(crate) fn borrow(&self) -> CellRef<'a> {
        let cell = unsafe { (self as *const LifeCell<'a>).as_ref().unwrap() };
        CellRef { cell }
    }
}

impl<'a> Debug for LifeCell<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "LifeCell {{ coord: {:?}, state: {:?}, desc: {:?}, level: {:?} }}",
            self.coord,
            self.state.get(),
            self.desc.get(),
            self.level.get()
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CellRef<'a> {
    cell: &'a LifeCell<'a>,
}

impl<'a> PartialEq for CellRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.cell, other.cell)
    }
}

impl<'a> Eq for CellRef<'a> {}

impl<'a> Deref for CellRef<'a> {
    type Target = LifeCell<'a>;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<'a> Debug for CellRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "CellRef {{ coord: {:?}, state: {:?}, desc: {:?}, level: {:?} }}",
            self.coord,
            self.state.get(),
            self.desc.get(),
            self.level.get()
        )
    }
}
