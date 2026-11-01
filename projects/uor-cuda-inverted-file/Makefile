CXX ?= c++
CPPFLAGS ?= -Iinclude -Isrc
CXXFLAGS ?= -std=c++17 -Wall -Wextra -Werror -pedantic -O2
BUILD_DIR := build

.PHONY: all host-test test cuda cuda-test check-nvcc evidence clean
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

NVCC ?= nvcc
UOR_NVCC_ARCH ?= compute_80
SM_ARCH = $(subst compute_,sm_,$(UOR_NVCC_ARCH))

$(BUILD_DIR)/admit: src/admission.cpp src/admission.hpp src/admit_main.cpp | $(BUILD_DIR)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) src/admission.cpp src/admit_main.cpp -o $@

$(BUILD_DIR)/uor_inverted_file.ptx: src/kernels.cu | $(BUILD_DIR)
	$(NVCC) -ptx -std=c++17 -arch=$(UOR_NVCC_ARCH) -o $@ $<

$(BUILD_DIR)/uor_inverted_file.cubin: $(BUILD_DIR)/uor_inverted_file.ptx $(BUILD_DIR)/admit
	./$(BUILD_DIR)/admit $(BUILD_DIR)/uor_inverted_file.ptx
	$(NVCC) -cubin --gpu-code=$(SM_ARCH) -o $@ $(BUILD_DIR)/uor_inverted_file.ptx

$(BUILD_DIR)/cuda_parity_test: src/cuda_driver.cpp src/cuda_driver.hpp src/inverted_file.cpp src/codebook.cpp tests/cuda_parity_test.cpp $(BUILD_DIR)/uor_inverted_file.cubin
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) src/cuda_driver.cpp src/inverted_file.cpp src/codebook.cpp tests/cuda_parity_test.cpp -lcuda -o $@

check-nvcc:
	@command -v $(NVCC) >/dev/null 2>&1 || { echo 'NOT_RUN: nvcc is unavailable; CUDA compilation and tests were not performed'; exit 2; }

cuda: check-nvcc $(BUILD_DIR)/uor_inverted_file.cubin
	@echo 'AOT CUDA module built and admitted'

cuda-test: check-nvcc $(BUILD_DIR)/cuda_parity_test
	./$(BUILD_DIR)/cuda_parity_test

evidence:
	@mkdir -p evidence
	@$(MAKE) -s clean host-test > evidence/host-test-1.log 2>&1
	@$(MAKE) -s clean host-test > evidence/host-test-2.log 2>&1
	@cmp evidence/host-test-1.log evidence/host-test-2.log
	@rm -rf $(BUILD_DIR)/clean && mkdir -p $(BUILD_DIR)/clean
	@tar --exclude=build --exclude=dist -cf - . | tar -xf - -C $(BUILD_DIR)/clean
	@$(MAKE) -s -C $(BUILD_DIR)/clean clean host-test > evidence/clean-clone-test.log 2>&1
	@cmp evidence/host-test-1.log evidence/clean-clone-test.log
	@shasum -a 256 evidence/host-test-1.log evidence/host-test-2.log evidence/clean-clone-test.log

clean:
	rm -rf $(BUILD_DIR) dist
