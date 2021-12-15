SHELL			:= /bin/bash

define cargo_project_src
	$(shell find $1|grep -v $1/target)
endef

ifeq ($(MAKECMDGOALS), test)
	BUILD_DIR	:=	build/test
else
	BUILD_DIR	:=	build/
endif

LIBS_DIR	:=	libs
SERVERS_DIR	:=	servers

CONFIG_TOML	:=	.cargo/config.toml
CARGO_TOML	:=	Cargo.toml

EFI_DIR			:= bootx64
EFI_SRC	:=	$(shell find $(EFI_DIR)/src)
EFI_SRC	+=	$(EFI_DIR)/$(CONFIG_TOML)
EFI_SRC	+=	$(EFI_DIR)/$(CARGO_TOML)
EFI_FILE		:= $(BUILD_DIR)/bootx64.efi

TERMINAL_DIR	:=	$(LIBS_DIR)/terminal
TERMINAL_SRC	:=	$(call cargo_peojct, $(TERMINAL_DIR))

COMMON_DIR	:=	$(LIBS_DIR)/common
COMMON_SRC	:=	$(call cargo_project_src, $(COMMON_DIR))

SYSCALLS_DIR	:=	$(LIBS_DIR)/syscalls
SYSCALLS_SRC	:=	$(call cargo_project_src, $(SYSCALLS_DIR))

PAGE_BOX_DIR	:=	$(LIBS_DIR)/page_box
PAGE_BOX_SRC	:=	$(call cargo_project_src, $(PAGE_BOX_DIR))

MESSAGE_DIR	:=	$(LIBS_DIR)/message
MESSAGE_SRC	:=	$(call cargo_project_src, $(MESSAGE_DIR))

FRAME_MANAGER_DIR	:=	$(LIBS_DIR)/frame_manager
FRAME_MANAGER_SRC	:=	$(call cargo_project_src, $(FRAME_MANAGER_DIR))

KERNEL_DIR		:= kernel
KERNEL_LIB_SRC	:=	$(call cargo_project_src, $(KERNEL_DIR))
KERNEL_LIB		:= $(BUILD_DIR)/libkernel.a
KERNEL_LIB_DEPENDENCIES_SRC	:=	$(TERMINAL_SRC) $(COMMON_SRC) $(SYSCALLS_SRC) $(PAGE_BOX_SRC) $(MESSAGE_SRC) $(PORT_SERVER_SRC) $(FRAME_MANAGER_SRC)
KERNEL_LD			:= $(KERNEL_DIR)/kernel.ld
KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin

RALIB_DIR	:=	$(LIBS_DIR)/ralib
RALIB_SRC	:=	$(call cargo_project_src, $(RALIB_DIR))

PORT_SERVER_DIR	:=	$(SERVERS_DIR)/port_server
PORT_SERVER_SRC	:=	$(call cargo_project_src, $(PORT_SERVER_DIR))
PORT_SERVER_LIB	:=	$(BUILD_DIR)/libport_server.a
PORT_SERVER_DEPENDENCIES_SRC	:=	$(MESSAGE_SRC) $(RALIB_SRC) $(SYSCALLS_SRC)
PORT_SERVER	:=	$(BUILD_DIR)/port_server.bin

XHCI_DIR	:=	$(SERVERS_DIR)/xhci
XHCI_LIB_SRC	:=	$(call cargo_project_src, $(XHCI_DIR))
XHCI_LIB	:=	$(BUILD_DIR)/libxhci.a
XHCI_LIB_DEPENDENCIES_SRC	:=	$(PAGE_BOX_SRC) $(RALIB_SRC) $(SYSCALLS_SRC)
XHCI	:=	$(BUILD_DIR)/xhci.bin

IMG_FILE		:= $(BUILD_DIR)/ramen_os.img

INITRD			:= $(BUILD_DIR)/initrd.cpio

LD				:= ld
RUSTC			:= cargo
RM				:= rm -rf

RUSTCFLAGS		:= --release
LDFLAGS			:= -nostdlib

