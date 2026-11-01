#include "sha256.hpp"

#include <array>
#include <iomanip>
#include <sstream>
#include <vector>

namespace uor_inverted_file {
namespace {

constexpr std::array<std::uint32_t, 64> K{{
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
}};

std::uint32_t rotr(std::uint32_t x, unsigned n) { return (x >> n) | (x << (32 - n)); }

} // namespace

std::string sha256_hex(const std::uint8_t* data, std::size_t length) {
    std::vector<std::uint8_t> bytes;
    if (length != 0) bytes.assign(data, data + length);
    bytes.push_back(0x80);
    while (bytes.size() % 64 != 56) bytes.push_back(0);
    const std::uint64_t bits = static_cast<std::uint64_t>(length) * 8;
    for (int shift = 56; shift >= 0; shift -= 8) bytes.push_back(static_cast<std::uint8_t>(bits >> shift));

    std::array<std::uint32_t, 8> hash{
        0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
    for (std::size_t offset = 0; offset < bytes.size(); offset += 64) {
        std::array<std::uint32_t, 64> w{};
        for (std::size_t i = 0; i < 16; ++i) {
            const auto j = offset + i * 4;
            w[i] = (std::uint32_t(bytes[j]) << 24) | (std::uint32_t(bytes[j + 1]) << 16) |
                   (std::uint32_t(bytes[j + 2]) << 8) | bytes[j + 3];
        }
        for (std::size_t i = 16; i < 64; ++i) {
            const auto s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            const auto s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        auto a = hash[0], b = hash[1], c = hash[2], d = hash[3];
        auto e = hash[4], f = hash[5], g = hash[6], h = hash[7];
        for (std::size_t i = 0; i < 64; ++i) {
            const auto s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            const auto choice = (e & f) ^ (~e & g);
            const auto temp1 = h + s1 + choice + K[i] + w[i];
            const auto s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            const auto majority = (a & b) ^ (a & c) ^ (b & c);
            const auto temp2 = s0 + majority;
            h = g; g = f; f = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        for (std::size_t i = 0; i < 8; ++i) hash[i] += std::array<std::uint32_t, 8>{a,b,c,d,e,f,g,h}[i];
    }
    std::ostringstream output;
    output << std::hex << std::setfill('0');
    for (const auto value : hash) output << std::setw(8) << value;
    return output.str();
}

} // namespace uor_inverted_file
