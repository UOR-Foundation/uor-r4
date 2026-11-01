#include "cuda_driver.hpp"

#include "sha256.hpp"

#include <cuda.h>

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iterator>
#include <vector>

namespace uor_inverted_file::cuda {
namespace {

struct Buffer {
    CUdeviceptr ptr = 0;
    std::size_t bytes = 0;
    std::size_t capacity_bytes = 0;

    Buffer() = default;
    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;
    ~Buffer() { if (ptr != 0) cuMemFree(ptr); }
};

Status allocate(Buffer& buffer, std::size_t bytes) {
    if (bytes == 0) return Status::InvalidArgument;
    buffer.bytes = bytes;
    buffer.capacity_bytes = bytes;
    return cuMemAlloc(&buffer.ptr, bytes) == CUDA_SUCCESS ? Status::Ok : Status::CudaFailure;
}

Status write(const Buffer& buffer, const void* source) {
    return cuMemcpyHtoD(buffer.ptr, source, buffer.bytes) == CUDA_SUCCESS
        ? Status::Ok : Status::CudaFailure;
}

Status read(void* destination, const Buffer& buffer, std::size_t bytes) {
    if (bytes > buffer.capacity_bytes) return Status::InvalidArgument;
    return cuMemcpyDtoH(destination, buffer.ptr, bytes) == CUDA_SUCCESS
        ? Status::Ok : Status::CudaFailure;
}

Status read(void* destination, const Buffer& buffer) {
    return read(destination, buffer, buffer.bytes);
}

template <typename T>
Status upload(const std::vector<T>& source, Buffer& buffer) {
    const auto status = allocate(buffer, source.size() * sizeof(T));
    if (status != Status::Ok) return status;
    return write(buffer, source.data());
}

template <typename T>
Status upload_capacity(const std::vector<T>& source, std::size_t capacity, Buffer& buffer) {
    if (source.size() > capacity) return Status::InvalidArgument;
    const auto status = allocate(buffer, capacity * sizeof(T));
    if (status != Status::Ok) return status;
    buffer.bytes = source.size() * sizeof(T);
    return write(buffer, source.data());
}

template <typename T, std::size_t N>
Status upload(const std::array<T, N>& source, Buffer& buffer) {
    const auto status = allocate(buffer, N * sizeof(T));
    if (status != Status::Ok) return status;
    return write(buffer, source.data());
}

struct Session {
    CUcontext context = nullptr;
    CUmodule module = nullptr;
    CUfunction build_fn = nullptr;
    CUfunction update_fn = nullptr;
    CUfunction probe_fn = nullptr;
    CUfunction full_scan_fn = nullptr;

