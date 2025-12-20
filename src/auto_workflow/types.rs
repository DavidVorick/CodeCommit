use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    SelfConsistent,
    ProjectConsistent,
    Complete,
    Secure,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::SelfConsistent => "self-consistent",
            Stage::ProjectConsistent => "project-consistent",
            Stage::Complete => "complete",
            Stage::Secure => "secure",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Option<Stage> {
        match self {
            Stage::SelfConsistent => Some(Stage::ProjectConsistent),
            Stage::ProjectConsistent => Some(Stage::Complete),
            Stage::Complete => Some(Stage::Secure),
            Stage::Secure => None,
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
