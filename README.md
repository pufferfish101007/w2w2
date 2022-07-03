# w2w2

A WAT->WASM compiler, written in rust. Name inspired by w2c2.

## Usage

I'll publish this to crates.io at sone point hopefully. It doesn't work yet anyway :D

## Contributing

Issues & pull requests are always welcome (no guarantee they'll get fixed/merged).

### Getting started
To get started with editing this project, you'll need git and rust nightly (>=v1.58) installed.
To clone the repository:
```bash
$ git clone https://github.com/pufferfish101007/w2w2.git
```
To run tests & benchmarks:
```bash
$ cd src
$ cargo test tests
$ cargo bench benches
```

### File structure

- `src` - the directory containing all the source code 
- - `main.rs` - the bin file for running w2w2 from the command line
- - `lib.rs` - the library file that exports useful stuff
- - `lexer.rs` - contains the code, tests & benchmarks for converting a string into a `Vec` of `Token`s