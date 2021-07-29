[![Build Status](https://travis-ci.org/hypp/modtool.svg?branch=master)](https://travis-ci.org/hypp/modtool)

# About
Some small programs written in Rust for manipulating Amiga ProTracker MOD-files.
They can also read (some?) MOD-files packed with The Player 6.1, including 
8-bit and 4-bit delta packed samples, and also create The Player compatible files.

The programs can show various statistics about the file, extract the samples,
remove unused samples and remove unused patterns.

Please feel free to report bugs and contribute in anyway you like.

# Usage
To see usage, run:
```
 modtool -h
 mod2json -h
 json2mod -h
 p612mod -h
 mod2p61 -h
```

# License
Released under MIT License, please see the file LICENSE.

# Prerequisites
Install Rust using rustup https://www.rust-lang.org/tools/install

# Build
```
git clone https://github.com/hypp/modtool
cd modtool
cargo build --release
```

# Build on Windows
Quick instructions for building on Windows.  

1. Install Visual C++ Build Tools  
https://visualstudio.microsoft.com/visual-cpp-build-tools/  
Download and run the installer  
Select Workloads => C++ Desktop, Language Pack => English, Individual Components => Windows 10 SDK  

2. Install git for windows  
https://gitforwindows.org/  

3. Install rustup  
https://rustup.rs/  
Download and install rustup-init.exe  

4. Launch a command window, cmd.exe  

5. Create a work folder  
```
mkdir rust
cd rust
````

6. Get the source code  
```
git clone https://github.com/hypp/modtool
````

7.  Enter the project folder and build  
```
cd modtool
cargo build --release
```

8. Do a testrun  
```
target\release\modtool.exe -h
```
