#![cfg(feature = "serialize")]
//! Saves the world.

use crate::{
    cells::{Coord, State},
    config::Config,
    rules::{Life, NtLife, Rule},
    search::{Reason, Search, SetCell},
    world::World,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// A representation of `SetCell` that can be easily serialized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SetCellSer {
    /// The coordinates of the set cell.
    coord: Coord,

    /// The state.
    state: State,

    /// The reason for setting a cell.
    reason: Reason,
}

impl<'a, R: Rule> SetCell<'a, R> {
    fn ser(&self) -> SetCellSer {
        SetCellSer {
            coord: self.cell.coord,
            state: self.cell.state.get().unwrap(),
            reason: self.reason,
        }
    }
}

/// A representation of the world that can be easily serialized.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSer {
    /// World configuration.
    ///
    /// I don't know why I put it here.
    config: Config,

    /// Number of conflicts during the search.
    conflicts: u64,

    /// A stack to records the cells whose values are set during the search.
    ///
    /// The cells in this table always have known states.
    ///
    /// It is used in the backtracking.
    set_stack: Vec<SetCellSer>,

    /// The position in the `set_stack` of the next cell to be examined.
    ///
    /// See `proceed` for details.
    check_index: usize,

    /// The position in the `search_list` of the last decided cell.
    search_index: usize,
}

impl WorldSer {
    /// Restores the world from the `WorldSer`, with the given rule.
    fn world_with_rule<'a, R: Rule>(&self, rule: R) -> Result<World<'a, R>, Box<dyn Error>> {
        let mut world = World::new(&self.config, rule);
        for &SetCellSer {
            coord,
            state,
            reason,
        } in self.set_stack.iter()
        {
            let cell = world.find_cell(coord).ok_or(SetCellErr { coord })?;
            if let Some(old_state) = cell.state.get() {
                if old_state != state {
                    return Err(Box::new(SetCellErr { coord }));
                }
            } else {
                world.set_cell(cell, state, reason);
            }
        }
        world.conflicts = self.conflicts;
        world.check_index = self.check_index;
        world.search_index = self.search_index;
        Ok(world)
    }

    /// Restores the world from the `WorldSer`.
    pub fn world(&self) -> Result<Box<dyn Search>, Box<dyn Error>> {
        if let Ok(rule) = Life::parse_rule(&self.config.rule_string) {
            let world = self.world_with_rule(rule)?;
            Ok(Box::new(world))
        } else {
            let rule = NtLife::parse_rule(&self.config.rule_string)?;
            let world = self.world_with_rule(rule)?;
            Ok(Box::new(world))
        }
    }
}

impl<'a, R: Rule> World<'a, R> {
    /// Saves the world as a `WorldSer`.
    pub fn ser(&self) -> WorldSer {
        WorldSer {
            config: self.config.clone(),
            conflicts: self.conflicts,
            set_stack: self.set_stack.iter().map(|s| s.ser()).collect(),
            check_index: self.check_index,
            search_index: self.search_index,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetCellErr {
    coord: Coord,
}

impl Display for SetCellErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to set cell at {:?}.", self.coord)
    }
}

impl Error for SetCellErr {}