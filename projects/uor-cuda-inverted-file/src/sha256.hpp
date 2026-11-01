#pragma once

#include <cstddef>
#include <cstdint>
#include <string>

namespace uor_inverted_file {

[[nodiscard]] std::string sha256_hex(const std::uint8_t* data, std::size_t length);

} // namespace uor_inverted_file
