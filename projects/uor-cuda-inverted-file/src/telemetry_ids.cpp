#include "telemetry_ids.hpp"

#include <array>
#include <chrono>
#include <cstdint>
#include <random>

namespace uor_inverted_file {
namespace {

std::uint64_t next_random() {
    thread_local static std::uint64_t state = [] {
        const auto now = std::chrono::high_resolution_clock::now().time_since_epoch().count();
        const auto device = std::random_device{}();
        return (now ^ (static_cast<std::uint64_t>(device) << 32)) + 0x9e3779b97f4a7c15ULL;
    }();
    state += 0x9e3779b97f4a7c15ULL;
    auto z = state;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

template <std::size_t N>
std::string identifier() {
    static constexpr char hex[] = "0123456789abcdef";
    std::string output;
    output.reserve(N * 2);
    bool nonzero = false;
    for (std::size_t i = 0; i < N; ++i) {
        const auto value = static_cast<unsigned>(next_random() & 0xff);
        if (value != 0) nonzero = true;
        output.push_back(hex[value >> 4]);
        output.push_back(hex[value & 15]);
    }
    if (!nonzero) output[0] = '1';
    return output;
}

} // namespace

std::string new_trace_id() { return identifier<16>(); }
std::string new_span_id() { return identifier<8>(); }

} // namespace uor_inverted_file
