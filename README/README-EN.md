# Fourier Analogue-in-Digital

## Project Overview

Rust implementation of [AAPM](https://mikhael-openworkspace.notion.site/Project-Archivist-e512fa7a21474ef6bdbd615a424293cf)@Audio-8151. More information can be found in the [Notion](https://mikhael-openworkspace.notion.site/Fourier-Analogue-in-Digital-d170c1760cbf4bb4aaea9b1f09b7fead?pvs=4).

## Input/Output PCM format

Floating-point

- f16be, f32be, f64be(Default)
- f16le, f32le, f64le

Signed Integer

- s8
- s16be, s24be, s32be, s64be
- s16le, s24le, s32le, s64le

Unsigned Integer

- u8
- u16be, u24be, u32be, u64be
- u16le, u24le, u32le, u64le

## How to install

1. download the library with Git clone
2. build with cargo build --release
3. move the executable file in target/release to your desired location
4. register the variable in PATH

```bash
git clone https://github.com/H4n-uL/FrAD_Rust.git
cd FrAD_Rust
cargo build --release
mv target/release/frad-rs /path/to/bin/frad-rs
export PATH=/path/to/bin:$PATH
```

**Warning: Building without `--release` will result in extremely slow execution, so be sure to build with `--release`.**

## External Resources

[Rust](https://github.com/rust-lang/rust)

### Cargo crates

#### Library dependencies

1. half
2. miniz_oxide
3. palmfft

#### Application dependencies

1. base64
2. infer
3. rodio
4. same-file
5. serde_json
6. tempfile

## How to contribute

### Contributing to FrAD format

Contributions to the FrAD format itself should be made [here](https://github.com/H4n-uL/Fourier_Analogue-in-Digital) or by emailing me directly.

### Contributions to Rust Master

Contributions that are specific to the Rust implementation can be made directly to this repository. Anything from feature additions, bug fixes, or performance improvements are welcome.

Here's how to contribute

1. fork the repository
2. create a new branch
3. make a fix and suffer through the bugs
4. push to the main branch
5. create a pull request to this repository

Once the pull request is created, we'll review it and give you feedback or merge it - in fact, we almost always take it if it's compatible with FrAD standard.

## Developer information

HaמuL, <jun061119@proton.me>
