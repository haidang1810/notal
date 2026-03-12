use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryLayer {
    Working,
    Episodic,
    Semantic,
}

impl fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Working => write!(f, "working"),
            Self::Episodic => write!(f, "episodic"),
            Self::Semantic => write!(f, "semantic"),
        }
    }
}

impl MemoryLayer {
    pub fn from_str(s: &str) -> Self {
        match s {
            "episodic" => Self::Episodic,
            "semantic" => Self::Semantic,
            _ => Self::Working,
        }
    }
}
