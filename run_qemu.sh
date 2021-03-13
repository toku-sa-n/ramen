#!/bin/bash

flags="-drive if=pflash,format=raw,file=OVMF_CODE.fd,readonly=on -drive if=pflash,format=raw,file=OVMF_VARS.fd,readonly=on -drive format=raw,file=build/ramen_os.img -no-reboot -m 4G -d int -device isa-debug-exit,iobase=0xf4,iosize=0x04 -device qemu-xhci,id=xhci -device usb-kbd --trace events=trace.event -device usb-mouse, -drive id=usb,file=build/bootx64.efi,if=none,format=raw -device usb-storage,drive=usb"

make build/ramen_os.img
qemu-system-x86_64 $flags -no-shutdown -monitor stdio
