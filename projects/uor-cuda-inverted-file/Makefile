CXX ?= c++
CPPFLAGS ?= -Iinclude -Isrc
CXXFLAGS ?= -std=c++17 -Wall -Wextra -Werror -pedantic -O2
BUILD_DIR := build

.PHONY: all host-test test cuda cuda-test clean
all: host-test

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/admission_test: src/admission.cpp src/admission.hpp tests/admission_test.cpp | $(BUILD_DIR)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) src/admission.cpp tests/admission_test.cpp -o $@

$(BUILD_DIR)/inverted_file_test: src/inverted_file.cpp src/codebook.cpp include/uor_inverted_file.hpp tests/inverted_file_test.cpp | $(BUILD_DIR)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) src/inverted_file.cpp src/codebook.cpp tests/inverted_file_test.cpp -o $@

host-test: $(BUILD_DIR)/admission_test $(BUILD_DIR)/inverted_file_test
	./$(BUILD_DIR)/admission_test
	./$(BUILD_DIR)/inverted_file_test

test: host-test

cuda:
	@command -v nvcc >/dev/null 2>&1 || { echo 'NOT_RUN: nvcc is unavailable; CUDA compilation was not performed'; exit 2; }

cuda-test:
	@command -v nvcc >/dev/null 2>&1 || { echo 'CudaUnavailable: CUDA tests were not run'; exit 2; }

clean:
	rm -rf $(BUILD_DIR) dist
