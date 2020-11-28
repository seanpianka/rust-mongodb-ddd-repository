pub mod mongo;
pub mod primitives;

#[cfg(test)]
mod tests {
    use crate::{
        mongo,
        primitives::{AggregateReadRepository, AggregateWriteRepository, RootAggregate, ID},
    };
    use lazy_static;
    use serde::{Deserialize, Serialize};

    const MONGODB_URL: &str = "mongodb://0.0.0.0:27017";
    const MONGODB_TEST_DATABASE: &str = "my-cool-app";
    const COLLECTION_NAME: &str = "users";

    // Invoke sync functions from tests
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    struct Fixture {
        client: mongodb::sync::Client,
    }

    impl Fixture {
        pub fn new() -> Self {
            let client_options = match aw!(mongodb::options::ClientOptions::parse(MONGODB_URL)) {
                Ok(co) => co,
                Err(e) => {
                    println!();
                    panic!(
                        "failed to parse mongodb url into client options: {:?}\
                    \n*** Warning: make sure mongoDB is running at {}\
                    \n***          use: $ docker run --rm -p 27017:27017 mongo:latest",
                        e, MONGODB_URL
                    )
                }
            };
            let client = match mongodb::sync::Client::with_options(client_options) {
                Ok(c) => c,
                Err(e) => panic!("failed to create mongodb client: {:?}", e),
            };
            let fixture = Self { client };
            let mut users_repo: Box<dyn AggregateWriteRepository<User>> =
                Box::new(fixture.new_repository(COLLECTION_NAME));
            users_repo.clear().unwrap();
            fixture
        }

        /// Create or attach to a collection and manage via a new Repository object. The database
        /// used is a test mongodb database.
        pub fn new_repository<S>(&self, collection: S) -> mongo::Repository
        where
            S: Into<String>,
        {
            mongo::Repository {
                db: MONGODB_TEST_DATABASE.to_string(),
                collection: collection.into(),
                client: self.client.clone(),
            }
        }
    }

    lazy_static::lazy_static! {
        static ref FIXTURE: Fixture = Fixture::new();
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct User {
        id: ID,
        name: String,
        age: i16,
    }

    impl RootAggregate for User {
        fn id(&self) -> &ID {
            &self.id
        }
    }

    #[test]
    fn should_find_no_users_by_default() {
        let users_repo = FIXTURE.new_repository(COLLECTION_NAME.to_string());
        let users: Vec<User> = users_repo.find_all();
        assert_eq!(0, users.len());
    }

    #[test]
    fn should_store_and_retrieve_user() {
        const NAME: &str = "Sean";

        let mut users_repo = FIXTURE.new_repository(COLLECTION_NAME.to_string());
        if let Err(e) = users_repo.store(User {
            id: ID::new_v4(),
            name: NAME.to_string(),
            age: 125,
        }) {
            panic!("failed to store user: {}", e);
        }
        let users: Vec<User> = users_repo.find_all();
        assert_eq!(1, users.len());
        assert_eq!(NAME, users[0].name.as_str());
    }
}
