CC = x86_64-w64-mingw32-g++
CFLAGS = -shared -nostdlib -mno-red-zone -fno-stack-protector -Wall -e EfiMain -std=c++17 -I include/

BUILD_DIR = build
SRC_DIR = src

EFIFILE = $(BUILD_DIR)/bootx64.efi

.SUFFIXES:

.PHONY=all copy_to_usb clean

all:$(EFIFILE)

$(BUILD_DIR)/%.efi:$(BUILD_DIR)/%.dll|$(BUILD_DIR)
	objcopy --target=efi-app-x86_64 $< $@

$(BUILD_DIR)/%.dll:$(SRC_DIR)/%.cpp $(SRC_DIR)/utils/os_sprintf.cpp $(SRC_DIR)/efi_utils.cpp $(SRC_DIR)/gop.cpp |$(BUILD_DIR)
	$(CC) $(CFLAGS) $^ -o $@

copy_to_usb:$(EFIFILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo cp $(EFIFILE) /mnt/efi/boot/
	sudo umount /mnt
endif

$(BUILD_DIR):
	mkdir -p $@

clean:
	rm -rf $(BUILD_DIR)
