RUST_SRC_DIR	:= src
BUILD_DIR		:= build
EFI_DIR			:= bootx64
EFI_SRC_DIR		:= $(EFI_DIR)/$(RUST_SRC_DIR)
COMMON_SRC_DIR	:= common
KERNEL_DIR		:= kernel
KERNEL_SRC_DIR	:= $(KERNEL_DIR)/$(RUST_SRC_DIR)

CARGO_JSON		:= x86_64-unknown-ramen.json
RUST_SRC		:= $(shell find $(KERNEL_DIR) -name '*.rs')
EFI_SRC			:= $(shell find $(EFI_DIR) -name '*.rs')
CARGO_TOML		:= Cargo.toml
CONFIG_TOML		:= $(KERNEL_DIR)/.cargo/config.toml

COMMON_SRC		:= $(addprefix $(COMMON_SRC_DIR)/$(RUST_SRC_DIR)/, $(shell ls $(COMMON_SRC_DIR)/$(RUST_SRC_DIR)))

LD_SRC			:= $(KERNEL_DIR)/os.ld

EFI_FILE		:= $(BUILD_DIR)/bootx64.efi

KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin
LIB_FILE		:= $(BUILD_DIR)/libramen_os.a
IMG_FILE		:= $(BUILD_DIR)/ramen_os.img
FAT_IMG			:= $(BUILD_DIR)/fat.img

LD				:= ld
RUSTCC			:= cargo
RM				:= rm -rf
VIEWER			:= qemu-system-x86_64

OVMF_CODE		:= OVMF_CODE.fd
OVMF_VARS		:= OVMF_VARS.fd

# If you change values of `iobase` and `iosize`, don't forget to change the corresponding values in `kernel/src/lib.rs`!
VIEWERFLAGS		:= -drive if=pflash,format=raw,file=$(OVMF_CODE),readonly=on -drive if=pflash,format=raw,file=$(OVMF_VARS),readonly=on -drive format=raw,file=$(IMG_FILE) -no-reboot -m 4G -d int -device isa-debug-exit,iobase=0xf4,iosize=0x04 -device qemu-xhci,id=xhci -device usb-kbd --trace events=trace.event -drive id=disk,file=$(FAT_IMG),if=none,format=raw -device ahci,id=ahci -device ide-drive,drive=disk,bus=ahci.0

# This is a workaround for `compiler_builtins` crate which is supported only for optimized build.
RELEASE_FLAGS	:= --release

LDFLAGS			:= -nostdlib -T $(LD_SRC)

.PHONY:all copy_to_usb run test_general test release_test release clippy clean

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

run:$(IMG_FILE) $(OVMF_VARS) $(OVMF_CODE) $(FAT_IMG)
	$(VIEWER) $(VIEWERFLAGS) -no-shutdown -monitor stdio

test_general:$(IMG_FILE)
	make $(IMG_FILE) RELEASE_FLAGS=$(RELEASE_FLAGS) TEST_FLAG=--features=qemu_test -B
	make $(FAT_IMG)
	$(VIEWER) $(VIEWERFLAGS) -nographic; if [[ $$? -eq 33 ]];\
		then echo "Booting test succeed! ($(TEST_MODE) mode)"; exit 0;\
		else echo "Booting test failed ($(TEST_MODE) mode)"; exit 1;fi

test:
	make test_general TEST_MODE=debug

release_test:
	make test_general TEST_MODE=release RELEASE_FLAGS=--release

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

$(FAT_IMG):$(IMG_FILE)
	cp $^ $@

release:
	make RELEASE_FLAGS=--release -B

release_run:
	make release && make run

$(KERNEL_FILE):$(LIB_FILE) $(LD_SRC)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $(LIB_FILE)

$(LIB_FILE): $(RUST_SRC) $(COMMON_SRC) $(COMMON_SRC_DIR)/$(CARGO_TOML) $(KERNEL_DIR)/$(CARGO_TOML) $(KERNEL_DIR)/$(CARGO_JSON) $(CONFIG_TOML)|$(BUILD_DIR)
	# FIXME: Currently `cargo` tries to read `$(pwd)/.cargo/config.toml`, not
	# `$(dirname argument_of_--manifest-path)/.cargo/config.toml`.
	# See: https://github.com/rust-lang/cargo/issues/2930
	cd $(KERNEL_DIR) && $(RUSTCC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(RELEASE_FLAGS) $(TEST_FLAG)

%.fd:
	@echo "$@ not found"
	exit 1

$(EFI_FILE):$(EFI_SRC) $(COMMON_SRC) $(COMMON_SRC_DIR)/$(CARGO_TOML) $(EFI_DIR)/$(CARGO_TOML)|$(BUILD_DIR)
	cd $(EFI_DIR) && $(RUSTCC) build --out-dir=../$(BUILD_DIR) -Z unstable-options $(RELEASE_FLAGS)

$(BUILD_DIR):
	mkdir $@ -p

clippy:
	(cd $(KERNEL_DIR) && $(RUSTCC) clippy)
	(cd $(EFI_DIR) && $(RUSTCC) clippy)

clean:
	$(RM) build
	$(RUSTCC) clean --manifest-path=$(KERNEL_DIR)/Cargo.toml
	$(RUSTCC) clean --manifest-path=$(EFI_DIR)/Cargo.toml
