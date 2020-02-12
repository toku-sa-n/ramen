RUST_SRC_DIR:= src
BUILD_DIR	:= build
ASM_DIR		:= asm

IPL_SRC		:= $(ASM_DIR)/ipl.asm
HEAD_SRC	:= $(ASM_DIR)/head.asm
CARGO_JSON	:= cargo_settings
RUST_SRC	:= $(shell cd $(RUST_SRC_DIR) && ls)

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

LDFLAGS := -nostdlib -m elf_i386 -T $(LD_SRC)
ASMFLAGS := -w+all

.PHONY:run release clean

.SUFFIXES:

$(IMG_FILE):$(IPL_FILE) $(SYS_FILE)|$(BUILD_DIR)
	mformat -f 1440 -C -B $(IPL_FILE) -i $@ ::
	mcopy $(SYS_FILE) -i $@ ::

release:$(IPL_FILE) $(HEAD_FILE) $(LD_SRC)|$(BUILD_DIR)
	make clean
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR) --release
	cp $(BUILD_DIR)/$(CARGO_JSON)/$@/$(shell basename $(LIB_FILE))  $(LIB_FILE)
	make $(IMG_FILE)

$(SYS_FILE):$(HEAD_FILE) $(KERNEL_FILE)|$(BUILD_DIR)
	$(CAT) $^ > $@

$(KERNEL_FILE):$(LIB_FILE) $(LD_SRC)|$(BUILD_DIR)
	$(LD) $(LDFLAGS) -o $@ $<

$(LIB_FILE): $(addprefix $(RUST_SRC_DIR)/, $(RUST_SRC))|$(BUILD_DIR)
	$(RUSTCC) xbuild --target-dir $(BUILD_DIR)
	cp $(BUILD_DIR)/$(CARGO_JSON)/debug/$(shell basename $(LIB_FILE)) $@

$(BUILD_DIR)/%.asm.o:$(ASM_DIR)/%.asm|$(BUILD_DIR)
	$(ASMC) $(ASMFLAGS) -o $@ $<

run:$(IMG_FILE)
	make $^
	$(VIEWER) -drive file=$<,format=raw,if=floppy -monitor stdio

$(BUILD_DIR):
	mkdir $@

clean:
	$(RM) build
