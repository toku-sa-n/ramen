BUILD_DIR	:= build
ASM_DIR		:= asm

IPL_SRC		:= $(ASM_DIR)/ipl.asm
HEAD_SRC	:= $(ASM_DIR)/head.asm
CARGO_JSON	:= cargo_settings

LD_SRC		:= os.ld

IPL_FILE	:= $(BUILD_DIR)/ipl.asm.o
HEAD_FILE	:= $(BUILD_DIR)/head.asm.o

KERNEL_FILE	:= $(BUILD_DIR)/kernel.bin
IMG_FILE	:= $(BUILD_DIR)/ramen_os.img
SYS_FILE	:= $(BUILD_DIR)/ramen_os.sys
LIB_FILE	:= $(BUILD_DIR)/libramen_os.a

ASMC		:= nasm
CAT			:= cat
VIEWER		:= qemu-system-i386
LD			:= ld
RUSTCC		:= cargo
RM			:= rm -rf

LDFLAGS := -nostdlib -m elf_i386 -Tdata=0x00310000 -T $(LD_SRC)

.PHONY:run clean

.SUFFIXES:

$(IMG_FILE):$(IPL_FILE) $(SYS_FILE)|$(BUILD_DIR)
	mformat -f 1440 -C -B $(IPL_FILE) -i $@ ::
	mcopy $(SYS_FILE) -i $@ ::

$(SYS_FILE):$(HEAD_FILE) $(KERNEL_FILE)|$(BUILD_DIR)
	$(CAT) $^ > $@

$(KERNEL_FILE):$(LIB_FILE)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $<

$(LIB_FILE):|$(BUILD_DIR)
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR)
	cp $(BUILD_DIR)/$(CARGO_JSON)/debug/$(shell basename $(LIB_FILE)) $@

$(BUILD_DIR)/%.asm.o:$(ASM_DIR)/%.asm|$(BUILD_DIR)
	$(ASMC) -o $@ $<

run:$(IMG_FILE)
	make $^
	$(VIEWER) -drive file=$<,format=raw,if=floppy

$(BUILD_DIR):
	mkdir $@

clean:
	$(RM) build
