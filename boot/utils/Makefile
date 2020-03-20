BUILD_DIR		:= ../build/utils

CXX_SRC			:= os_sprintf.cpp

OBJ				:= $(addprefix $(BUILD_DIR)/, $(CXX_SRC:%.cpp=%.cpp.o))

CXX				:= gcc
CXXFLAGS		:= -march=i486 -m32 -nostdlib -O0 -g -fno-pie -Wall -I ../include

all:$(OBJ)

$(BUILD_DIR)/%.cpp.o:%.cpp|$(BUILD_DIR)
	$(CXX) $(CXXFLAGS) -c -o $@ $<

$(BUILD_DIR):
	mkdir $@ -p