QEMU	:=	qemu-system-x86_64
QEMUFLAGS	:=	\
	-drive if=pflash,format=raw,file=OVMF_CODE.fd,readonly=on \
	-drive if=pflash,format=raw,file=OVMF_VARS.fd,readonly=on \
	-drive format=raw,file=$(IMG_FILE) \
	-drive id=usb,file=gpt.img,if=none,format=raw \
	-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
	-device qemu-xhci,id=xhci \
	-device usb-kbd \
	-device usb-mouse \
	-device usb-storage,drive=usb \
	-no-reboot \
	-m 4G \
	--trace events=trace.event \
	-d int

.PHONY:all copy_to_usb test run clean
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

test: QEMUFLAGS	+=	\
	-serial stdio	\
	-display none
test: OK_STATUS	:=	33
test: TEST_FLAG	:=	--features qemu_test
test: $(IMG_FILE)
	cargo test $(RUSTCFLAGS)
	$(QEMU) $(QEMUFLAGS);\
	if [ $$? -eq $(OK_STATUS) ];\
	then\
		echo Test succeeds!;\
	else\
		echo Test failed!;\
		exit 1;\
	fi

run: QEMUFLAGS	+=	\
	-no-shutdown	\
	-monitor stdio
run: $(IMG_FILE)
	$(QEMU) $(QEMUFLAGS)

$(IMG_FILE):$(KERNEL_FILE) $(EFI_FILE) $(INITRD)
	dd if=/dev/zero of=$@ bs=1k count=28800
	mformat -i $@ -h 200 -t 500 -s 144::
	# Cannot replace these mmd and mcopy with `make copy_to_usb` because `mount` needs `sudo`
	# regardless of the permission of the image file or the device. Using `mmd` and `mcopy` is
	# the only way to edit image file without `sudo`.
	mmd -i $@ ::/efi
	mmd -i $@ ::/efi/boot
	mcopy -i $@ $(KERNEL_FILE) ::
	mcopy -i $@ $(INITRD) ::
	mcopy -i $@ $(EFI_FILE) ::/efi/boot

$(KERNEL_FILE):$(KERNEL_LIB) $(KERNEL_LD)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $(KERNEL_LIB) -T $(KERNEL_LD)

$(KERNEL_LIB):$(KERNEL_LIB_SRC) $(KERNEL_LIB_DEPENDENCIES_SRC)|$(BUILD_DIR)
	# FIXME: Currently `cargo` tries to read `$(pwd)/.cargo/config.toml`, not
	# `$(dirname argument_of_--manifest-path)/.cargo/config.toml`.
	# See: https://github.com/rust-lang/cargo/issues/2930
	cd $(KERNEL_DIR) && $(RUSTC) build --out-dir ../$(BUILD_DIR) -Z unstable-options $(TEST_FLAG) $(RUSTCFLAGS)

$(INITRD):$(PORT_SERVER) $(XHCI)|$(BUILD_DIR)
	(cd $(BUILD_DIR); echo $(notdir $(PORT_SERVER)); echo $(notdir $(XHCI))|cpio -o > $(notdir $@) --format=odc)

$(PORT_SERVER):$(PORT_SERVER_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -Ttext 0x800000 -o $@ -e main $^

$(PORT_SERVER_LIB):$(PORT_SERVER_SRC) $(PORT_SERVER_DEPENDENCIES_SRC)|$(BUILD_DIR)
	cd $(PORT_SERVER_DIR) && $(RUSTC) build --out-dir ../../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(EFI_FILE):$(EFI_SRC)|$(BUILD_DIR)
	cd $(EFI_DIR) && $(RUSTC) build --out-dir=../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(XHCI):$(XHCI_LIB)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -Ttext 0x800000 -o $@ -e main $^

$(XHCI_LIB):$(XHCI_LIB_SRC) $(XHCI_LIB_DEPENDENCIES_SRC)|$(BUILD_DIR)
	cd $(XHCI_DIR) && $(RUSTC) build --out-dir ../../$(BUILD_DIR) -Z unstable-options $(RUSTCFLAGS)

$(BUILD_DIR):
	mkdir $@ -p

clean:
	$(RM) build
	cargo clean
