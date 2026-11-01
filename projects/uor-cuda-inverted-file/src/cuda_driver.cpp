#include "cuda_driver.hpp"

#include <cuda.h>

#include <cstddef>
#include <cstdint>
#include <vector>

namespace uor_inverted_file::cuda {
namespace {

struct Buffer {
    CUdeviceptr ptr = 0;
    std::size_t bytes = 0;

    Buffer() = default;
    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;
    ~Buffer() { if (ptr != 0) cuMemFree(ptr); }
};

Status allocate(Buffer& buffer, std::size_t bytes) {
    if (bytes == 0) return Status::InvalidArgument;
    buffer.bytes = bytes;
    return cuMemAlloc(&buffer.ptr, bytes) == CUDA_SUCCESS ? Status::Ok : Status::CudaFailure;
}

Status write(const Buffer& buffer, const void* source) {
    return cuMemcpyHtoD(buffer.ptr, source, buffer.bytes) == CUDA_SUCCESS
        ? Status::Ok : Status::CudaFailure;
}

Status read(void* destination, const Buffer& buffer) {
    return cuMemcpyDtoH(destination, buffer.ptr, buffer.bytes) == CUDA_SUCCESS
        ? Status::Ok : Status::CudaFailure;
}

template <typename T>
Status upload(const std::vector<T>& source, Buffer& buffer) {
    const auto status = allocate(buffer, source.size() * sizeof(T));
    if (status != Status::Ok) return status;
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
    if (cuModuleLoadFatBinary(&session.module, "build/uor_inverted_file.cubin") != CUDA_SUCCESS) {
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

Status build(const Table& table, Index& index) {
    if (table.tokens.empty()) return Status::InvalidArgument;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    const std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
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
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    const std::uint32_t count = static_cast<std::uint32_t>(table.tokens.size());
    Buffer table_d, roots_d, offsets_d, members_d, cells_d, absent_d, status_d;
    if ((status = upload(table_values, table_d)) != Status::Ok ||
        (status = upload(root_values, roots_d)) != Status::Ok ||
        (status = upload(index.offsets, offsets_d)) != Status::Ok ||
        (status = upload(index.members, members_d)) != Status::Ok ||
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
    if ((status = read(index.offsets.data(), offsets_d)) != Status::Ok ||
        (status = read(index.members.data(), members_d)) != Status::Ok ||
        (status = read(index.cells.data(), cells_d)) != Status::Ok ||
        (status = read(&index.absent_count, absent_d)) != Status::Ok) return status;
    return validate(table, index);
}

Status run_scan(const Table& table, const Query& query, std::uint32_t k,
                const Index& index, ScanResult& result, bool full_scan) {
    if (k == 0 || k > 16) return Status::InvalidArgument;
    Session session;
    auto status = open(session);
    if (status != Status::Ok) return status;
    const auto table_values = table_bytes(table);
    const auto root_values = roots_bytes();
    const auto query_values = query_bytes(query);
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
    void* args[] = {&table_d.ptr, &roots_d.ptr, &queries_d.ptr, &rows, &offsets_d.ptr,
                    &members_d.ptr, &cells_d.ptr, &k, &predictions_d.ptr, &admitted_d.ptr,
                    &top8_d.ptr, &status_d.ptr};
    const auto function = full_scan ? session.full_scan_fn : session.probe_fn;
    if (cuLaunchKernel(function, 1, 1, 1, 1, 1, 1, 0, nullptr, args, nullptr) != CUDA_SUCCESS) {
        return Status::CudaFailure;
    }
    if ((status = synchronize()) != Status::Ok ||
        (status = read(&kernel_status, status_d)) != Status::Ok) return status;
    if (kernel_status != 0) return status_from_kernel(kernel_status);
    if ((status = read(&result.prediction, predictions_d)) != Status::Ok ||
        (status = read(result.admitted.data(), admitted_d)) != Status::Ok ||
        (status = read(result.top8.data(), top8_d)) != Status::Ok) return status;
    return Status::Ok;
}

Status probe(const Table& table, const Query& query, std::uint32_t k,
             const Index& index, ScanResult& result) {
    return run_scan(table, query, k, index, result, false);
}

Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                 ScanResult& result) {
    return run_scan(table, query, k, index, result, true);
}

} // namespace uor_inverted_file::cuda
