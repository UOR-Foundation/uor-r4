#include "../include/uor_inverted_file.hpp"
#include "../src/cuda_driver.hpp"

#include <array>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <string>
#include <vector>

namespace {

using uor_inverted_file::ABSENT;
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
    Token token{roots[(i * 7 + 1) % 16], Block{}, Block{}, roots[(i * 11 + 3) % 16]};
    for (uint32_t d = 0; d < 8; ++d) {
        token[1][d] = static_cast<int32_t>((i * 29 + d * 7) % 37) - 18;
        token[2][d] = static_cast<int32_t>((i * 13 + d * 5) % 41) - 20;
    }
    return token;
}

Table table_of(uint32_t count) {
    Table table;
    for (uint32_t i = 0; i < count; ++i) table.tokens.push_back(token_at(i));
    return table;
}

Query query_at(uint32_t seed) {
    Query query{roots[seed % 16], Block{}, Block{}, roots[(seed * 5 + 1) % 16]};
    for (uint32_t d = 0; d < 8; ++d) {
        query[1][d] = static_cast<int32_t>((seed * 11 + d * 3) % 23) - 11;
        query[2][d] = static_cast<int32_t>((seed * 17 + d * 2) % 19) - 9;
    }
    return query;
}

bool same(const ScanResult& a, const ScanResult& b) {
    return a.prediction == b.prediction && a.admitted == b.admitted && a.top8 == b.top8;
}

uint32_t root_id(const Block& block) {
    for (uint32_t root = 0; root < roots.size(); ++root) if (block == roots[root]) return root;
    return ABSENT;
}

uint32_t cell_of(const Token& token) {
    const auto first = root_id(token[0]);
    const auto last = root_id(token[3]);
    return first == ABSENT || last == ABSENT ? ABSENT : first * 16 + last;
}

int64_t distance(const Query& query, const Token& token) {
    int64_t total = 0;
    for (size_t block = 0; block < query.size(); ++block) {
        for (size_t d = 0; d < query[block].size(); ++d) {
            const auto delta = static_cast<int64_t>(query[block][d]) - token[block][d];
            total += delta < 0 ? -delta : delta;
        }
    }
    return total;
}

bool valid_result(const Table& table, const Query& query, const ScanResult& result) {
    if (result.prediction >= table.tokens.size() || result.prediction != result.top8[0]) return false;
    std::array<bool, 256> seen{};
    std::array<bool, 256> occupied{};
    for (const auto& token : table.tokens) {
        const auto cell = cell_of(token);
        if (cell != ABSENT) occupied[cell] = true;
    }
    for (const auto cell : result.admitted) {
        if (cell == ABSENT) continue;
        if (cell >= 256 || !occupied[cell] || seen[cell]) return false;
        seen[cell] = true;
    }
    int64_t previous_distance = -1;
    uint32_t previous_token = 0;
    for (const auto token : result.top8) {
        if (token == ABSENT) continue;
        if (token >= table.tokens.size()) return false;
        const auto current = distance(query, table.tokens[token]);
        if (current < previous_distance || (current == previous_distance && token <= previous_token)) {
            return false;
        }
        previous_distance = current;
        previous_token = token;
    }
    return true;
}

std::ofstream rows;

void log_row(uint32_t v, uint32_t k, const char* op, uint32_t step, Status status, bool valid,
             const ScanResult* result = nullptr) {
    if (!rows.is_open()) return;
    rows << v << '\t' << k << '\t' << op << '\t' << step << '\t' << static_cast<int>(status) << '\t'
         << valid;
    if (result == nullptr) {
        rows << "\tNA\tNA\tNA\n";
        return;
    }
    rows << '\t' << result->prediction << '\t';
    for (const auto cell : result->admitted) rows << cell << ',';
    rows << '\t';
    for (const auto token : result->top8) rows << token << ',';
    rows << '\n';
}

bool log_result(uint32_t v, uint32_t k, const char* op, uint32_t step, Status status,
                const ScanResult& result, const Table& table, const Query& query,
                const ScanResult* comparison = nullptr) {
    const bool valid = status == Status::Ok && valid_result(table, query, result) &&
                       (comparison == nullptr || same(result, *comparison));
    log_row(v, k, op, step, status, valid, &result);
    return valid;
}

} // namespace

