#!/bin/bash

set -e

create_storage_image() {
    mkdir -p build
    dd if=/dev/zero of=build/storage.img count=10000
}

readonly common_parameters=" \
    -drive if=pflash,format=raw,file=OVMF_CODE.fd,readonly=on \
    -drive if=pflash,format=raw,file=OVMF_VARS.fd,readonly=on \
    -drive format=raw,file=build/ramen_os.iso \
    -drive id=usb,file=build/storage.img,if=none,format=raw \
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

readonly parameters_for_running_qemu=" \
    $common_parameters \
    -no-shutdown \
    -monitor stdio \
    "

readonly parameters_for_testing=" \
    $common_parameters \
    -nographic \
    "

if [[ $1 == "-t" ]]
then
    make test -j
    create_storage_image

    # QEMU exist with the exit code nonzero value even on success.
    set +e
    qemu-system-x86_64 ${parameters_for_testing}
    readonly status=$?
    readonly ok_status=33
    if [[ $status -eq $ok_status ]]
    then
        echo "Test succeeded."
    else
        echo "Test failed."
        exit 1
    fi
else
    make build/ramen_os.iso -j
    create_storage_image
    qemu-system-x86_64 ${parameters_for_running_qemu}
fi
