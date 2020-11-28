pub mod mongo;
pub mod primitives;
mod user;

#[cfg(test)]
mod tests {
    use crate::{
        mongo,
        primitives::{AggregateReadRepository, AggregateWriteRepository},
        user::{ReadRepository, User},
    };
    use lazy_static;

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

    #[test]
    fn should_find_no_users_by_default() {
        let users_repo = FIXTURE.new_repository(COLLECTION_NAME.to_string());
        let users: Vec<User> = users_repo.find_all();
        assert_eq!(0, users.len());
    }

    #[test]
    fn should_store_and_retrieve_user() {
        const EMAIL: &str = "a@example.com";
        const NAME: &str = "Shaun";

        let mut users_repo = FIXTURE.new_repository(COLLECTION_NAME.to_string());
        if let Err(e) = users_repo.store(User::new_random(NAME, EMAIL.to_string())) {
            panic!("failed to store user: {}", e);
        }
        let users: Vec<User> = users_repo.find_all();
        assert_eq!(
            EMAIL,
            users
                .iter()
                .find(|u| u.name.as_str() == NAME)
                .unwrap()
                .email
                .as_str()
        );
    }

    #[test]
    fn should_find_user_using_the_user_repository_trait_extension() {
        const EMAIL: &str = "b@example.com";
        const NAME: &str = "Sean";

        let mut users_repo = FIXTURE.new_repository(COLLECTION_NAME.to_string());
        if let Err(e) = users_repo.store(User::new_random(NAME, EMAIL.to_string())) {
            panic!("failed to store user: {}", e);
        }
        if let Err(e) = users_repo.store(User::new_random(
            "Other User",
            "otheremail@example.com".to_string(),
        )) {
            panic!("failed to store user: {}", e);
        }
        let user: User = match users_repo.find_by_email(EMAIL.to_string()) {
            Ok(u) => u,
            Err(e) => {
                panic!("failed to find_by_email: {}", e);
            }
        };
        assert_eq!(EMAIL, user.email.as_str());
    }
}
