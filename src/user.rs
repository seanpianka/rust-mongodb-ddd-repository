use crate::primitives::{AggregateReadRepository, RepositoryReadError, RootAggregate, ID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub(crate) id: ID,
    pub(crate) name: String,
    pub(crate) email: String,
}

impl User {
    pub fn new_random<S, T>(name: S, email: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        Self::new(ID::new_v4(), name, email)
    }

    pub fn new<S, T>(id: ID, name: S, email: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        Self {
            id,
            name: name.as_ref().to_string(),
            email: email.as_ref().to_string(),
        }
    }
}

impl RootAggregate for User {
    fn id(&self) -> &ID {
        &self.id
    }
}

/// Extension of a basic repository for custom user look-ups
pub trait ReadRepository: AggregateReadRepository<User> {
    /// Find one user by an e-mail address, and error if 0 or more than 1 user
    /// are found.
    fn find_by_email(&self, email: String) -> Result<User, RepositoryReadError>;
}
