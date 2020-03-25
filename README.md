# ramen

[![Build Status](https://travis-ci.com/toku-sa-n/ramen.svg?branch=master)](https://travis-ci.com/toku-sa-n/ramen)

A toy OS

## Requirements
- A computer supporting UEFI
- nasm
- Rustup nightly version
- Cargo
- xbuild

## Installation
First of all, you have to install the dependencies:
```sh
sudo apt-get install nasm
sudo cargo install cargo-xbuild
```

Next, you have to create an EFI partition.

Then run the following command:
```sh
git clone https://github.com/toku-sa-n/ramen.git
cd ramen
rustup component add rust-src
make release
USB_DEVICE_PATH="/dev/sdx1" make copy_to_usb
```
(/dev/sdx1 is the EFI partition you created.)

## Execution
Reboot your machine and run Ramen OS.

---
<div style="text-align:center;"><img src="images/ramen.jpg"></div>

(This image is not related to the project.)
