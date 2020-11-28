use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub type ID = uuid::Uuid;

/// An AGGREGATE is a cluster of associated objects that we treat as a unit for
/// the purpose of data changes. Each AGGREGATE has a root and a boundary. The
/// boundary defines what is inside the AGGREGATE. The root is a single,
/// specific ENTITY contained in the AGGREGATE.
///
/// The root is the only member of the AGGREGATE that outside objects are
/// allowed to hold references to.
pub trait RootAggregate: Clone + Serialize + DeserializeOwned + Debug {
    /// id should be the entities globally unique id. It doesn't matter what it
    /// is internally as long as that thing can be returned as a string
    /// (implements Display from std).
    fn id(&self) -> &ID;
}

pub trait AggregateWriteRepository<T: RootAggregate> {
    fn store(&mut self, aggregate: T) -> Result<(), RepositoryWriteError>;
    fn clear(&mut self) -> Result<(), RepositoryWriteError>;
}

pub trait AggregateReadRepository<T: RootAggregate> {
    fn find(&self, id: &ID) -> Result<T, RepositoryReadError>;
    fn find_all(&self) -> Vec<T>;
}

#[derive(Error, Debug, Clone)]
pub enum RepositoryReadError {
    #[error("no events were found")]
    NoEventsFound,
    #[error("entity `{0}` was not found")]
    UnknownEntity(String),
    #[error("multiple entities found for `{0}`")]
    MultipleEntitiesFound(String),
}

#[derive(Error, Debug, Clone)]
pub enum RepositoryWriteError {
    #[error("failed to persist entity: `{cause}`")]
    FailedToPersist { cause: String },
}
