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

Last but not least the crate requires a build script which ensures the linker script that is required to properly build and link the test runner binary that will be executed within QEMU. The file `build.rs` will be created in the crates root folder with at least the following contents:

```rust
use std::{env, fs, path::Path};

fn main() {
    // copy the linker script from the boot crate to the current directory
    // so it will be invoked by the linker
    match env::var_os("DEP_RUSPIRO_TEST_FRAMEWORK_LINKERSCRIPT") {
        Some(source) => {
            println!("found test framework dependency");
            let ld_source = source.to_str().unwrap().replace("\\", "/");
            let src_file = Path::new(&ld_source);
            let trg_file = format!(
                "{}/{}",
                env::current_dir().unwrap().display(),
                src_file.file_name().unwrap().to_str().unwrap()
            );
            println!("Copy linker script from {:?}, to {:?}", src_file, trg_file);
            fs::copy(src_file, trg_file).unwrap();
        },
        _ => ()
    }
}
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

## License

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.
