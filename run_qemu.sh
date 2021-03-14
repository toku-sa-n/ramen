#!/bin/bash

readonly general_flags=" \
    -drive if=pflash,format=raw,file=OVMF_CODE.fd,readonly=on \
    -drive if=pflash,format=raw,file=OVMF_VARS.fd,readonly=on \
    -drive format=raw,file=build/ramen_os.img \
    -drive id=usb,file=build/bootx64.efi,if=none,format=raw \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -device qemu-xhci,id=xhci \
    -device usb-kbd \
    -device usb-mouse \
    -device usb-storage,drive=usb \
    -no-reboot \
    -m 4G \
    -d int \
    --trace events=trace.event \
    "

readonly run_flags=" \
    $general_flags \
    -no-shutdown \
    -monitor stdio \
    "

test_flags=" \
    $general_flags \
    -nographic \
    "

if [[ $1 == "-t" ]]
then
    make test -j
    qemu-system-x86_64 ${test_flags}
    status=$?
    readonly ok_status=33
    if [[ $status -eq $ok_status ]]
    then
        echo "Test succeeded."
    else
        echo "Test failed."
        exit 1
    fi
else
    make build/ramen_os.img -j
    qemu-system-x86_64 ${run_flags}
fi
