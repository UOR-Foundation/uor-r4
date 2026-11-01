#include "../include/uor_inverted_file.hpp"

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
constexpr uint32_t absent = std::numeric_limits<uint32_t>::max();
int failures = 0;

void expect(bool condition, const char* name) {
    if (!condition) {
        ++failures;
        std::cerr << "FAIL: " << name << '\n';
    }
}

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

void expect_same(const ScanResult& a, const ScanResult& b, const char* name) {
    expect(a.prediction == b.prediction, name);
    expect(a.admitted == b.admitted, name);
    expect(a.top8 == b.top8, name);
}

void parity(const Table& table, uint32_t k, const char* name) {
    Index index;
    expect(uor_inverted_file::build(table, index) == Status::Ok, name);
    expect(uor_inverted_file::validate(table, index) == Status::Ok, name);
    ScanResult bucket, full;
    expect(uor_inverted_file::probe(table, query_at(7), k, index, bucket) == Status::Ok, name);
    expect(uor_inverted_file::full_scan(table, query_at(7), k, full) == Status::Ok, name);
    expect_same(bucket, full, name);
}

} // namespace

int main() {
    for (const uint32_t v : {1u, 64u, 256u, 4096u}) {
        for (const uint32_t k : {1u, 8u, 16u}) parity(table_of(v), k, "seeded table parity");
    }

    Table single;
    for (uint32_t i = 0; i < 256; ++i) single.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    parity(single, 8, "single-cell collapse");

    Table few;
    for (uint32_t i = 0; i < 3; ++i) few.tokens.push_back(Token{roots[i], Block{}, Block{}, roots[i]});
    parity(few, 16, "fewer than K occupied cells");

    Index index;
    expect(uor_inverted_file::build(few, index) == Status::Ok, "few-cell build");
    ScanResult result;
    expect(uor_inverted_file::probe(few, query_at(0), 0, index, result) == Status::InvalidArgument,
           "zero K refused");
    expect(uor_inverted_file::probe(few, query_at(0), 17, index, result) == Status::InvalidArgument,
           "K above 16 refused");

    Table invalid = few;
    invalid.tokens[0][0] = Block{};
    Index refused;
    expect(uor_inverted_file::build(invalid, refused) == Status::NonRootBlock,
           "non-root endpoint refused");

    Table moving = table_of(256);
    Index updated;
    expect(uor_inverted_file::build(moving, updated) == Status::Ok, "initial update-index build");
    for (uint32_t step = 0; step < 64; ++step) {
        const uint32_t moved = (step * 17) % 256;
        moving.tokens[moved][0] = roots[(step + 3) % 16];
        moving.tokens[moved][3] = roots[(step * 5 + 1) % 16];
        expect(uor_inverted_file::update(moving, moved, updated) == Status::Ok, "64 incremental updates");
        expect(uor_inverted_file::validate(moving, updated) == Status::Ok, "update CSR invariant");
        Index fresh;
        expect(uor_inverted_file::build(moving, fresh) == Status::Ok, "fresh build after update");
        expect(updated.offsets == fresh.offsets, "update offsets equal fresh build");
        expect(updated.members == fresh.members, "update members equal fresh build");
        expect(updated.cells == fresh.cells, "update cells equal fresh build");
        expect(updated.absent_count == fresh.absent_count, "update absent count equal fresh build");
        ScanResult a, b;
        expect(uor_inverted_file::probe(moving, query_at(step), 8, updated, a) == Status::Ok, "updated probe");
        expect(uor_inverted_file::full_scan(moving, query_at(step), 8, b) == Status::Ok, "updated full scan");
        expect_same(a, b, "updated parity");
    }

    Table removal = few;
    Index removal_index;
    expect(uor_inverted_file::build(removal, removal_index) == Status::Ok, "removal initial build");
    removal.tokens[0][0] = Block{};
    expect(uor_inverted_file::update(removal, 0, removal_index) == Status::Ok, "cell to ABSENT update");
    expect(removal_index.absent_count == 1, "ABSENT count after removal");
    expect(uor_inverted_file::validate(removal, removal_index) == Status::Ok, "removal CSR invariant");
    removal.tokens[0][0] = roots[0];
    expect(uor_inverted_file::update(removal, 0, removal_index) == Status::Ok, "ABSENT to cell update");
    expect(removal_index.absent_count == 0, "ABSENT count after reinsertion");
    expect(uor_inverted_file::validate(removal, removal_index) == Status::Ok, "reinsertion CSR invariant");

    Table ties;
    ties.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    ties.tokens.push_back(Token{roots[0], Block{}, Block{}, roots[0]});
    parity(ties, 1, "token-id tie break");
    Index tie_index;
    expect(uor_inverted_file::build(ties, tie_index) == Status::Ok, "tie build");
    ScanResult tie_result;
    expect(uor_inverted_file::probe(ties, query_at(0), 1, tie_index, tie_result) == Status::Ok, "tie probe");
    expect(tie_result.prediction == 0, "lowest token id wins tie");

    Index stale = tie_index;
    stale.cells[0] = 7;
    expect(uor_inverted_file::validate(ties, stale) == Status::CsrInvariant, "stale bucket detected");

    Index corrupt = tie_index;
    corrupt.offsets[1] = 99;
    expect(uor_inverted_file::validate(ties, corrupt) == Status::CsrInvariant, "corrupted CSR detected");

    Table over = few;
    Index over_index;
    expect(uor_inverted_file::build(over, over_index) == Status::Ok, "over-prune fixture build");
    ScanResult bucket, full;
    expect(uor_inverted_file::probe(over, query_at(0), 1, over_index, bucket) == Status::Ok, "over-prune probe");
    expect(uor_inverted_file::full_scan(over, query_at(0), 1, full) == Status::Ok, "over-prune full scan");
    bucket.admitted[0] = absent;
    expect(bucket.prediction != full.prediction || bucket.admitted != full.admitted || bucket.top8 != full.top8,
           "over-pruned result detected");

    if (failures != 0) {
        std::cerr << failures << " inverted-file assertions failed\n";
        return 1;
    }
    std::cout << "inverted-file tests: all parity, update, refusal, and fault checks passed\n";
    return 0;
}