int main(int argc, char** argv) {
    if (argc == 3 && std::string(argv[1]) == "--out") {
        rows.open(argv[2], std::ios::trunc);
        if (!rows) {
            std::cerr << "cannot open row output\n";
            return 2;
        }
    } else if (argc != 1) {
        std::cerr << "usage: cuda_parity_test [--out PATH]\n";
        return 2;
    }

    for (const uint32_t v : {64u, 256u}) {
        for (const uint32_t k : {1u, 8u, 16u}) {
            Table table = table_of(v);
            Index index;
            auto status = uor_inverted_file::cuda::build(table, index);
            log_row(v, k, "build", 0, status, status == Status::Ok);
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
                Index fresh;
                const auto fresh_status = uor_inverted_file::build(table, fresh);
                const bool equal = fresh_status == Status::Ok && index.offsets == fresh.offsets &&
                                   index.members == fresh.members && index.cells == fresh.cells &&
                                   index.absent_count == fresh.absent_count;
                log_row(v, k, "update", step, status, status == Status::Ok && equal);
                if (status != Status::Ok || !equal) {
                    std::cerr << "CUDA update differs from fresh build\n";
                    return 1;
                }
            }
            for (uint32_t seed = 0; seed < 4; ++seed) {
                const auto query = query_at(seed);
                ScanResult gpu_bucket, gpu_full, cpu;
                const auto bucket_status = uor_inverted_file::cuda::probe(table, query, k, index, gpu_bucket);
                const auto full_status = uor_inverted_file::cuda::full_scan(table, query, k, gpu_full);
                const auto cpu_status = uor_inverted_file::full_scan(table, query, k, cpu);
                const bool bucket_ok = log_result(v, k, "probe_gpu", seed, bucket_status, gpu_bucket,
                                                  table, query, &cpu);
                const bool full_ok = log_result(v, k, "full_scan_gpu", seed, full_status, gpu_full,
                                                table, query, &cpu);
                const bool cpu_ok = log_result(v, k, "oracle_cpu", seed, cpu_status, cpu, table, query);
                if (!bucket_ok || !full_ok || !cpu_ok) {
                    std::cerr << "CUDA/CPU parity mismatch\n";
                    return 1;
                }
            }

            table.tokens[0][0] = Block{};
            status = uor_inverted_file::cuda::update(table, 0, index);
            log_row(v, k, "refuse_nonroot", 0, status, status == Status::NonRootBlock);
            if (status != Status::NonRootBlock) {
                std::cerr << "non-root CUDA update was not refused\n";
                return 1;
            }
            status = uor_inverted_file::remove_token(table, 0, index);
            log_row(v, k, "remove_cpu", 0, status, status == Status::Ok);
            if (status != Status::Ok) {
                std::cerr << "explicit removal setup failed\n";
                return 1;
            }
            table.tokens[0][0] = roots[0];
            status = uor_inverted_file::cuda::update(table, 0, index);
            Index reinsert_fresh;
            const auto fresh_status = uor_inverted_file::build(table, reinsert_fresh);
            const bool reinsert_exact = fresh_status == Status::Ok &&
                                        index.offsets == reinsert_fresh.offsets &&
                                        index.members == reinsert_fresh.members &&
                                        index.cells == reinsert_fresh.cells &&
                                        index.absent_count == reinsert_fresh.absent_count;
            log_row(v, k, "reinsert_gpu", 0, status, status == Status::Ok && reinsert_exact);
            if (status != Status::Ok || !reinsert_exact) {
                std::cerr << "CUDA ABSENT-to-cell update failed\n";
                return 1;
            }

            Index corrupt = index;
            corrupt.offsets[1] = corrupt.offsets[256];
            status = uor_inverted_file::cuda::update(table, 0, corrupt);
            log_row(v, k, "refuse_corrupt_update", 0, status, status == Status::CsrInvariant);
            if (status != Status::CsrInvariant) {
                std::cerr << "corrupt CUDA update was not refused\n";
                return 1;
            }

            auto extreme_query = query_at(0);
            for (size_t block = 1; block < 3; ++block) extreme_query[block].fill(std::numeric_limits<int32_t>::min());
            for (auto& token : table.tokens) {
                for (size_t block = 1; block < 3; ++block) token[block].fill(std::numeric_limits<int32_t>::max());
            }
            ScanResult gpu_bucket, gpu_full, cpu;
            const auto bucket_status = uor_inverted_file::cuda::probe(table, extreme_query, 1, index, gpu_bucket);
            const auto full_status = uor_inverted_file::cuda::full_scan(table, extreme_query, 1, gpu_full);
            const auto cpu_status = uor_inverted_file::full_scan(table, extreme_query, 1, cpu);
            const bool bucket_ok = log_result(v, k, "extreme_probe", 0, bucket_status, gpu_bucket,
                                              table, extreme_query, &cpu);
            const bool full_ok = log_result(v, k, "extreme_full_scan", 0, full_status, gpu_full,
                                            table, extreme_query, &cpu);
            const bool cpu_ok = log_result(v, k, "extreme_oracle", 0, cpu_status, cpu, table, extreme_query);
            if (!bucket_ok || !full_ok || !cpu_ok) {
                std::cerr << "extreme-value CUDA/CPU parity mismatch\n";
                return 1;
            }
        }
    }
    rows.close();
    if (!rows.is_open() && argc == 3 && rows.fail()) {
        std::cerr << "failed to write row output\n";
        return 1;
    }
    std::cout << "CUDA parity tests: all build/update/probe/full-scan checks passed\n";
    return 0;
}
