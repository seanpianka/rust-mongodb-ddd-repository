use mongodb::{
    bson::{doc, Bson},
    options::{UpdateModifications, UpdateOptions},
};

use crate::primitives::ID;
use crate::primitives::{
    AggregateReadRepository, AggregateWriteRepository, RepositoryReadError, RepositoryWriteError,
    RootAggregate,
};

pub struct Repository {
    pub db: String,
    pub collection: String,
    pub client: mongodb::sync::Client,
}

impl<RA: RootAggregate> AggregateWriteRepository<RA> for Repository {
    fn clear(&mut self) -> Result<(), RepositoryWriteError> {
        let coll = self
            .client
            .database(self.db.as_str())
            .collection(self.collection.as_str());
        if let Err(e) = coll.drop(None) {
            return Err(RepositoryWriteError::FailedToPersist {
                cause: e.to_string(),
            });
        }
        Ok(())
    }

    fn store(&mut self, aggregate: RA) -> Result<(), RepositoryWriteError> {
        let aggregate_bson = match mongodb::bson::to_bson(&aggregate) {
            Ok(agg) => agg,
            Err(e) => {
                return Err(RepositoryWriteError::FailedToPersist {
                    cause: format!("failed to encode aggregate as bson: {}", e),
                });
            }
        };
        match aggregate_bson {
            mongodb::bson::Bson::Document(document) => {
                let id = aggregate.id().to_string();
                let coll = self
                    .client
                    .database(self.db.as_str())
                    .collection(self.collection.as_str());
                let update_modifications = UpdateModifications::Document(document);
                let mut update_options = UpdateOptions::default();
                update_options.upsert = Some(true);
                match coll.update_one(doc! {"id": id}, update_modifications, update_options) {
                    Ok(_res) => Ok(()),
                    Err(e) => Err(RepositoryWriteError::FailedToPersist {
                        cause: format!("failed to update collection: {:?}", e),
                    }),
                }
            }
            unknown => Err(RepositoryWriteError::FailedToPersist {
                cause: format!("bson of encoded aggregate was: {}", unknown),
            }),
        }
    }
}

impl<RA: RootAggregate> AggregateReadRepository<RA> for Repository {
    fn find(&self, id: &ID) -> Result<RA, RepositoryReadError> {
        let id = id.to_string();
        let unknown_entity_err = RepositoryReadError::UnknownEntity(id.clone());
        let coll = self
            .client
            .database(self.db.as_str())
            .collection(self.collection.as_str());
        let filter = doc! {"id": id};
        match coll.find_one(filter, None) {
            Ok(d) => match d {
                Some(document) => match mongodb::bson::from_bson::<RA>(Bson::from(document)) {
                    Ok(agg) => Ok(agg),
                    Err(e) => {
                        println!("*** {:?}", e);
                        Err(unknown_entity_err)
                    }
                },
                None => Err(unknown_entity_err),
            },
            Err(e) => {
                println!("*** {:?}", e);
                Err(unknown_entity_err)
            }
        }
    }

    fn find_all(&self) -> Vec<RA> {
        let coll = self
            .client
            .database(self.db.as_str())
            .collection(self.collection.as_str());
        let filter = doc! {};
        let mut results = vec![];
        match coll.find(filter, None) {
            Ok(cursor) => {
                for result in cursor {
                    match result {
                        Ok(document) => {
                            match mongodb::bson::from_bson::<RA>(Bson::from(document)) {
                                Ok(agg) => {
                                    results.push(agg);
                                }
                                Err(e) => {
                                    println!("*** {:?}", e);
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            println!("*** {:?}", e);
                            continue;
                        }
                    }
                }
                results
            }
            Err(e) => {
                println!("*** {:?}", e);
                results
            }
        }
    }
}
