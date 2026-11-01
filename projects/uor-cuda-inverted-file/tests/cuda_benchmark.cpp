#include "../src/cuda_driver.hpp"

#include <array>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>

namespace {

using uor_inverted_file::Block;
using uor_inverted_file::Index;
using uor_inverted_file::Query;
using uor_inverted_file::Status;
using uor_inverted_file::Table;
using uor_inverted_file::Token;

const std::array<Block, 16> roots{{
    {2, 2, 0, 0, 0, 0, 0, 0}, {-2, -2, 0, 0, 0, 0, 0, 0},
    {-2, 2, 0, 0, 0, 0, 0, 0}, {0, 0, -2, 0, -2, 0, 0, 0},
    {0, 0, -2, 0, 2, 0, 0, 0}, {0, 0, 0, -2, 0, 0, 0, -2},
    {0, 0, 0, -2, 0, 0, 0, 2}, {0, 0, 0, 0, 0, -2, -2, 0},
    {0, 0, 0, 0, 0, -2, 2, 0}, {0, 0, 0, 0, 0, 2, -2, 0},
    {0, 0, 0, 0, 0, 2, 2, 0}, {0, 0, 0, 2, 0, 0, 0, -2},
    {0, 0, 0, 2, 0, 0, 0, 2}, {0, 0, 2, 0, -2, 0, 0, 0},
    {0, 0, 2, 0, 2, 0, 0, 0}, {2, -2, 0, 0, 0, 0, 0, 0},
}};

Token token_at(std::uint32_t i) {
    Token token{roots[(i * 7 + 1) % 16], Block{}, Block{}, roots[(i * 11 + 3) % 16]};
    for (std::uint32_t d = 0; d < 8; ++d) {
        token[1][d] = static_cast<std::int32_t>((i * 29 + d * 7) % 37) - 18;
        token[2][d] = static_cast<std::int32_t>((i * 13 + d * 5) % 41) - 20;
    }
    return token;
}

Table table_of(std::uint32_t count) {
    Table table;
    for (std::uint32_t i = 0; i < count; ++i) table.tokens.push_back(token_at(i));
    return table;
}

Query query_at(std::uint32_t seed) {
    Query query{roots[seed % 16], Block{}, Block{}, roots[(seed * 5 + 1) % 16]};
    for (std::uint32_t d = 0; d < 8; ++d) {
        query[1][d] = static_cast<std::int32_t>((seed * 11 + d * 3) % 23) - 11;
        query[2][d] = static_cast<std::int32_t>((seed * 17 + d * 2) % 19) - 9;
    }
    return query;
}

bool valid_source(const std::string& source) {
    if (source.size() != 40) return false;
    for (const char c : source) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) return false;
    }
    return true;
}

bool valid_run_id(const std::string& run_id) {
    if (run_id.empty()) return false;
    for (const char c : run_id) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || c == '-' || c == '_')) return false;
    }
    return true;
}

