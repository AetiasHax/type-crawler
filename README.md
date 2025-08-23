# type-crawler
A library which scans for type definitions in a C/C++ codebase.

## Contents
- [Usage](#usage)
- [Running tests](#running-tests)

## Usage
```rust
use type_crawler::{Env, EnvOptions, TypeCrawler};

let env = Env::new(EnvOptions::default());
let crawler = TypeCrawler::new(env).unwrap();
let types = crawler.parse_file("path/to/file.hpp").unwrap();

for ty in types.types() {
    println!("{ty}");
}
```

See the [`lib/tests/`](/lib/tests/) directory for more examples.

## Running tests
The `clang` crate only allows one `Clang` instance at a time, so tests must be run on one thread:
```
cargo test -- --test-threads=1
```