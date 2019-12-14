# ramen

[![Build Status](https://travis-ci.com/toku-sa-n/ramen.svg?branch=master)](https://travis-ci.com/toku-sa-n/ramen)

A toy OS

## Requirements
- QEMU
- nasm
- mtools
- Rustup nightly version
- Cargo
- xbuild

## Installation
First of all, you have to install the dependencies:
```sh
sudo apt-get install nasm qemu mtools
sudo cargo install cargo-xbuild
```

Then run the following command:
```sh
git clone https://github.com/toku-sa-n/ramen.git
cd ramen
rustup component add rust-src
make release
```

## Execution
```sh
make run
```

---
<div style="text-align:center;"><img src="images/ramen.jpg"></div>

(This image is not related to the project.)
