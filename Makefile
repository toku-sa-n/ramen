RUST_SRC_DIR:= src
BUILD_DIR	:= build
ASM_DIR		:= asm
BOOT_DIR	:= boot

HEAD_SRC	:= $(ASM_DIR)/head.asm
CARGO_JSON	:= cargo_settings
RUST_SRC	:= $(shell cd $(RUST_SRC_DIR) && ls)

LD_SRC		:= os.ld

HEAD_FILE	:= $(BUILD_DIR)/head.asm.o
EFI_FILE	:= $(BOOT_DIR)/$(BUILD_DIR)/bootx64.efi
IMG_FILE	:= $(BUILD_DIR)/ramen_os.img

HEAD_DEPENDS:= $(ASM_DIR)/paging_64.asm

KERNEL_FILE	:= $(BUILD_DIR)/kernel.bin
LIB_FILE	:= $(BUILD_DIR)/libramen_os.a

ASMC		:= nasm
CAT			:= cat
LD			:= ld
RUSTCC		:= cargo
RM			:= rm -rf
VIEWER		:= qemu-system-x86_64

OVMF_CODE	:= OVMF_CODE-pure-efi.fd
OVMF_VARS	:= OVMF_VARS-pure-efi.fd

LDFLAGS := -nostdlib -T $(LD_SRC)
ASMFLAGS := -w+all -i $(ASM_DIR)/
VIEWERFLAGS := -drive if=pflash,format=raw,file=$(OVMF_CODE),readonly=on -drive if=pflash,format=raw,file=$(OVMF_VARS),readonly=on -drive format=raw,file=$(IMG_FILE) -monitor stdio -no-reboot -no-shutdown -m 4G

.PHONY:all run release clean $(EFI_FILE)

.SUFFIXES:

all:$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)

run:$(IMG_FILE) $(OVMF_CODE) $(OVMF_VARS)
	$(VIEWER) $(VIEWERFLAGS)

$(OVMF_CODE):
	@echo "$@ not found."
	exit 1

$(OVMF_VARS):
	@echo "$@ not found."
	exit 1

$(IMG_FILE):$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)
	dd if=/dev/zero of=$@ bs=1k count=1440
	mformat -i $@ -f 1440 ::
	mmd -i $@ ::/efi
	mmd -i $@ ::/efi/boot
	mcopy -i $@ $(KERNEL_FILE) ::
	mcopy -i $@ $(HEAD_FILE) ::
	mcopy -i $@ $(EFI_FILE) ::/efi/boot

copy_to_usb:$(KERNEL_FILE) $(HEAD_FILE) $(EFI_FILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo mkdir -p /mnt/efi/boot/
	sudo cp $(EFI_FILE) /mnt/efi/boot/
	sudo cp $(KERNEL_FILE) /mnt/
	sudo cp $(HEAD_FILE) /mnt/
	sudo umount /mnt
endif

release:$(KERNEL_FILE) $(HEAD_FILE) $(LD_SRC)|$(BUILD_DIR)
	make clean
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR) --release
	cp $(BUILD_DIR)/$(CARGO_JSON)/$@/$(shell basename $(LIB_FILE))  $(LIB_FILE)
	make

$(KERNEL_FILE):$(LIB_FILE) $(LD_SRC)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $<

$(LIB_FILE): $(addprefix $(RUST_SRC_DIR)/, $(RUST_SRC))|$(BUILD_DIR)
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR)
	cp $(BUILD_DIR)/$(CARGO_JSON)/debug/$(shell basename $(LIB_FILE)) $@

$(HEAD_FILE):$(HEAD_DEPENDS)

$(BUILD_DIR)/%.asm.o:$(ASM_DIR)/%.asm|$(BUILD_DIR)
	$(ASMC) $(ASMFLAGS) -o $@ $<

$(EFI_FILE):
	make -C $(BOOT_DIR)

$(BUILD_DIR):
	mkdir $@

clean:
	$(RM) build
	make -C $(BOOT_DIR) clean
