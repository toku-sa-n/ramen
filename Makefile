SHELL			:= /bin/bash

BUILD_DIR		:= build
EFI_DIR			:= bootx64
KERNEL_DIR		:= kernel

LD_SRC			:= $(KERNEL_DIR)/kernel.ld

EFI_FILE		:= $(BUILD_DIR)/bootx64.efi
KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin
LIB_FILE		:= $(BUILD_DIR)/libramen_os.a
IMG_FILE		:= $(BUILD_DIR)/ramen_os.img

LD				:= ld
RUSTC			:= cargo
RM				:= rm -rf

OVMF_CODE		:= OVMF_CODE.fd
OVMF_VARS		:= OVMF_VARS.fd

RUSTCFLAGS		:= --release
LDFLAGS			:= -nostdlib -T $(LD_SRC)

.PHONY:all copy_to_usb test clean $(LIB_FILE) $(EFI_FILE)
.SUFFIXES:

all:$(KERNEL_FILE) $(EFI_FILE)

copy_to_usb:$(KERNEL_FILE) $(EFI_FILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo mkdir -p /mnt/efi/boot
	sudo cp $(EFI_FILE) /mnt/efi/boot/
	sudo cp $(KERNEL_FILE) /mnt/
	sudo umount /mnt
endif

test:
	make clean
	make $(IMG_FILE) TEST_FLAG=--features=qemu_test

$(IMG_FILE):$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)
	dd if=/dev/zero of=$@ bs=1k count=28800
	mformat -i $@ -h 200 -t 500 -s 144::
	# Cannot replace these mmd and mcopy with `make copy_to_usb` because `mount` needs `sudo`
	# regardless of the permission of the image file or the device. Using `mmd` and `mcopy` is
	# the only way to edit image file without `sudo`.
	mmd -i $@ ::/efi
	mmd -i $@ ::/efi/boot
	mcopy -i $@ $(KERNEL_FILE) ::
	mcopy -i $@ $(EFI_FILE) ::/efi/boot

$(KERNEL_FILE):$(LIB_FILE) $(LD_SRC)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $(LIB_FILE)

$(LIB_FILE):|$(BUILD_DIR)
	# FIXME: Currently `cargo` tries to read `$(pwd)/.cargo/config.toml`, not
	# `$(dirname argument_of_--manifest-path)/.cargo/config.toml`.
	# See: https://github.com/rust-lang/cargo/issues/2930
	cd $(KERNEL_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(TEST_FLAG) $(RUSTCFLAGS)

%.fd:
	@echo "$@ not found"
	exit 1

$(EFI_FILE):|$(BUILD_DIR)
	cd $(EFI_DIR) && $(RUSTC) build --out-dir=../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(BUILD_DIR):
	mkdir $@ -p

clean:
	$(RM) build
