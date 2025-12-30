use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    SelfConsistent,
    Implemented,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::SelfConsistent => "self-consistent",
            Stage::Implemented => "implemented",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub spec_path: PathBuf,
    pub stage: Stage,
}
