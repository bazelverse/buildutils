# Build information

Bazel is configured as primary build system for this project. See the information below for more details.

A few important differences between Cargo and Bazel:

* Bazel builds in parallel and is therefore significantly faster than Cargo especially for large projects.
* Bazel tests uses tagging thus allowing to query and run only selected tests i.e. only runs tests tagged with "foo".
  Cargo does not support this.
* Bazel caches build artifacts reliably and never rebuilds anything without code change. Cargo, however, has a long history of incorrect cache invalidation and often rebuilds the entire worskpace after only a minor change.
* Bazel supports building polyglot targets, Cargo does not.
* Bazel supports remote builds i.e. compiling on a remote cluster. Cargo does not. 

## Aliasing

This project uses target aliases for Cargo and Bazel mainly to make build targets location independent
and therefore easy to refactor. When you move a crate from one folder to another, just update the alias,
and run make build again. This works because all dependencies are declared using aliases.

For Cargo, crate aliases are defined in the workspace [Cargo.toml](Cargo.toml) file.
Please recognize that Cargo neither supports nested alias namespaces nor path dependent aliases
with the implications that, across the entire workspace, all crates must have exactly one unique names and aliases.
For simplicity, the alias is the same as the crate name.

All Bazel aliases are defined in the [BUILD.bazel](BUILD.bazel) file inside the [alias folder](alias).
For simplicity, Bazel aliases are also sticking to the convention that the alias is the same as the crate name to
replicate the Cargo convention. However, Bazel supports nested alias namespaces and path dependent target aliases
and everything in between. Please refer to the official documentation for more information.

## Build commands

Cargo build works as expected. However, Bazel is configured as primary build system for this project.
Because not everyone is familiar with Bazel, I made a makefile to simplify all build related tasks.

```text
    make build          Builds the code base incrementally (fast) for dev.
    make current        Builds the current target incrementally (fast) defined in current.txt.
    make doc            Builds documentation for the project.
    make format         Formats call code according to cargo fmt style.
    make lint           Lints and formats the code of the project.
    make fix            Fixes linting issues as reported by clippy.
    make test           Tests across all crates.
```



## Selective target build

To build only a single target, run `make current` and specify the target in the current.txt file.
The command in the underlying shell script is the Bazel equivalent to Cargo build -p target.

## Dependencies

External crates come from `Cargo.toml` and `Cargo.lock`: `rules_rs` resolves them with no vendored copy, so a dependency is added in
the manifest and nowhere else. A target names one as `@crates//:name`:

```text
    deps = [
        # External crates
        "@crates//:serde",
        "@crates//:tokio",
    ],
```
