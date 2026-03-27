use std::fmt;

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Owner {
    Self_,
    User,
    World,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    Observed,
    Said,
    Acted,
    Inferred,
    Draft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventKind {
    Observation,
    Conversation,
    Action,
    Reflection,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Namespace(String);

impl Namespace {
    pub fn self_() -> Self {
        Self("self".to_string())
    }

    pub fn world() -> Self {
        Self("world".to_string())
    }

    pub fn for_user(user_id: impl AsRef<str>) -> Self {
        Self(format!("user/{}", user_id.as_ref()))
    }

    pub fn for_project(project_id: impl AsRef<str>) -> Self {
        Self(format!("project/{}", project_id.as_ref()))
    }

    pub fn for_owner(owner: Owner) -> Self {
        match owner {
            Owner::Self_ => Self::self_(),
            Owner::User => Self::for_user("default"),
            Owner::World | Owner::Unknown => Self::world(),
        }
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let is_valid = value == "self"
            || value == "world"
            || value
                .strip_prefix("user/")
                .is_some_and(|suffix| !suffix.is_empty())
            || value
                .strip_prefix("project/")
                .is_some_and(|suffix| !suffix.is_empty());

        if is_valid {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidNamespace)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches_owner(&self, owner: Owner) -> bool {
        match owner {
            Owner::Self_ => self.as_str() == "self",
            Owner::User => self.as_str().starts_with("user/"),
            Owner::World | Owner::Unknown => {
                self.as_str() == "world" || self.as_str().starts_with("project/")
            }
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
