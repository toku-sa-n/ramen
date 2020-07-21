RUST_SRC_DIR	:= src
BUILD_DIR		:= build
ASM_DIR			:= asm
BOOT_DIR		:= bootx64
EFI_SRC_DIR		:= $(BOOT_DIR)/$(RUST_SRC_DIR)

HEAD_SRC		:= $(ASM_DIR)/head.asm
CARGO_JSON		:= cargo_settings
RUST_SRC		:= $(shell cd $(RUST_SRC_DIR) && ls)
EFI_SRC			:= $(shell cd $(EFI_SRC_DIR) && ls)

LD_SRC			:= os.ld

HEAD_FILE		:= $(BUILD_DIR)/head.asm.o
EFI_FILE		:= $(BOOT_DIR)/target/x86_64-unknown-uefi/debug/bootx64.efi

HEAD_DEPENDS	:= $(ASM_DIR)/paging_64.asm

KERNEL_FILE		:= $(BUILD_DIR)/kernel.bin
LIB_FILE		:= $(BUILD_DIR)/libramen_os.a
IMG_FILE		:= $(BUILD_DIR)/ramen_os.img

ASMC			:= nasm
CAT				:= cat
LD				:= ld
RUSTCC			:= cargo
RM				:= rm -rf
VIEWER			:= qemu-system-x86_64

OVMF_CODE		:= OVMF_CODE-pure-efi.fd
OVMF_VARS		:= OVMF_VARS-pure-efi.fd

VIEWERFLAGS		:= -drive if=pflash,format=raw,file=$(OVMF_CODE),readonly=on -drive if=pflash,format=raw,file=$(OVMF_VARS),readonly=on -drive format=raw,file=$(IMG_FILE) -monitor stdio -no-reboot -no-shutdown -m 4G -d int

LDFLAGS			:= -nostdlib -T $(LD_SRC)
ASMFLAGS		:= -w+all -i $(ASM_DIR)/

.PHONY:all show_kernel_map run release clean

.SUFFIXES:

all:$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)

copy_to_usb:$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo mkdir -p /mnt/efi/boot
	sudo cp $(EFI_FILE) /mnt/efi/boot/
	sudo cp $(KERNEL_FILE) /mnt/
	sudo cp $(HEAD_FILE) /mnt/
	sudo umount /mnt
endif

run:$(IMG_FILE) $(OVMF_VARS) $(OVMF_CODE)
	$(VIEWER) $(VIEWERFLAGS)

$(IMG_FILE):$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)
	dd if=/dev/zero of=$@ bs=1k count=2880
	mformat -i $@ -f 2880 ::
	mmd -i $@ ::/efi
	mmd -i $@ ::/efi/boot
	mcopy -i $@ $(KERNEL_FILE) ::
	mcopy -i $@ $(HEAD_FILE) ::
	mcopy -i $@ $(EFI_FILE) ::/efi/boot

release:
	make clean
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR) --release
	$(RUSTCC) xbuild --target=x86_64-unknown-uefi --manifest-path=$(BOOT_DIR)/Cargo.toml --release
	cp $(BUILD_DIR)/$(CARGO_JSON)/$@/$(shell basename $(LIB_FILE))  $(LIB_FILE)
	mkdir -p $(BOOT_DIR)/target/x86_64-unknown-uefi/debug
	cp $(BOOT_DIR)/target/x86_64-unknown-uefi/$@/bootx64.efi $(BOOT_DIR)/target/x86_64-unknown-uefi/debug/bootx64.efi
	make

$(KERNEL_FILE):$(LIB_FILE) $(LD_SRC)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $<

$(LIB_FILE): $(addprefix $(RUST_SRC_DIR)/, $(RUST_SRC))|$(BUILD_DIR)
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR)
	cp $(BUILD_DIR)/$(CARGO_JSON)/debug/$(shell basename $(LIB_FILE)) $@

$(HEAD_FILE):$(HEAD_DEPENDS)

$(BUILD_DIR)/%.asm.o:$(ASM_DIR)/%.asm|$(BUILD_DIR)
	$(ASMC) $(ASMFLAGS) -o $@ $<

$(OVMF_CODE):
	@echo "$@ not found."
	exit 1

$(OVMF_VARS):
	@echo "$@ not found."
	exit 1

$(EFI_FILE):$(addprefix $(EFI_SRC_DIR)/, $(EFI_SRC))
	$(RUSTCC) xbuild --target=x86_64-unknown-uefi --manifest-path=$(BOOT_DIR)/Cargo.toml

$(BUILD_DIR):
	mkdir $@

clean:
	$(RM) build
	$(RUSTCC) clean
	$(RUSTCC) clean --manifest-path=$(BOOT_DIR)/Cargo.toml
