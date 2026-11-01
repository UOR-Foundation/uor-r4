#include "../src/instrumented.hpp"

#include <cstdlib>
#include <iostream>

int main() {
    using uor_inverted_file::Block;
    using uor_inverted_file::Index;
    using uor_inverted_file::Instrumented;
    using uor_inverted_file::Query;
    using uor_inverted_file::ScanResult;
    using uor_inverted_file::Status;
    using uor_inverted_file::codebook;
    using uor_inverted_file::Table;
    using uor_inverted_file::Token;

    const char* output = std::getenv("UOR_OTEL_OUT");
    Instrumented instrumented(output ? std::filesystem::path(output) : std::filesystem::path{});
    Table table;
    for (std::uint32_t i = 0; i < 4; ++i) {
        const auto& first = codebook[i];
        const auto& last = codebook[(i * 5 + 1) % codebook.size()];
        table.tokens.push_back(Token{first, Block{}, Block{}, last});
    }
    Index index;
    const auto build_status = instrumented.build(table, index);
    if (build_status != Status::Ok) {
        std::cerr << "build failed: " << static_cast<int>(build_status) << '\n';
        return 1;
    }
    table.tokens[1][3] = codebook[2];
    if (instrumented.update(table, 1, index) != Status::Ok) {
        std::cerr << "update failed\n";
        return 1;
    }
    const Query query{codebook[0], Block{}, Block{}, codebook[1]};
    ScanResult bucket, full;
    const auto probe_status = instrumented.probe(table, query, 2, index, bucket);
    const auto full_status = instrumented.full_scan(table, query, 2, full);
    if (probe_status != Status::Ok || full_status != Status::Ok || bucket.prediction != full.prediction) {
        std::cerr << "scan parity failed\n";
        return 1;
    }
    std::cout << "instrumented build/probe/full-scan parity: prediction=" << bucket.prediction << '\n';
    if (instrumented.enabled()) {
        std::cout << "OTLP JSON written to " << output << '\n';
    } else {
        std::cout << "telemetry disabled: no file or endpoint created\n";
    }
    return 0;
}
