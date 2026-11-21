// Helpers shared by the reader kernels (reader_read.cu, reader_train.cu): the
// asked turn's forms, what a slot names at a place, slot ranking and one
// row's answer. Integer compares, shifts and table reads only.
#pragma once
typedef unsigned int u32;
typedef unsigned long long u64;

#define NONE 0xFFFFFFFFu
#define AT_START 0xFFFFFFFFu
#define AFTER_COPY 0xFFFFFFFEu
#define COPIED 4096u
#define FORM_WORDS 14u
#define KEY_WORDS 11u
#define SLOT_WORDS 7u
#define MAX_SLOTS 8u
#define ROW_WORDS 56u
#define STEP_WORDS 36u
#define MAX_LIMIT 128u
#define FORMS_HELD 6u
#define FORM_TOKENS 30u
#define FORM_GAP 12u
#define ROW_RIGHT 2u
#define CLASS_FORM 0x80u
#define OPEN_FORM 0x40u
#define OPEN_HEAD 16
#define OPEN_TAIL 8
#define OPEN_HEAD_LEAST 4

#define STATUS_READ 0u
#define STATUS_LADDER 1u
#define STATUS_NO_FORM 2u

struct Place {
    const u32* tok;
    u32 len;
    u32 floor;
    u32 start;
    u32 cursor;
};

struct Named {
    u32 ok;
    u32 token;
    u32 source;
};

struct Answer {
    u32 found;
    Named named;
    u32 index;
    u32 entry[5];
    u64 right;
    u64 spoke;
    u32 is_token;
};

// A 128-bit word as four u32, most significant first: shift in `bits` bits.
__device__ static void push_bits(u32* w, u32 bits, u32 value) {
    w[0] = (w[0] << bits) | (w[1] >> (32u - bits));
    w[1] = (w[1] << bits) | (w[2] >> (32u - bits));
    w[2] = (w[2] << bits) | (w[3] >> (32u - bits));
    w[3] = (w[3] << bits) | value;
}

__device__ static u32 find(const u32* table, u32 count, u32 width, const u32* key) {
    u32 low = 0;
    u32 high = count;
    while (low < high) {
        u32 middle = low + (high - low) / 2u;
        const u32* row = table + (u64)middle * width;
        int order = 0;
        for (u32 word = 0; word < width; ++word) {
            if (row[word] != key[word]) {
                order = row[word] < key[word] ? -1 : 1;
                break;
            }
        }
        if (order < 0) {
            low = middle + 1u;
        } else if (order > 0) {
            high = middle;
        } else {
            return middle;
        }
    }
    return NONE;
}

// The first place the reads may see for a target at `position` (window_at).
__device__ static u32 window_floor(const u32* tok, u32 position, u32 context, u32 bos) {
    u32 low = position > context ? position - context : 0u;
    u32 start = position;
    while (start > low) {
        --start;
        if (tok[start] == bos) {
            break;
        }
    }
    return start;
}

// The form that keeps turn[..head] and turn[resume..len] (Form::of, Form::open).
__device__ static void form_key(const u32* turn, u32 head, u32 resume, u32 len, const u32* codes,
                                u32 gap_at, u32 gap_len, u32* key) {
    for (u32 word = 0; word < 12u; ++word) {
        key[word] = (word % 4u == 3u) ? 1u : 0u;
    }
    u32 place = 0;
    for (u32 at = 0; at < len; ++at) {
        if (at >= head && at < resume) {
            continue;
        }
        u32 token = codes ? codes[turn[at]] : turn[at];
        push_bits(key + 4u * (place / 10u), 12u, token);
        ++place;
    }
    key[12] = gap_at;
    key[13] = gap_len;
}

