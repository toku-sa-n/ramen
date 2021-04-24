SHELL			:= /bin/bash

BUILD_DIR		:= build

CONFIG_TOML	:=	.cargo/config.toml
CARGO_TOML	:=	Cargo.toml

EFI_DIR			:= bootx64
EFI_SRC	:=	$(shell find $(EFI_DIR)/src)
EFI_SRC	+=	$(EFI_DIR)/$(CONFIG_TOML)
EFI_SRC	+=	$(EFI_DIR)/$(CARGO_TOML)
EFI_FILE		:= $(BUILD_DIR)/bootx64.efi

TERMINAL_DIR	:=	terminal
TERMINAL_SRC	:=	$(shell find $(TERMINAL_DIR)/src)
TERMINAL_SRC	+=	$(TERMINAL_DIR)/$(CARGO_TOML)

COMMON_DIR	:=	common
COMMON_SRC	:=	$(shell find $(COMMON_DIR)/src)
COMMON_SRC	+=	$(COMMON_DIR)/$(CARGO_TOML)

SYSCALLS_DIR	:=	syscalls
SYSCALLS_SRC	:=	$(shell find $(SYSCALLS_DIR)/src)
SYSCALLS_SRC	+=	$(SYSCALLS_DIR)/$(CARGO_TOML)

PAGE_BOX_DIR	:=	page_box
PAGE_BOX_SRC	:=	$(shell find $(PAGE_BOX_DIR)/src)
PAGE_BOX_SRC	+=	$(PAGE_BOX_DIR)/$(CARGO_TOML)

MESSAGE_DIR	:=	message
MESSAGE_SRC	:=	$(shell find $(MESSAGE_DIR)/src)
MESSAGE_SRC	+=	$(MESSAGE_DIR)/$(CARGO_TOML)

FRAME_MANAGER_DIR	:=	frame_manager
FRAME_MANAGER_SRC	:=	$(shell find $(FRAME_MANAGER_DIR)/src)
FRAME_MANAGER_SRC	+=	$(FRAME_MANAGER_DIR)/$(CARGO_TOML)

FS_SERVER_DIR	:=	fs_server
FS_SERVER_SRC	:=	$(shell find $(FS_SERVER_DIR)/src)
FS_SERVER_SRC	+=	$(FS_SERVER_DIR)/$(CARGO_TOML)
FS_SERVER_LIB	:=	$(BUILD_DIR)/libfs_server.a
FS_SERVER_LIB_DEPENDENCIES_SRC	:=	$(SYSCALLS_SRC) $(RALIB_SRC) $(MESSAGE_SRC)
FS_SERVER	:=	$(BUILD_DIR)/fs_server.bin

KERNEL_DIR		:= kernel
KERNEL_LIB_SRC	:=	$(shell find $(KERNEL_DIR)/src)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/$(CONFIG_TOML)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/$(CARGO_TOML)
KERNEL_LIB_SRC	+=	$(KERNEL_DIR)/build.rs
KERNEL_LIB		:= $(BUILD_DIR)/libramen_os.a
KERNEL_LIB_DEPENDENCIES_SRC	:=	$(TERMINAL_SRC) $(COMMON_SRC) $(SYSCALLS_SRC) $(PAGE_BOX_SRC) $(MESSAGE_SRC) $(PORT_SERVER_SRC) $(FRAME_MANAGER_SRC) $(FS_SERVER_SRC)
KERNEL_LD			:= $(KERNEL_DIR)/kernel.ld
KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin

RALIB_DIR	:=	ralib
RALIB_SRC	:=	$(shell find $(RALIB_DIR)/src)
RALIB_SRC	+=	$(RALIB_DIR)/$(CARGO_TOML)

PORT_SERVER_DIR	:=	port_server
PORT_SERVER_SRC	:=	$(shell find $(PORT_SERVER_DIR)/src)
PORT_SERVER_SRC	+=	$(PORT_SERVER_DIR)/$(CARGO_TOML)
PORT_SERVER_LIB	:=	$(BUILD_DIR)/libport_server.a
PORT_SERVER_DEPENDENCIES_SRC	:=	$(MESSAGE_SRC) $(RALIB_SRC) $(SYSCALLS_SRC)
PORT_SERVER	:=	$(BUILD_DIR)/port_server.bin

PM_DIR			:= pm
PM_LIB_SRC	:=	$(shell find $(PM_DIR)/src)
PM_LIB_SRC	+=	$(PM_DIR)/$(CONFIG_TOML)
PM_LIB_SRC	+=	$(PM_DIR)/$(CARGO_TOML)
PM_LIB			:= $(BUILD_DIR)/libpm.a
PM_LIB_DEPENDENCIES_SRC	:=	$(MESSAGE_SRC) $(RALIB_SRC) $(SYSCALLS_SRC)
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

$(KERNEL_LIB):$(KERNEL_LIB_SRC) $(INITRD) $(KERNEL_LIB_DEPENDENCIES_SRC)|$(BUILD_DIR)
	# FIXME: Currently `cargo` tries to read `$(pwd)/.cargo/config.toml`, not
	# `$(dirname argument_of_--manifest-path)/.cargo/config.toml`.
	# See: https://github.com/rust-lang/cargo/issues/2930
	cd $(KERNEL_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(TEST_FLAG) $(RUSTCFLAGS)

$(INITRD):$(PM) $(PORT_SERVER) $(FS_SERVER)|$(BUILD_DIR)
	(echo $(PM); echo $(PORT_SERVER); echo $(FS_SERVER))|cpio -o > $@ --format=odc

$(PM):$(PM_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ -e main $(PM_LIB)

$(PM_LIB):$(PM_LIB_SRC) $(PM_LIB_DEPENDENCIES_SRC)|$(BUILD_DIR)
	cd $(PM_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(PORT_SERVER):$(PORT_SERVER_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ -e main $^

$(PORT_SERVER_LIB):$(PORT_SERVER_SRC) $(PORT_SERVER_DEPENDENCIES_SRC)|$(BUILD_DIR)
	cd $(PORT_SERVER_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(FS_SERVER):$(FS_SERVER_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ -e main $^

$(FS_SERVER_LIB):$(FS_SERVER_SRC) $(FS_SERVER_LIB_DEPENDENCIES_SRC)|$(BUILD_DIR)
	cd $(FS_SERVER_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(EFI_FILE):$(EFI_SRC)|$(BUILD_DIR)
	cd $(EFI_DIR) && $(RUSTC) build --out-dir=../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(BUILD_DIR):
	mkdir $@ -p

clean:
	$(RM) build