bool same_index(const Index& a, const Index& b) {
    return a.offsets == b.offsets && a.members == b.members && a.cells == b.cells &&
           a.absent_count == b.absent_count;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 13 || std::string(argv[1]) != "--out" || std::string(argv[3]) != "--run-id" ||
        std::string(argv[5]) != "--source" || std::string(argv[7]) != "--correctness" ||
        std::string(argv[9]) != "--gpu-info" || std::string(argv[11]) != "--nvcc-info" ||
        !valid_source(argv[6]) || !valid_run_id(argv[4])) {
        std::cerr << "usage: cuda_benchmark --out PATH --run-id ID --source 40_HEX_COMMIT "
                     "--correctness TSV --gpu-info LOG --nvcc-info LOG\n";
        return 2;
    }
    const std::string output_path = argv[2];
    const std::string run_id = argv[4];
    const std::string source = argv[6];
    std::string correctness_sha256, gpu_info_sha256, nvcc_info_sha256;
    if (uor_inverted_file::cuda::benchmark_file_sha256(argv[8], correctness_sha256) != Status::Ok ||
        uor_inverted_file::cuda::benchmark_file_sha256(argv[10], gpu_info_sha256) != Status::Ok ||
        uor_inverted_file::cuda::benchmark_file_sha256(argv[12], nvcc_info_sha256) != Status::Ok) {
        std::cerr << "cannot hash correctness or hardware identity files\n";
        return 2;
    }
    uor_inverted_file::cuda::BenchmarkIdentity identity;
    const auto identity_status = uor_inverted_file::cuda::benchmark_identity(identity);
    if (identity_status != Status::Ok) {
        std::cerr << "benchmark identity failed: " << static_cast<int>(identity_status) << '\n';
        return 1;
    }
    std::ofstream output(output_path, std::ios::trunc);
    if (!output) {
        std::cerr << "cannot open benchmark output\n";
        return 2;
    }
    output << std::setprecision(9);
    std::uint32_t completed = 0;
    for (const std::uint32_t v : {64u, 256u, 1024u, 4096u}) {
        Table table = table_of(v);
        Index index;
        const auto build_status = uor_inverted_file::cuda::build(table, index);
        if (build_status == Status::CudaUnavailable) {
            std::cout << "CudaUnavailable: benchmark was not run\n";
            return 2;
        }
        if (build_status != Status::Ok) {
            std::cerr << "benchmark build failed: " << static_cast<int>(build_status) << '\n';
            return 1;
        }
        Index cpu_index;
        const auto cpu_build_status = uor_inverted_file::build(table, cpu_index);
        if (cpu_build_status != Status::Ok || !same_index(index, cpu_index)) {
            std::cerr << "GPU/CPU index mismatch at V=" << v << '\n';
            return 1;
        }
        for (const std::uint32_t k : {1u, 8u, 16u}) {
            for (const bool full_scan : {false, true}) {
                const auto query = query_at(7);
                std::vector<uor_inverted_file::cuda::BenchmarkMeasurement> measurements;
                const auto status = uor_inverted_file::cuda::benchmark_scan(
                    table, query, k, index, full_scan, 3, 10, measurements);
                if (status != Status::Ok) {
                    std::cerr << "benchmark scan failed: " << static_cast<int>(status) << '\n';
                    return 1;
                }
                if (measurements.size() != 10) {
                    std::cerr << "incomplete benchmark measurement set\n";
                    return 1;
                }
                for (std::uint32_t repetition = 0; repetition < measurements.size(); ++repetition) {
                    const auto& m = measurements[repetition];
                    output << "{\"schema\":\"uor.cuda.inverted-file-measurement/1\",\"status\":\"ok\",\"v\":"
                           << v << ",\"k\":" << k << ",\"query_seed\":7,\"operation\":\""
                           << (full_scan ? "full_scan" : "probe") << "\",\"repetition\":" << repetition
                           << ",\"warmup\":3,\"kernel_ms\":" << m.kernel_ms
                           << ",\"direct_ms\":" << m.direct_ms
                           << ",\"end_to_end_ms\":" << m.end_to_end_ms
                           << ",\"registers\":" << m.registers
                           << ",\"static_shared_bytes\":" << m.static_shared_bytes
                           << ",\"dynamic_shared_bytes\":" << m.dynamic_shared_bytes
                           << ",\"local_bytes\":" << m.local_bytes
                           << ",\"max_active_blocks_per_sm\":" << m.max_active_blocks_per_sm
                           << ",\"oracle_match\":" << (m.oracle_match ? "true" : "false")
                           << ",\"oracle_prediction\":" << m.oracle_prediction
                           << ",\"index_match\":true}\n";
                    ++completed;
                }
            }
        }
    }
    output.close();
    if (!output) {
        std::cerr << "failed to write benchmark output\n";
        return 1;
    }
    std::string raw_sha256;
    const auto hash_status = uor_inverted_file::cuda::benchmark_file_sha256(output_path, raw_sha256);
    if (hash_status != Status::Ok) {
        std::cerr << "benchmark output hash failed\n";
        return 1;
    }
    std::ofstream manifest(output_path + ".manifest.json", std::ios::trunc);
    manifest << "{\"schema\":\"uor.cuda.inverted-file-benchmark-manifest/1\",\"run_id\":\"" << run_id
             << "\",\"source\":\"" << source << "\",\"device\":\"" << identity.device
             << "\",\"compute_capability\":\"" << identity.compute_capability
             << "\",\"driver_cuda_version\":\"" << identity.driver_cuda_version
             << "\",\"toolkit_header_version\":\"" << identity.toolkit_header_version
             << "\",\"gpu_info_sha256\":\"" << gpu_info_sha256
             << "\",\"nvcc_info_sha256\":\"" << nvcc_info_sha256
             << "\",\"correctness_tsv_sha256\":\"" << correctness_sha256
             << "\",\"ptx_sha256\":\"" << identity.ptx_sha256
             << "\",\"cubin_sha256\":\"" << identity.cubin_sha256
             << "\",\"raw_jsonl_sha256\":\"" << raw_sha256
             << "\",\"records\":" << completed << ",\"oracle_match\":true,\"index_match\":true}\n";
    manifest.close();
    if (!manifest) {
        std::cerr << "failed to write benchmark manifest\n";
        return 1;
    }
    std::cout << "CUDA benchmark: " << completed << " measurements with three timing scopes\n";
    return 0;
}