// Kept forms of the turn, the whole turn first, then shorter gaps before longer.
__device__ static u32 held_forms(const u32* turn, u32 len, const u32* codes, u32 mark,
                                 const u32* forms, u32 form_count, u32* held) {
    u32 key[FORM_WORDS];
    u32 count = 0;
    form_key(turn, 0u, 0u, len, codes, 0u, mark, key);
    u32 id = find(forms, form_count, FORM_WORDS, key);
    if (id != NONE) {
        held[count++] = id;
    }
    u32 longest = len < FORM_GAP ? len : FORM_GAP;
    for (u32 gap = 1; gap <= longest && count < FORMS_HELD; ++gap) {
        for (u32 at = 0; at + gap <= len && count < FORMS_HELD; ++at) {
            form_key(turn, at, at + gap, len, codes, at, gap | mark, key);
            id = find(forms, form_count, FORM_WORDS, key);
            if (id != NONE) {
                held[count++] = id;
            }
        }
    }
    return count;
}

// Kept open forms of a long turn, those that keep most first, the longer
// opening on a tie (Form::each_open).
__device__ static u32 open_forms(const u32* turn, u32 len, const u32* forms, u32 form_count,
                                 u32* held) {
    u32 key[FORM_WORDS];
    u32 count = 0;
    for (int kept = OPEN_HEAD + OPEN_TAIL; kept >= OPEN_HEAD_LEAST && count < FORMS_HELD; --kept) {
        for (int head = kept < OPEN_HEAD ? kept : OPEN_HEAD;
             head >= OPEN_HEAD_LEAST && count < FORMS_HELD; --head) {
            int tail = kept - head;
            if (tail > OPEN_TAIL) {
                break;
            }
            if ((u32)kept + FORM_GAP >= len) {
                continue;
            }
            form_key(turn, (u32)head, len - (u32)tail, len, 0, (u32)head, OPEN_FORM | (u32)tail,
                     key);
            u32 id = find(forms, form_count, FORM_WORDS, key);
            if (id != NONE) {
                held[count++] = id;
            }
        }
    }
    return count;
}

// What a slot names at a place (Entry::name, pointer_source under the pair stand).
__device__ static Named name(const u32* entry, const Place& at) {
    Named named;
    named.ok = 0;
    named.token = 0;
    named.source = NONE;
    const u32* tok = at.tok;
    u32 kind = entry[0];
    if (kind == 1u) {
        named.ok = 1;
        named.token = entry[4];
        return named;
    }
    u32 source = NONE;
    if (kind == 2u) {
        u32 back = entry[1];
        u32 ahead = entry[2];
        u32 wide = entry[3];
        u32 before = entry[4];
        if (at.start < 1u + back) {
            return named;
        }
        u32 anchor = at.start - 1u - back;
        u32 low = at.floor + wide;
        if (anchor < low) {
            return named;
        }
        if (ahead == 0u) {
            source = anchor;
        } else {
            u32 key = tok[anchor];
            u32 stand = NONE;
            for (u32 place = anchor; place > low;) {
                --place;
                if (place + ahead >= anchor) {
                    continue;
                }
                if (wide && tok[place - 1u] != tok[anchor - 1u]) {
                    continue;
                }
                if (tok[place] == key && tok[place + ahead - 1u] == before) {
                    stand = place;
                    break;
                }
            }
            if (stand == NONE) {
                return named;
            }
            source = stand + ahead;
            if (source >= anchor) {
                return named;
            }
        }
    } else if (kind == 3u) {
        if (at.cursor == NONE) {
            return named;
        }
        source = at.cursor + 1u;
        if (source >= at.start) {
            return named;
        }
    } else if (kind == 4u) {
        if (at.len == 0u) {
            return named;
        }
        u32 last = at.len - 1u;
        u32 stand = NONE;
        for (u32 place = last; place > at.floor + 1u;) {
            --place;
            if (tok[place - 1u] == tok[last - 1u] && tok[place] == tok[last]) {
                stand = place;
                break;
            }
        }
        if (stand == NONE) {
            return named;
        }
        source = stand + 1u;
    } else {
        return named;
    }
    named.ok = 1;
    named.token = tok[source];
    named.source = source;
    return named;
}

