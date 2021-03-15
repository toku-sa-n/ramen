SHELL			:= /bin/bash

BUILD_DIR		:= build
KERNEL_DIR		:= kernel
ISOFILES_DIR	:= $(BUILD_DIR)/isofiles

LD_SRC			:= $(KERNEL_DIR)/kernel.ld
GRUB_CFG		:= grub.cfg

KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin
LIB_FILE		:= $(BUILD_DIR)/libramen_os.a
ISO_FILE		:= $(BUILD_DIR)/ramen_os.iso

LD				:= ld
RUSTC			:= cargo
RM				:= rm -rf

OVMF_CODE		:= OVMF_CODE.fd
OVMF_VARS		:= OVMF_VARS.fd

RUSTCFLAGS		:= --release
LDFLAGS			:= -nostdlib -T $(LD_SRC)

.PHONY:all copy_to_usb test clean $(LIB_FILE)
.SUFFIXES:

all:$(ISO_FILE)

copy_to_usb:$(KERNEL_FILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo cp $(KERNEL_FILE) /mnt/
	sudo umount /mnt
endif

test:
	make clean
	make $(ISO_FILE) TEST_FLAG=--features=qemu_test

$(ISO_FILE):$(KERNEL_FILE) $(GRUB_CFG) |$(ISOFILES_DIR)
	mkdir -p $(ISOFILES_DIR)/boot/grub
	cp $(GRUB_CFG) $(ISOFILES_DIR)/boot/grub
	cp $(KERNEL_FILE) $(ISOFILES_DIR)/boot
	grub-mkrescue -o $(ISO_FILE) $(ISOFILES_DIR)

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

$(BUILD_DIR):
	mkdir $@ -p

$(ISOFILES_DIR):
	mkdir $@ -p

clean:
	$(RM) build
