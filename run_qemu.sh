#!/bin/bash

flags="
-drive if=pflash,format=raw,file=OVMF_CODE.fd,readonly=on
-drive if=pflash,format=raw,file=OVMF_VARS.fd,readonly=on
-drive format=raw,file=build/ramen_os.img
-drive id=usb,file=build/bootx64.efi,if=none,format=raw
-device isa-debug-exit,iobase=0xf4,iosize=0x04
-device qemu-xhci,id=xhci
-device usb-kbd
-device usb-mouse
-device usb-storage,drive=usb
-no-reboot
-m 4G
-d int
--trace events=trace.event
"

make build/ramen_os.img -j
qemu-system-x86_64 "$flags" -no-shutdown -monitor stdio