// Slot a ranks before slot b: right more often of the times it spoke (one
// doubt added), a read before a token, then times right (HeldRow::ranked).
__device__ static bool ranks_before(const u32* row, u32 a, u32 b) {
    u64 right_a = row[a * SLOT_WORDS + 5u];
    u64 right_b = row[b * SLOT_WORDS + 5u];
    u64 spoke_a = right_a + row[a * SLOT_WORDS + 6u] + 1u;
    u64 spoke_b = right_b + row[b * SLOT_WORDS + 6u] + 1u;
    u64 left = right_b * spoke_a;
    u64 right = right_a * spoke_b;
    if (left != right) {
        return left < right;
    }
    u32 token_a = row[a * SLOT_WORDS] == 1u;
    u32 token_b = row[b * SLOT_WORDS] == 1u;
    if (token_a != token_b) {
        return token_a < token_b;
    }
    return right_b < right_a;
}

// Live slots in rank order, the lower slot on a tie.
__device__ static u32 rank(const u32* row, u32* order) {
    u32 count = 0;
    for (u32 slot = 0; slot < MAX_SLOTS; ++slot) {
        if (row[slot * SLOT_WORDS + 5u] == 0u) {
            continue;
        }
        u32 at = count;
        while (at > 0u && ranks_before(row, slot, order[at - 1u])) {
            order[at] = order[at - 1u];
            --at;
        }
        order[at] = slot;
        ++count;
    }
    return count;
}

// One row's answer and whether its best slot is a read that names nothing
// (HeldRow::best over HeldRow::read, HeldRow::empty_read).
__device__ static Answer row_answer(const u32* row, const Place& at, u32* missed) {
    Answer answer;
    answer.found = 0;
    u32 order[MAX_SLOTS];
    u32 live = rank(row, order);
    u32 by = NONE;
    bool first = true;
    for (u32 turn = 0; turn < live; ++turn) {
        u32 slot = order[turn];
        const u32* entry = row + slot * SLOT_WORDS;
        if (entry[5] < ROW_RIGHT) {
            continue;
        }
        Named named = name(entry, at);
        if (first && (entry[0] == 2u || entry[0] == 4u) && !named.ok) {
            *missed = 1u;
        }
        first = false;
        if (named.ok) {
            answer.named = named;
            by = slot;
            break;
        }
    }
    if (by == NONE) {
        return answer;
    }
    // The place is the one the first ranked copying slot read this token from.
    u32 source = NONE;
    for (u32 turn = 0; turn < live; ++turn) {
        const u32* entry = row + order[turn] * SLOT_WORDS;
        if (entry[5] < ROW_RIGHT) {
            continue;
        }
        Named named = name(entry, at);
        if (named.ok && named.token == answer.named.token && named.source != NONE) {
            source = named.source;
            break;
        }
    }
    answer.named.source = source;
    const u32* named_by = row + by * SLOT_WORDS;
    for (u32 field = 0; field < 5u; ++field) {
        answer.entry[field] = named_by[field];
    }
    // The record is that of the first slot holding the same entry.
    u32 record = by;
    for (u32 slot = 0; slot < MAX_SLOTS; ++slot) {
        const u32* entry = row + slot * SLOT_WORDS;
        if (entry[5] == 0u) {
            continue;
        }
        bool same = true;
        for (u32 field = 0; field < 5u; ++field) {
            same = same && entry[field] == named_by[field];
        }
        if (same) {
            record = slot;
            break;
        }
    }
    answer.right = row[record * SLOT_WORDS + 5u];
    answer.spoke = answer.right + row[record * SLOT_WORDS + 6u] + 1u;
    answer.is_token = named_by[0] == 1u;
    answer.found = 1;
    return answer;
}
