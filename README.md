[![Build Status](https://travis-ci.org/hypp/modtool.svg?branch=master)](https://travis-ci.org/hypp/modtool)

# About
This is a program written in Rust for manipulating Amiga ProTracker MOD-files.
It can also read (some?) MOD-files packed with The Player 6.1 , including 
8-bit and 4-bit delta packed samples.

The program can show various statistics about the file, extract the samples,
remove unused samples and remove unused patterns.
Future improvements will be the ability to replace samples.

Please feel free to report bugs and contribute in anyway you like.

# License
Released under MIT License, please see the file LICENSE.

# Prerequisites
Install using rustup https://www.rust-lang.org/tools/install

# Build
```
git clone https://github.com/hypp/modtool
cd modtool
cargo build --release
```

# Usage
To see usage, run modtool -h

