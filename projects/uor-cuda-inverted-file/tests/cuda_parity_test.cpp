#include "../include/uor_inverted_file.hpp"
#include "../src/cuda_driver.hpp"

#include <array>
#include <cstdint>
#include <iostream>
#include <limits>
#include <vector>

namespace {

using uor_inverted_file::Block;
using uor_inverted_file::Index;
using uor_inverted_file::Query;
using uor_inverted_file::ScanResult;
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

Token token_at(uint32_t i) {
    return Token{roots[(i * 7 + 1) % 16], Block{}, Block{}, roots[(i * 11 + 3) % 16]};
}

Table table_of(uint32_t count) {
    Table table;
    for (uint32_t i = 0; i < count; ++i) table.tokens.push_back(token_at(i));
    return table;
}

Query query_at(uint32_t seed) {
    return Query{roots[seed % 16], Block{}, Block{}, roots[(seed * 5 + 1) % 16]};
}

bool same(const ScanResult& a, const ScanResult& b) {
    return a.prediction == b.prediction && a.admitted == b.admitted && a.top8 == b.top8;
}

} // namespace

int main() {
    for (const uint32_t v : {64u, 256u}) {
        for (const uint32_t k : {1u, 8u, 16u}) {
            Table table = table_of(v);
            Index index;
            auto status = uor_inverted_file::cuda::build(table, index);
            if (status == Status::CudaUnavailable) {
                std::cout << "CudaUnavailable: CUDA parity tests were not run\n";
                return 2;
            }
            if (status != Status::Ok) {
                std::cerr << "CUDA build failed: " << static_cast<int>(status) << '\n';
                return 1;
            }
            for (uint32_t step = 0; step < 64; ++step) {
                const uint32_t moved = (step * 17) % v;
                table.tokens[moved][0] = roots[(step + 3) % 16];
                table.tokens[moved][3] = roots[(step * 5 + 1) % 16];
                status = uor_inverted_file::cuda::update(table, moved, index);
                if (status != Status::Ok) {
                    std::cerr << "CUDA update failed: " << static_cast<int>(status) << '\n';
                    return 1;
                }
                Index fresh;
                if (uor_inverted_file::build(table, fresh) != Status::Ok ||
                    index.offsets != fresh.offsets || index.members != fresh.members ||
                    index.cells != fresh.cells || index.absent_count != fresh.absent_count) {
                    std::cerr << "CUDA update differs from fresh build\n";
                    return 1;
                }
            }
            for (uint32_t seed = 0; seed < 4; ++seed) {
                const auto query = query_at(seed);
                ScanResult gpu_bucket, gpu_full, cpu;
                if (uor_inverted_file::cuda::probe(table, query, k, index, gpu_bucket) != Status::Ok ||
                    uor_inverted_file::cuda::full_scan(table, query, k, gpu_full) != Status::Ok ||
                    uor_inverted_file::full_scan(table, query, k, cpu) != Status::Ok) {
                    std::cerr << "CUDA scan failed\n";
                    return 1;
                }
                if (!same(gpu_bucket, gpu_full) || !same(gpu_bucket, cpu) ||
                    gpu_bucket.prediction >= v || gpu_full.prediction >= v) {
                    std::cerr << "CUDA/CPU parity mismatch\n";
                    return 1;
                }
            }

            table.tokens[0][0] = Block{};
            if (uor_inverted_file::cuda::update(table, 0, index) != Status::NonRootBlock) {
                std::cerr << "non-root CUDA update was not refused\n";
                return 1;
            }
            if (uor_inverted_file::remove_token(table, 0, index) != Status::Ok) {
                std::cerr << "explicit removal setup failed\n";
                return 1;
            }
            table.tokens[0][0] = roots[0];
            if (uor_inverted_file::cuda::update(table, 0, index) != Status::Ok) {
                std::cerr << "CUDA ABSENT-to-cell update failed\n";
                return 1;
            }
            auto extreme_query = query_at(0);
            for (size_t block = 1; block < 3; ++block) extreme_query[block].fill(std::numeric_limits<int32_t>::min());
            for (auto& token : table.tokens) {
                for (size_t block = 1; block < 3; ++block) token[block].fill(std::numeric_limits<int32_t>::max());
            }
            ScanResult gpu_bucket, gpu_full, cpu;
            if (uor_inverted_file::cuda::probe(table, extreme_query, 1, index, gpu_bucket) != Status::Ok ||
                uor_inverted_file::cuda::full_scan(table, extreme_query, 1, gpu_full) != Status::Ok ||
                uor_inverted_file::full_scan(table, extreme_query, 1, cpu) != Status::Ok ||
                !same(gpu_bucket, gpu_full) || !same(gpu_bucket, cpu)) {
                std::cerr << "extreme-value CUDA/CPU parity mismatch\n";
                return 1;
            }
        }
    }
    std::cout << "CUDA parity tests: all build/update/probe/full-scan checks passed\n";
    return 0;
}
