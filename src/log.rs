pub use ::log::{debug, error, info, trace, warn};

/// Log Context
#[derive(Clone)]
pub struct LC {
    pub name: Box<str>,
    pub should_log: bool,
}

impl LC {
    pub fn new(name: &str, should_log: bool) -> Self {
        Self {
            name: name.into(),
            should_log,
        }
    }
    pub fn child(&self, name_extention: &str) -> Self {
        Self {
            name: format!("{} {}", self.name, name_extention).into(),
            should_log: self.should_log,
        }
    }
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for LC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "'{}'", self.name)
    }
}
