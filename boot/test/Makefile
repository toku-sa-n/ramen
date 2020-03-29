TEST_TARGET_DIR	:= ../utils
BUILD_DIR		:= build

TEST_TARGET		:= os_sprintf.cpp
TEST_SRC		:= test_utils.cpp

CXXFLAGS		:= -O2 -pipe -march=native

.PHONY:run clean

run:$(BUILD_DIR)/utils_test|$(BUILD_DIR)
	$(BUILD_DIR)/utils_test

$(BUILD_DIR)/utils_test:$(TEST_SRC) $(TEST_TARGET_DIR)/$(TEST_TARGET)|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) -o $@ $^

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

clean:
	rm -rf $(BUILD_DIR)
