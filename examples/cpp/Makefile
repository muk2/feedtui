# Makefile for feedtui C++ example
#
# This Makefile demonstrates how to build a C++ application that uses
# the feedtui FFI interface.

# Configuration
CXX ?= g++
CXXFLAGS = -std=c++17 -Wall -Wextra -O2
LDFLAGS =

# Paths
FEEDTUI_ROOT = ../..
INCLUDE_DIR = $(FEEDTUI_ROOT)/include
LIB_DIR = $(FEEDTUI_ROOT)/target/release

# Platform-specific settings
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -lpthread -ldl -lm
    LIB_NAME = libfeedtui.so
    RUN_PREFIX = LD_LIBRARY_PATH=$(LIB_DIR)
endif
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -framework Security -framework CoreFoundation
    LIB_NAME = libfeedtui.dylib
    RUN_PREFIX = DYLD_LIBRARY_PATH=$(LIB_DIR)
endif

# Targets
TARGET = feedtui_example

.PHONY: all clean run build-lib

all: $(TARGET)

# Build the C++ example
$(TARGET): main.cpp $(LIB_DIR)/$(LIB_NAME)
	$(CXX) $(CXXFLAGS) -o $@ main.cpp \
		-I$(INCLUDE_DIR) \
		-L$(LIB_DIR) \
		-lfeedtui \
		$(LDFLAGS)

# Build the Rust library with FFI support
$(LIB_DIR)/$(LIB_NAME): build-lib

build-lib:
	@echo "Building feedtui library with FFI support..."
	cd $(FEEDTUI_ROOT) && cargo build --release --features ffi

# Run the example
run: $(TARGET)
	$(RUN_PREFIX) ./$(TARGET)

# Run with custom config
run-config: $(TARGET)
	$(RUN_PREFIX) ./$(TARGET) -c ../../examples/config.toml

# Clean build artifacts
clean:
	rm -f $(TARGET)

# Show help
help:
	@echo "feedtui C++ Example Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all       - Build the example (default)"
	@echo "  build-lib - Build the Rust library with FFI support"
	@echo "  run       - Build and run with default config"
	@echo "  run-config- Build and run with example config file"
	@echo "  clean     - Remove build artifacts"
	@echo "  help      - Show this help message"
