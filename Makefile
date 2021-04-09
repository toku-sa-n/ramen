SHELL			:= /bin/bash

BUILD_DIR		:= build

CONFIG_TOML	:=	.cargo/config.toml
CARGO_TOML	:=	Cargo.toml

EFI_DIR			:= bootx64
EFI_SRC	:=	$(shell find $(EFI_DIR)/src)
EFI_SRC	+=	$(EFI_DIR)/$(CONFIG_TOML)
EFI_SRC	+=	$(EFI_DIR)/$(CARGO_TOML)
EFI_FILE		:= $(BUILD_DIR)/bootx64.efi

KERNEL_DIR		:= kernel
KERNEL_LIB_SRC	:=	$(shell find $(KERNEL_DIR)/src)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/$(CONFIG_TOML)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/$(CARGO_TOML)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/build.rs
KERNEL_LIB		:= $(BUILD_DIR)/libramen_os.a
KERNEL_LD			:= $(KERNEL_DIR)/kernel.ld
KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin

PM_DIR			:= pm
PM_LIB_SRC	:=	$(shell find $(PM_DIR)/src)
PM_LIB_SRC	+=	$(PM_DIR)/$(CONFIG_TOML)
PM_LIB_SRC	+=	$(PM_DIR)/$(CARGO_TOML)
PM_LIB			:= $(BUILD_DIR)/libpm.a
PM				:= $(BUILD_DIR)/pm.bin

IMG_FILE		:= $(BUILD_DIR)/ramen_os.img

INITRD			:= $(BUILD_DIR)/initrd.cpio

LD				:= ld
RUSTC			:= cargo
RM				:= rm -rf

RUSTCFLAGS		:= --release
LDFLAGS			:= -nostdlib

.PHONY:all copy_to_usb test clean
.SUFFIXES:

all:$(IMG_FILE)

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

$(IMG_FILE):$(KERNEL_FILE) $(EFI_FILE)
	dd if=/dev/zero of=$@ bs=1k count=28800
	mformat -i $@ -h 200 -t 500 -s 144::
	# Cannot replace these mmd and mcopy with `make copy_to_usb` because `mount` needs `sudo`
	# regardless of the permission of the image file or the device. Using `mmd` and `mcopy` is
	# the only way to edit image file without `sudo`.
	mmd -i $@ ::/efi
	mmd -i $@ ::/efi/boot
	mcopy -i $@ $(KERNEL_FILE) ::
	mcopy -i $@ $(EFI_FILE) ::/efi/boot

$(KERNEL_FILE):$(KERNEL_LIB) $(KERNEL_LD)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $(KERNEL_LIB) -T $(KERNEL_LD)

$(KERNEL_LIB):$(KERNEL_LIB_SRC) $(INITRD)|$(BUILD_DIR)
	# FIXME: Currently `cargo` tries to read `$(pwd)/.cargo/config.toml`, not
	# `$(dirname argument_of_--manifest-path)/.cargo/config.toml`.
	# See: https://github.com/rust-lang/cargo/issues/2930
	cd $(KERNEL_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(TEST_FLAG) $(RUSTCFLAGS)

$(INITRD):$(PM)|$(BUILD_DIR)
	echo $(PM)|cpio -o > $@ --format=odc

$(PM):$(PM_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ -e main $(PM_LIB)

$(PM_LIB):$(PM_LIB_SRC)|$(BUILD_DIR)
	cd $(PM_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(EFI_FILE):$(EFI_SRC)|$(BUILD_DIR)
	cd $(EFI_DIR) && $(RUSTC) build --out-dir=../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(BUILD_DIR):
	mkdir $@ -p

clean:
	$(RM) build
