#pragma once

#include "../include/uor_inverted_file.hpp"

#include <string>

namespace uor_inverted_file::cuda {

struct BenchmarkIdentity {
    std::string device;
    std::string compute_capability;
    std::string driver_cuda_version;
    std::string toolkit_header_version;
    std::string ptx_sha256;
    std::string cubin_sha256;
};

[[nodiscard]] Status benchmark_identity(BenchmarkIdentity& identity);
[[nodiscard]] Status benchmark_file_sha256(const std::string& path, std::string& digest);

struct BenchmarkMeasurement {
    double kernel_ms = 0;
    double direct_ms = 0;
    double end_to_end_ms = 0;
    int registers = 0;
    int static_shared_bytes = 0;
    int dynamic_shared_bytes = 0;
    int local_bytes = 0;
    int max_active_blocks_per_sm = 0;
    bool oracle_match = false;
    std::uint32_t oracle_prediction = ABSENT;
};

[[nodiscard]] Status benchmark_scan(const Table& table, const Query& query, std::uint32_t k,
                                    const Index& index, bool full_scan, std::uint32_t warmup,
                                    std::uint32_t repetitions,
                                    std::vector<BenchmarkMeasurement>& measurements);

[[nodiscard]] Status build(const Table& table, Index& index);
[[nodiscard]] Status update(const Table& table, std::uint32_t moved_token, Index& index);
[[nodiscard]] Status probe(const Table& table, const Query& query, std::uint32_t k,
                           const Index& index, ScanResult& result);
[[nodiscard]] Status full_scan(const Table& table, const Query& query, std::uint32_t k,
                               ScanResult& result);

} // namespace uor_inverted_file::cuda
