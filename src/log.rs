pub use crate::{debug, error, info, trace, warn};

/// Log Context
#[derive(Clone)]
pub struct LC {
    /// The name to log under. This could be a Rc or Arc, but as of now just copying everything
    /// over.
    pub name: String,
    /// Whether or not the log statements should actually log for info and below.
    /// Error and Warn will still be logged.
    pub should_log: bool,
}

impl LC {
    pub fn new(name: &str, should_log: bool) -> Self {
        Self {
            name: name.into(),
            should_log,
        }
    }

    /// Create a log for a child element, such as a TextBox in another element.
    pub fn child(&self, name_extention: &str) -> Self {
        Self {
            name: format!("{} > {}", self.name, name_extention),
            should_log: self.should_log,
        }
    }

    /// Log for both elements to log under both of them.
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            name: format!("{} & {}", self, other),
            should_log: self.should_log || other.should_log,
        }
    }

    /// return a new log
    pub fn with_log(self, should_log: bool) -> Self {
        Self { should_log, ..self }
    }
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for LC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.name)
    }
}

#[macro_export]
macro_rules! error {
    ($ctx:expr, $fmt:literal $(,$args:expr)*) => {
        ::log::error!("{} {}", $ctx, format!($fmt, $($args),*))
    }
}

#[macro_export]
macro_rules! warn {
    ($ctx:expr, $fmt:literal $(,$args:expr)*) => {
        ::log::warn!("{} {}", $ctx, format!($fmt, $($args),*))
    }
}

#[macro_export]
macro_rules! info {
    ($ctx:expr, $fmt:literal $(,$args:expr)*) => {
        if $ctx.should_log {
            ::log::info!("{} {}", $ctx, format!($fmt, $($args),*))
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($ctx:expr, $fmt:literal $(,$args:expr)*) => {
        if $ctx.should_log {
            ::log::debug!("{} {}", $ctx, format!($fmt, $($args),*))
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($ctx:expr, $fmt:literal $(,$args:expr)*) => {
        if $ctx.should_log {
            ::log::trace!("{} {}", $ctx, format!($fmt, $($args),*))
        }
    }
}
