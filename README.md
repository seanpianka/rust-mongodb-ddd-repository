# rust-mongodb-ddd-repository

An example of using MongoDB as the backing store for a DDD-style Root Aggregate. This approach uses MongoDB as a ["aggregate-oriented database"](https://www.martinfowler.com/bliki/AggregateOrientedDatabase.html).

`crate::primitives` provides type definitions for central DDD concepts such as:
* A `RootAggregate`, a cluster of objects treated as a unit for the purpose of data changes and behavior.
* An `AggregateWriteRepository` and `AggregateReadRepository`, which is an abstraction for a repository that works with root aggregates. 

`crate::mongo` provides the generic implementation for persisting and retrieving any root aggregate into MongoDB. The interface is simple (and does not provide lazy-loading) since good aggregate design should lend to smaller, more focused aggregates.

## Tests

Setup a Docker container running mongodb:

```
$ docker run --rm -p 27017:27017 mongo:latest
```

Then, run the tests located in `src/lib.rs`:

```
$ cargo test
```