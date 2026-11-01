#pragma once

#include <cstddef>
#include <string_view>

namespace test_json {

class Parser {
public:
    explicit Parser(std::string_view input) : input_(input) {}

    bool parse() {
        if (!value()) return false;
        whitespace();
        return position_ == input_.size();
    }

private:
    void whitespace() {
        while (position_ < input_.size()) {
            const char c = input_[position_];
            if (c != ' ' && c != '\t' && c != '\n' && c != '\r') break;
            ++position_;
        }
    }

    bool take(char expected) {
        if (position_ >= input_.size() || input_[position_] != expected) return false;
        ++position_;
        return true;
    }

    bool value() {
        whitespace();
        if (position_ >= input_.size()) return false;
        switch (input_[position_]) {
            case '{': return object();
            case '[': return array();
            case '"': return string();
            case 't': return literal("true");
            case 'f': return literal("false");
            case 'n': return literal("null");
            default: return number();
        }
    }

    bool object() {
        ++position_;
        whitespace();
        if (take('}')) return true;
        while (position_ < input_.size()) {
            whitespace();
            if (!string()) return false;
            whitespace();
            if (!take(':') || !value()) return false;
            whitespace();
            if (take('}')) return true;
            if (!take(',')) return false;
        }
        return false;
    }

    bool array() {
        ++position_;
        whitespace();
        if (take(']')) return true;
        while (position_ < input_.size()) {
            if (!value()) return false;
            whitespace();
            if (take(']')) return true;
            if (!take(',')) return false;
        }
        return false;
    }

    bool string() {
        if (!take('"')) return false;
        while (position_ < input_.size()) {
            const auto c = static_cast<unsigned char>(input_[position_++]);
            if (c == '"') return true;
            if (c < 0x20) return false;
            if (c == '\\') {
                if (position_ >= input_.size()) return false;
                const char escaped = input_[position_++];
                if (escaped == 'u') {
                    for (int i = 0; i < 4; ++i) {
                        if (position_ >= input_.size()) return false;
                        const char digit = input_[position_++];
                        if (!((digit >= '0' && digit <= '9') || (digit >= 'a' && digit <= 'f') ||
                              (digit >= 'A' && digit <= 'F'))) return false;
                    }
                } else if (escaped != '"' && escaped != '\\' && escaped != '/' && escaped != 'b' &&
                           escaped != 'f' && escaped != 'n' && escaped != 'r' && escaped != 't') {
                    return false;
                }
            } else if (c >= 0x80) {
                --position_;
                if (!utf8()) return false;
            }
        }
        return false;
    }

    bool utf8() {
        const auto first = static_cast<unsigned char>(input_[position_]);
        std::size_t length = 0;
        std::uint32_t codepoint = 0;
        std::uint32_t minimum = 0;
        if (first >= 0xc2 && first <= 0xdf) {
            length = 2;
            codepoint = first & 0x1f;
            minimum = 0x80;
        } else if (first >= 0xe0 && first <= 0xef) {
            length = 3;
            codepoint = first & 0x0f;
            minimum = 0x800;
        } else if (first >= 0xf0 && first <= 0xf4) {
            length = 4;
            codepoint = first & 0x07;
            minimum = 0x10000;
        } else {
            return false;
        }
        if (position_ + length > input_.size()) return false;
        ++position_;
        for (std::size_t i = 1; i < length; ++i) {
            const auto next = static_cast<unsigned char>(input_[position_++]);
            if ((next & 0xc0) != 0x80) return false;
            codepoint = (codepoint << 6) | (next & 0x3f);
        }
        return codepoint >= minimum && codepoint <= 0x10ffff &&
               !(codepoint >= 0xd800 && codepoint <= 0xdfff);
    }

    bool number() {
        if (take('-') && position_ >= input_.size()) return false;
        if (take('0')) {
            if (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') return false;
        } else {
            if (position_ >= input_.size() || input_[position_] < '1' || input_[position_] > '9') return false;
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') ++position_;
        }
        if (take('.')) {
            if (position_ >= input_.size() || input_[position_] < '0' || input_[position_] > '9') return false;
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') ++position_;
        }
        if (position_ < input_.size() && (input_[position_] == 'e' || input_[position_] == 'E')) {
            ++position_;
            if (position_ < input_.size() && (input_[position_] == '+' || input_[position_] == '-')) ++position_;
            if (position_ >= input_.size() || input_[position_] < '0' || input_[position_] > '9') return false;
            while (position_ < input_.size() && input_[position_] >= '0' && input_[position_] <= '9') ++position_;
        }
        return true;
    }

    bool literal(std::string_view text) {
        if (input_.substr(position_, text.size()) != text) return false;
        position_ += text.size();
        return true;
    }

    std::string_view input_;
    std::size_t position_ = 0;
};

inline bool valid(std::string_view input) {
    return Parser(input).parse();
}

} // namespace test_json
