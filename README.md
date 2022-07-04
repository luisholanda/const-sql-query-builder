This repository contains an experiment on a full `const fn` SQL query builder. The objective is to
be able to build full SQL queries during compile time, and have only the execution of the query itself
be at runtime.

It is not focus of this experiment trying to map the entire SQL semantics into Rust, but it will handle
most of it. In particular, don't expect to have SQL <-> Rust types checks.

Note that, at time of writing, we've +20 unstable features enabled, meaning: **for the love of anyone
depending on your software, and for yourself, DO NOT USE THIS IN PRODUCTION**.

A few keypoints:

* As currently there isn't a way of handling `String`s on `const fn`s, this crate builds a full `ConstString`
  using the `const_box` feature and a custom `ConstAlloc: ~const Allocator`.
* We can't write some methods as they would require `Drop` support in `const` contexts, which isn't
  implemented yet.