    Session() = default;
    Session(const Session&) = delete;
    Session& operator=(const Session&) = delete;
    ~Session() {
        if (module != nullptr) cuModuleUnload(module);
        if (context != nullptr) cuCtxDestroy(context);
    }
};

Status open(Session& session) {
    if (cuInit(0) != CUDA_SUCCESS) return Status::CudaUnavailable;
    int count = 0;
    if (cuDeviceGetCount(&count) != CUDA_SUCCESS || count == 0) return Status::CudaUnavailable;
    CUdevice device = 0;
    if (cuDeviceGet(&device, 0) != CUDA_SUCCESS) return Status::CudaUnavailable;
    if (cuCtxCreate(&session.context, 0, device) != CUDA_SUCCESS) return Status::CudaUnavailable;
    if (cuModuleLoad(&session.module, "build/uor_inverted_file.cubin") != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    if (cuModuleGetFunction(&session.build_fn, session.module, "uor_if_build") != CUDA_SUCCESS ||
        cuModuleGetFunction(&session.update_fn, session.module, "uor_if_update") != CUDA_SUCCESS ||
        cuModuleGetFunction(&session.probe_fn, session.module, "uor_if_probe") != CUDA_SUCCESS ||
        cuModuleGetFunction(&session.full_scan_fn, session.module, "uor_if_full_scan") != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    return Status::Ok;
}

Status synchronize() {
    return cuCtxSynchronize() == CUDA_SUCCESS ? Status::Ok : Status::CudaFailure;
}

Status status_from_kernel(std::uint32_t status) {
    switch (status) {
        case 0: return Status::Ok;
        case 1: return Status::InvalidArgument;
        case 2: return Status::NonRootBlock;
        case 3: return Status::CsrInvariant;
        default: return Status::CudaFailure;
    }
}

std::vector<std::int32_t> table_bytes(const Table& table) {
    std::vector<std::int32_t> output;
    output.reserve(table.tokens.size() * 32);
    for (const auto& token : table.tokens) {
        for (const auto& block : token) output.insert(output.end(), block.begin(), block.end());
    }
    return output;
}

std::vector<std::int32_t> roots_bytes() {
    std::vector<std::int32_t> output;
    for (const auto& root : codebook) output.insert(output.end(), root.begin(), root.end());
    return output;
}

std::vector<std::int32_t> query_bytes(const Query& query) {
    std::vector<std::int32_t> output;
    for (const auto& block : query) output.insert(output.end(), block.begin(), block.end());
    return output;
}

} // namespace

Status benchmark_file_sha256(const std::string& path, std::string& digest) {
    std::ifstream input(path, std::ios::binary);
    if (!input) return Status::InvalidArgument;
    const std::vector<char> bytes((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
    if (!input.eof()) return Status::InvalidArgument;
    digest = sha256_hex(reinterpret_cast<const std::uint8_t*>(bytes.data()), bytes.size());
    return Status::Ok;
}

Status benchmark_identity(BenchmarkIdentity& identity) {
    if (cuInit(0) != CUDA_SUCCESS) return Status::CudaUnavailable;
    CUdevice device = 0;
    if (cuDeviceGet(&device, 0) != CUDA_SUCCESS) return Status::CudaUnavailable;
    char name[256]{};
    int major = 0, minor = 0, driver = 0;
    if (cuDeviceGetName(name, static_cast<int>(sizeof(name)), device) != CUDA_SUCCESS ||
        cuDeviceGetAttribute(&major, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR, device) != CUDA_SUCCESS ||
        cuDeviceGetAttribute(&minor, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR, device) != CUDA_SUCCESS ||
        cuDriverGetVersion(&driver) != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    identity.device = name;
    identity.compute_capability = std::to_string(major) + "." + std::to_string(minor);
    identity.driver_cuda_version = std::to_string(driver);
    identity.toolkit_header_version = std::to_string(CUDA_VERSION);
    auto status = benchmark_file_sha256("build/uor_inverted_file.ptx", identity.ptx_sha256);
    if (status != Status::Ok) return status;
    return benchmark_file_sha256("build/uor_inverted_file.cubin", identity.cubin_sha256);
}

Status build(const Table& table, Index& index) {
    if (table.tokens.empty()) return Status::InvalidArgument;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
    Buffer table_d, roots_d, offsets_d, members_d, cells_d, absent_d, status_d;
    if ((status = upload(table_values, table_d)) != Status::Ok ||
        (status = upload(root_values, roots_d)) != Status::Ok ||
        (status = allocate(offsets_d, 257 * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(members_d, count * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(cells_d, count * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(absent_d, sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(status_d, sizeof(std::uint32_t))) != Status::Ok) return status;
    std::vector<std::uint32_t> zeros(257, 0);
    if ((status = write(offsets_d, zeros.data())) != Status::Ok) return status;
    std::uint32_t absent = 0, kernel_status = 0;
    if ((status = write(absent_d, &absent)) != Status::Ok ||
        (status = write(status_d, &kernel_status)) != Status::Ok) return status;
    void* args[] = {&table_d.ptr, &roots_d.ptr, &count, &offsets_d.ptr, &members_d.ptr,
                    &cells_d.ptr, &absent_d.ptr, &status_d.ptr};
    if (cuLaunchKernel(session.build_fn, 1, 1, 1, 1, 1, 1, 0, nullptr, args, nullptr) != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    if ((status = synchronize()) != Status::Ok ||
        (status = read(&kernel_status, status_d)) != Status::Ok) return status;
    if (kernel_status != 0) return status_from_kernel(kernel_status);
    index.offsets.fill(0);
    index.cells.resize(count);
    index.members.resize(count);
    if ((status = read(index.offsets.data(), offsets_d)) != Status::Ok ||
        (status = read(index.members.data(), members_d)) != Status::Ok ||
        (status = read(index.cells.data(), cells_d)) != Status::Ok ||
        (status = read(&index.absent_count, absent_d)) != Status::Ok) return status;
    return validate(table, index);
}

Status update(const Table& table, std::uint32_t moved_token, Index& index) {
    if (moved_token >= table.tokens.size() || index.cells.size() != table.tokens.size()) {
        return Status::InvalidArgument;
    }
    if (validate_structure(index) != Status::Ok) return Status::CsrInvariant;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
    Buffer table_d, roots_d, offsets_d, members_d, cells_d, absent_d, status_d;
    if ((status = upload(table_values, table_d)) != Status::Ok ||
        (status = upload(root_values, roots_d)) != Status::Ok ||
        (status = upload(index.offsets, offsets_d)) != Status::Ok ||
        (status = upload_capacity(index.members, count, members_d)) != Status::Ok ||
        (status = upload(index.cells, cells_d)) != Status::Ok ||
        (status = allocate(absent_d, sizeof(std::uint32_t))) != Status::Ok ||
        (status = write(absent_d, &index.absent_count)) != Status::Ok ||
        (status = allocate(status_d, sizeof(std::uint32_t))) != Status::Ok) return status;
    std::uint32_t kernel_status = 0;
    if ((status = write(status_d, &kernel_status)) != Status::Ok) return status;
    void* args[] = {&table_d.ptr, &roots_d.ptr, &count, &moved_token, &offsets_d.ptr,
                    &members_d.ptr, &cells_d.ptr, &absent_d.ptr, &status_d.ptr};
    if (cuLaunchKernel(session.update_fn, 1, 1, 1, 1, 1, 1, 0, nullptr, args, nullptr) != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    if ((status = synchronize()) != Status::Ok ||
        (status = read(&kernel_status, status_d)) != Status::Ok) return status;
    if (kernel_status != 0) return status_from_kernel(kernel_status);
    if ((status = read(index.offsets.data(), offsets_d)) != Status::Ok) return status;
    if (index.offsets[256] > count) return Status::CsrInvariant;
    index.members.resize(count);
    if ((status = read(index.members.data(), members_d,
                       index.offsets[256] * sizeof(std::uint32_t))) != Status::Ok ||
        (status = read(index.cells.data(), cells_d)) != Status::Ok ||
        (status = read(&index.absent_count, absent_d)) != Status::Ok) return status;
    index.members.resize(index.offsets[256]);
    return validate(table, index);
}

Status run_scan(const Table& table, const Query& query, std::uint32_t k,
                const Index& index, ScanResult& result, bool full_scan) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    if (validate(table, index) != Status::Ok) return Status::CsrInvariant;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    const auto query_values = query_bytes(query);
    std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
    Buffer table_d, roots_d, queries_d, offsets_d, members_d, cells_d;
    Buffer predictions_d, admitted_d, top8_d, status_d;
    if ((status = upload(table_values, table_d)) != Status::Ok ||
        (status = upload(root_values, roots_d)) != Status::Ok ||
        (status = upload(query_values, queries_d)) != Status::Ok ||
        (status = upload(index.offsets, offsets_d)) != Status::Ok ||
        (status = upload(index.members, members_d)) != Status::Ok ||
        (status = upload(index.cells, cells_d)) != Status::Ok ||
        (status = allocate(predictions_d, sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(admitted_d, 16 * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(top8_d, 8 * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(status_d, sizeof(std::uint32_t))) != Status::Ok) return status;
    std::uint32_t kernel_status = 0, rows = 1;
    if ((status = write(status_d, &kernel_status)) != Status::Ok) return status;
    const auto function = full_scan ? session.full_scan_fn : session.probe_fn;
    const auto dynamic_shared = full_scan && count <= 4096 ? count * sizeof(std::int64_t) : 0;
    if (full_scan) {
        void* args[] = {&table_d.ptr, &roots_d.ptr, &count, &queries_d.ptr, &rows,
                        &offsets_d.ptr, &members_d.ptr, &cells_d.ptr, &k, &predictions_d.ptr,
                        &admitted_d.ptr, &top8_d.ptr, &status_d.ptr};
        if (cuLaunchKernel(function, rows, 1, 1, 256, 1, 1,
                           dynamic_shared, nullptr, args, nullptr) != CUDA_SUCCESS) {
            return Status::CudaFailure;
        }
    } else {
        void* args[] = {&table_d.ptr, &roots_d.ptr, &queries_d.ptr, &rows, &offsets_d.ptr,
                        &members_d.ptr, &cells_d.ptr, &k, &predictions_d.ptr, &admitted_d.ptr,
                        &top8_d.ptr, &status_d.ptr};
        if (cuLaunchKernel(function, rows, 1, 1, 1, 1, 1, 0, nullptr, args, nullptr) != CUDA_SUCCESS) {
            return Status::CudaFailure;
        }
    }
    if ((status = synchronize()) != Status::Ok ||
        (status = read(&kernel_status, status_d)) != Status::Ok) return status;
    if (kernel_status != 0) return status_from_kernel(kernel_status);
    if ((status = read(&result.prediction, predictions_d)) != Status::Ok ||
        (status = read(result.admitted.data(), admitted_d)) != Status::Ok ||
        (status = read(result.top8.data(), top8_d)) != Status::Ok) return status;
    return Status::Ok;
}

bool same_result(const ScanResult& a, const ScanResult& b) {
    return a.prediction == b.prediction && a.admitted == b.admitted && a.top8 == b.top8;
}

Status benchmark_kernel(const Table& table, const Query& query, std::uint32_t k,
                        const Index& index, bool full_scan, std::uint32_t warmup,
                        std::uint32_t repetitions, std::vector<double>& kernel_ms,
                        BenchmarkMeasurement& resources, ScanResult& result) {
    if (validate(table, index) != Status::Ok) return Status::CsrInvariant;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    const auto query_values = query_bytes(query);
    std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
    Buffer table_d, roots_d, queries_d, offsets_d, members_d, cells_d;
    Buffer predictions_d, admitted_d, top8_d, status_d;
    if ((status = upload(table_values, table_d)) != Status::Ok ||
        (status = upload(root_values, roots_d)) != Status::Ok ||
        (status = upload(query_values, queries_d)) != Status::Ok ||
        (status = upload(index.offsets, offsets_d)) != Status::Ok ||
        (status = upload(index.members, members_d)) != Status::Ok ||
        (status = upload(index.cells, cells_d)) != Status::Ok ||
        (status = allocate(predictions_d, sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(admitted_d, 16 * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(top8_d, 8 * sizeof(std::uint32_t))) != Status::Ok ||
        (status = allocate(status_d, sizeof(std::uint32_t))) != Status::Ok) return status;
    std::uint32_t kernel_status = 0, rows = 1;
    if ((status = write(status_d, &kernel_status)) != Status::Ok) return status;
    const auto function = full_scan ? session.full_scan_fn : session.probe_fn;
    const auto dynamic_shared = full_scan && count <= 4096 ? count * sizeof(std::int64_t) : 0;
    resources.dynamic_shared_bytes = static_cast<int>(dynamic_shared);
    if (cuFuncGetAttribute(&resources.registers, CU_FUNC_ATTRIBUTE_NUM_REGS, function) != CUDA_SUCCESS ||
        cuFuncGetAttribute(&resources.static_shared_bytes, CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES,
                           function) != CUDA_SUCCESS ||
        cuFuncGetAttribute(&resources.local_bytes, CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES,
                           function) != CUDA_SUCCESS ||
        cuOccupancyMaxActiveBlocksPerMultiprocessor(&resources.max_active_blocks_per_sm,
                                                     function, full_scan ? 256 : 1,
                                                     dynamic_shared) != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    for (std::uint32_t iteration = 0; iteration < warmup + repetitions; ++iteration) {
        CUevent start = nullptr, end = nullptr;
        if (cuEventCreate(&start, 0) != CUDA_SUCCESS || cuEventCreate(&end, 0) != CUDA_SUCCESS) {
            if (start != nullptr) cuEventDestroy(start);
            if (end != nullptr) cuEventDestroy(end);
            return Status::CudaFailure;
        }
        if (cuEventRecord(start, 0) != CUDA_SUCCESS) {
            cuEventDestroy(start);
            cuEventDestroy(end);
            return Status::CudaFailure;
        }
        CUresult launch_result = CUDA_SUCCESS;
        if (full_scan) {
            void* args[] = {&table_d.ptr, &roots_d.ptr, &count, &queries_d.ptr, &rows,
                            &offsets_d.ptr, &members_d.ptr, &cells_d.ptr, &k, &predictions_d.ptr,
                            &admitted_d.ptr, &top8_d.ptr, &status_d.ptr};
            launch_result = cuLaunchKernel(function, rows, 1, 1, 256, 1, 1,
                                           dynamic_shared, nullptr, args, nullptr);
        } else {
            void* args[] = {&table_d.ptr, &roots_d.ptr, &queries_d.ptr, &rows, &offsets_d.ptr,
                            &members_d.ptr, &cells_d.ptr, &k, &predictions_d.ptr, &admitted_d.ptr,
                            &top8_d.ptr, &status_d.ptr};
            launch_result = cuLaunchKernel(function, rows, 1, 1, 1, 1, 1, 0, nullptr, args, nullptr);
        }
        if (launch_result != CUDA_SUCCESS || cuEventRecord(end, 0) != CUDA_SUCCESS) {
            cuEventDestroy(start);
            cuEventDestroy(end);
            return Status::CudaFailure;
        }
        status = synchronize();
        float elapsed = 0;
        const auto elapsed_status = cuEventElapsedTime(&elapsed, start, end);
        cuEventDestroy(start);
        cuEventDestroy(end);
        if (status != Status::Ok) return status;
        if (elapsed_status != CUDA_SUCCESS) return Status::CudaFailure;
        if (iteration >= warmup) kernel_ms.push_back(elapsed);
    }
    if ((status = read(&kernel_status, status_d)) != Status::Ok) return status;
    if (kernel_status != 0) return status_from_kernel(kernel_status);
    if ((status = read(&result.prediction, predictions_d)) != Status::Ok ||
        (status = read(result.admitted.data(), admitted_d)) != Status::Ok ||
        (status = read(result.top8.data(), top8_d)) != Status::Ok) return status;
    return Status::Ok;
}

Status benchmark_scan(const Table& table, const Query& query, std::uint32_t k,
                      const Index& index, bool full_scan, std::uint32_t warmup,
                      std::uint32_t repetitions, std::vector<BenchmarkMeasurement>& measurements) {
    if (repetitions == 0 || k == 0 || k > 16) return Status::InvalidArgument;
    measurements.clear();
    ScanResult oracle;
    auto status = full_scan ? uor_inverted_file::full_scan(table, query, k, oracle)
                            : uor_inverted_file::probe(table, query, k, index, oracle);
    if (status != Status::Ok) return status;
    BenchmarkMeasurement resources;
    std::vector<double> kernel_ms;
    ScanResult gpu_result;
    status = benchmark_kernel(table, query, k, index, full_scan, warmup, repetitions,
                              kernel_ms, resources, gpu_result);
    if (status != Status::Ok) return status;
    if (!same_result(gpu_result, oracle) || kernel_ms.size() != repetitions) return Status::CudaFailure;
    for (std::uint32_t iteration = 0; iteration < warmup + repetitions; ++iteration) {
        const auto direct_start = std::chrono::steady_clock::now();
        ScanResult result;
        status = run_scan(table, query, k, index, result, full_scan);
        if (status != Status::Ok) return status;
        const auto direct_ms = std::chrono::duration<double, std::milli>(
            std::chrono::steady_clock::now() - direct_start).count();
        if (!same_result(result, oracle)) return Status::CudaFailure;
        if (iteration < warmup) continue;
        const auto end_start = std::chrono::steady_clock::now();
        if (full_scan) {
            status = uor_inverted_file::cuda::full_scan(table, query, k, result);
        } else {
            Index built;
            status = uor_inverted_file::build(table, built);
            if (status == Status::Ok) status = uor_inverted_file::cuda::probe(table, query, k, built, result);
        }
        if (status != Status::Ok) return status;
        if (!same_result(result, oracle)) return Status::CudaFailure;
        BenchmarkMeasurement measurement = resources;
        measurement.kernel_ms = kernel_ms[iteration - warmup];
        measurement.direct_ms = direct_ms;
        measurement.end_to_end_ms = std::chrono::duration<double, std::milli>(
            std::chrono::steady_clock::now() - end_start).count();
        measurement.oracle_match = true;
        measurement.oracle_prediction = oracle.prediction;
        measurements.push_back(measurement);
    }
    return Status::Ok;
}

Status probe(const Table& table, const Query& query, std::uint32_t k,
             const Index& index, ScanResult& result) {
    return run_scan(table, query, k, index, result, false);
}

Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                 ScanResult& result) {
    Index index;
    const auto status = build(table, index, true);
    if (status != Status::Ok) return status;
    return run_scan(table, query, k, index, result, true);
}

} // namespace uor_inverted_file::cuda
