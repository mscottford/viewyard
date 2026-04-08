use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitProtocol {
    #[default]
    Ssh,
    Https,
}

impl fmt::Display for GitProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ssh => write!(f, "ssh"),
            Self::Https => write!(f, "https"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViewsetConfig {
    pub protocol: GitProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub is_private: bool,
    pub source: String, // e.g., "GitHub (username)" or "GitHub (org/username)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>, // Optional explicit account field for git user configuration
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
