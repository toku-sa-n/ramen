CXX = x86_64-w64-mingw32-g++
CXXFLAGS = -shared -nostdlib -mno-red-zone -fno-stack-protector -Wall -std=c++17 -I include/ -O2 -pipe

BUILD_DIR = build
SRC_DIR = src
UTILS_DIR = utils

CXX_SRC = bootx64.cpp $(UTILS_DIR)/os_sprintf.cpp efi_utils.cpp gop.cpp
CXX_SRC_PATH = $(addprefix $(SRC_DIR))/, $(CXX_SRC))
CXX_OBJ = $(addprefix $(BUILD_DIR)/, $(CXX_SRC:%.cpp=%.o))

EFIFILE = $(BUILD_DIR)/bootx64.efi

.SUFFIXES:

.PHONY=all copy_to_usb clean

all:$(EFIFILE)

$(BUILD_DIR)/%.efi:$(BUILD_DIR)/%.dll|$(BUILD_DIR)
	objcopy --target=efi-app-x86_64 $< $@

$(BUILD_DIR)/%.dll:$(CXX_OBJ)|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) $^ -e EfiMain -o $@

$(BUILD_DIR)/%.o:$(SRC_DIR)/%.cpp|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) $^ -c -o $@

copy_to_usb:$(EFIFILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo cp $(EFIFILE) /mnt/efi/boot/
	sudo umount /mnt
endif

$(BUILD_DIR):
	mkdir -p $@/$(UTILS_DIR)

clean:
	rm -rf $(BUILD_DIR)
