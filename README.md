# RusPiRo Custom Testframework

The crate provides a custom test framwork that can be used for unit and integration tests with other `ruspiro-` crates using *QEMU* as runtime.

[![Travis-CI Status](https://api.travis-ci.com/RusPiRo/ruspiro-singleton.svg?branch=master)](https://travis-ci.com/RusPiRo/ruspiro-singleton)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-singleton.svg)](https://crates.io/crates/ruspiro-singleton)
[![Documentation](https://docs.rs/ruspiro-singleton/badge.svg)](https://docs.rs/ruspiro-singleton)
[![License](https://img.shields.io/crates/l/ruspiro-singleton.svg)](https://github.com/RusPiRo/ruspiro-singleton#license)

The content of this crate is inspired by the preparation work and initial investigations of [@André Richter](https://github.com/andre-richter) and [@Phil Opp](https://github.com/phil-opp).

## Usage

### Prerequisits

To use this crate several steps are required.

First add the dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-test-framework = "||VERSION||"
```

In addition the `main.rs` or `lib.rs` file of the crate that will use this test framework requires the following code to be placed close to the very beginning of the file:

```rust
#![cfg_attr(test, no_main)]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(ruspiro_test_framework::test_runner)]

#[cfg(test)]
extern crate ruspiro_test_framework;

#[cfg(test)]
#[no_mangle]
extern "C" fn run_test_main() {
    test_main();
}
```

Next a file named `config.toml` need to be created in the folder `.cargo` of the current crates root folder if it does not exist already. The following content need to be added:

```toml
[target.'cfg(target_os = "none")']
runner = "cargo make qemu-test"
```

As it is assumed that the whole building of `ruspiro-` crates is done with `cargo make` and this tool is already installed the actual `Makefile.toml` need to be enhanced with the following task:

```rust
[tasks.unittest]
env = { FEATURES = "ruspiro_pi3" }
command = "cargo"
args = ["xtest", "--target", "${BUILD_TARGET}", "--tests", "--features", "${FEATURES}"]

[tasks.qemu-test-objcopy]
command = "aarch64-none-elf-objcopy"
args = ["-O",  "binary",  "${CARGO_MAKE_TASK_ARGS}", "./target/kernel-test.img"]

[tasks.qemu-test]
script = [
    "qemu-system-aarch64 -semihosting -display none -M raspi3 -kernel ./target/kernel-test.img -serial null -serial stdio -d int,mmu -D qemu-test.log"
]
dependencies = [
    "qemu-test-objcopy"
]
```

If those prerequisits are provided writing test cases is straight forward and kind of similar to the way it would be done in the *standard* way:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ruspiro_test_framework::*;

    #[ruspiro_test]
    fn simple_unittest() {
        assert_eq!(1, 1);
    }
}
```

## Local Builds

To execute local builds `cargo-make` and `cargo-xbuild` is required. Also ensure the `aarch64-unknown-none` rust build target is installed as well as it's `src` components. It's safe to use the following commands as a one time setup, assuming the `aarch64-none-elf` crosscompilation toolchain is installed already.

```shell
$> cargo install cargo-xbuild
$> cargo install cargo-make
$> rustup target add aarch64-unknown-none
$> rustup component add rust-src
$> rustup component add llvm-tools-preview
```

## Prepare Pipeline Builds

To enable the full pipeline lifecycle for new crates the integration into `travis-ci` is required. The following prerequisits has to be configured:

1. Create a `release` branch in the github repository. Ensure that this branch is protected and requires PR review as well as travis-ci status checks (flag may only be available after the first travis-ci pipeline run).
2. Add the github repository of this crate to the travis-ci settings for the `RusPiRo` organisation.
3. create an OAuth github token and maintain it in the environment variable settings for this repo in travis-ci with the variable name `GIT_API_TOKEN`.
4. create a crates.io login token and mainain it in the environment variable settings for this repo in travis-ci with the variable name `CRATES_TOKEN`.

## Pipeline runs

The pipeline will run the following steps:

1. Each PR to `master` will trigger a first pipeline build. This build will do the following:
    - run the compilitaion of the library crate with `cargo make pi3`. This build will take the `Cargo.toml` and maintained dependencies *as-is*. This means the `[patch.crates-io]` section can be used to refer to dendent *ruspiro-* crates from their github repositories if multiple dependent changes are *in-flight*.
    - run the doc tests and unit tests of the crate with `cargo make doctest` and `cargo test --tests`
    - run a crates.io publish dry-run
2. If the PR is merged into the `master` branch the same steps from above are executed
    - run the compilitaion of the library crate with `cargo make pi3`. However, this time the `[patch.crates-io]` section of the `Cargo.toml` file will be removed and the used dependencies requires to be available on [crates.io](https://crates.io)
    - run the doc tests and unit tests of the crate with `cargo make doctest` and `cargo test --tests`
    - run a crates.io publish dry-run
    - create a new PR from `master` branch to `release` branch
3. The PR from `master` to `release` branch will be build exactly the same way the first PR to `master` was built.
4. If the PR is merged into the `release` branch it will do the following
    - run the compilitaion of the library crate with `cargo make pi3`. However, this time the `[patch.crates-io]` section of the `Cargo.toml` file will be removed and the used dependencies requires to be available on [crates.io](https://crates.io)
    - run the doc tests and unit tests of the crate with `cargo make doctest` and `cargo test --tests`
    - run a crates.io publish dry-run
    - create a github release with the version number provided in `Cargo.toml` file. This step will also update the `||VERSION||` placeholder in the respective files to ensure consistency
    - publish the actual crate to crates.io

After the first PR has been build using travis-ci you should revisit the branch protection rules for `master` and `release` branch to ensure the pipeline build step is marked as required before pushing to those branches is allowed.

## License

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.
