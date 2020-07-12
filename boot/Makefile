CXX = x86_64-w64-mingw32-g++
CXXFLAGS = -shared -nostdlib -mno-red-zone -fno-stack-protector -Wall -std=c++17 -I include/ -O1 -pipe

BUILD_DIR = build
SRC_DIR = src
UTILS_DIR = utils

CXX_SRC = bootx64.cpp $(UTILS_DIR)/os_sprintf.cpp efi_utils.cpp gop.cpp
CXX_SRC_PATH = $(addprefix $(SRC_DIR))/, $(CXX_SRC))
CXX_OBJ = $(addprefix $(BUILD_DIR)/, $(CXX_SRC:%.cpp=%.o))

DEPS = $(addprefix $(BUILD_DIR)/, $(CXX_SRC:%.cpp=%.d))

EFIFILE = $(BUILD_DIR)/bootx64.efi

.SUFFIXES:

.PHONY=all copy_to_usb clean

all:$(EFIFILE)

$(BUILD_DIR)/%.efi:$(BUILD_DIR)/%.dll|$(BUILD_DIR)
	objcopy --target=efi-app-x86_64 $< $@

$(BUILD_DIR)/%.dll:$(CXX_OBJ)|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) $^ -e EfiMain -o $@

# TODO: Fix the problem of not compiling files correctly.
# (See the output of `make && make`.)
$(BUILD_DIR)/%.o:$(SRC_DIR)/%.cpp|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) $< -c -MMD -MP -o $@

$(BUILD_DIR):
	mkdir -p $@/$(UTILS_DIR)

clean:
	rm -rf $(BUILD_DIR)

-include $(DEPS)
