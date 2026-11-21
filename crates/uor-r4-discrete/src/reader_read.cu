// The saved question reader's read, one thread a prompt: the asked turn's
// kept forms, then a greedy reply read from held rows found by binary search
// in the flat tables of conjunction/flat.rs. Integer compares, shifts and
// table reads only. Each step writes the 36 raw words the host replay
// compares (conjunction.rs step_words with the form's own fourteen words).
//
// Laid out for the read choices of the saved M2 readers alone: the pair
// stand, slots ranked precise, a place answered across its forms. A place no
// held row answers (the host's suffix ladder) and a prompt with no kept form
// are not read here: the thread stops and marks its status.
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

// The best-standing answer of the held forms' rows of one key; the earlier,
// more specific form keeps a tie (Asked::answer, `first` under across).
__device__ static Answer rows_answer(const u32* row_keys, const u32* row_slots, u32 row_count,
                                     const u32* held, u32 held_count, const u32* history,
                                     u32 tail, u32 empty, const Place& at, u32* missed) {
    Answer best;
    best.found = 0;
    u32 key[KEY_WORDS];
    key[1] = empty;
    key[2] = tail;
    for (u32 word = 0; word < 8u; ++word) {
        key[3u + word] = history[word];
    }
    for (u32 index = 0; index < held_count; ++index) {
        key[0] = held[index];
        u32 found = find(row_keys, row_count, KEY_WORDS, key);
        if (found == NONE) {
            continue;
        }
        Answer answer = row_answer(row_slots + (u64)found * ROW_WORDS, at, missed);
        if (!answer.found) {
            continue;
        }
        answer.index = index;
        bool beats = true;
        if (best.found) {
            u64 left = answer.right * best.spoke;
            u64 right = best.right * answer.spoke;
            beats = left != right ? left > right : best.is_token > answer.is_token;
        }
        if (beats) {
            best = answer;
        }
    }
    return best;
}

// params: vocabulary, context, bos, eos, forms, rows, codes, limit, stride, prompts.
// work: one row of `stride` ids a prompt, the prompt first; the reply is
// written after it. out_meta: reply ids, status, read again from the rows for
// an empty read, a read came up empty.
extern "C" __global__ void reader_read(const u32* params, const u32* forms, const u32* row_keys,
                                       const u32* row_slots, const u32* codes, u32* work,
                                       const u32* prompt_len, u32* out_words, u32* out_meta) {
    u32 prompt = blockIdx.x * blockDim.x + threadIdx.x;
    if (prompt >= params[9]) {
        return;
    }
    u32 context = params[1];
    u32 bos = params[2];
    u32 eos = params[3];
    u32 form_count = params[4];
    u32 row_count = params[5];
    u32 code_count = params[6];
    u32 limit = params[7];
    u32 stride = params[8];
    u32* tok = work + (u64)prompt * stride;
    u32 start = prompt_len[prompt];
    u32* words = out_words + (u64)prompt * limit * STEP_WORDS;
    u32* meta = out_meta + (u64)prompt * 4u;
    meta[0] = 0u;
    meta[1] = STATUS_NO_FORM;
    meta[2] = 0u;
    meta[3] = 0u;
    if (limit > MAX_LIMIT || start + limit > stride) {
        return;
    }

    // The asked turn: the ids after the latest EOS, at most thirty.
    u32 turn_floor = window_floor(tok, start, context, bos);
    u32 low = start > FORM_TOKENS ? start - FORM_TOKENS : 0u;
    if (low < turn_floor) {
        low = turn_floor;
    }
    u32 first = low;
    for (u32 place = start; place > low;) {
        --place;
        if (tok[place] == eos) {
            first = place + 1u;
            break;
        }
    }
    u32 turn_len = start - first;
    u32 held[FORMS_HELD];
    u32 held_count = held_forms(tok + first, turn_len, 0, 0u, forms, form_count, held);
    if (held_count == 0u && code_count > 0u) {
        bool coded = true;
        for (u32 place = first; place < start; ++place) {
            coded = coded && tok[place] < code_count;
        }
        if (coded) {
            held_count =
                held_forms(tok + first, turn_len, codes, CLASS_FORM, forms, form_count, held);
        }
    }
    if (held_count == 0u) {
        u32 long_first = turn_floor;
        for (u32 place = start; place > turn_floor;) {
            --place;
            if (tok[place] == eos) {
                long_first = place + 1u;
                break;
            }
        }
        if (long_first < start && tok[long_first] == bos) {
            ++long_first;
        }
        if (start - long_first > (u32)OPEN_HEAD_LEAST + FORM_GAP) {
            held_count = open_forms(tok + long_first, start - long_first, forms, form_count, held);
        }
    }
    if (held_count == 0u) {
        return;
    }

    u32 symbols[MAX_LIMIT];
    for (u32 pass = 0; pass < 2u; ++pass) {
        u32 missed = 0;
        u32 len = start;
        u32 cursor = NONE;
        u32 symbol_count = 0;
        u32 copied_from = NONE;
        while (len - start < limit) {
            Place at;
            at.tok = tok;
            at.len = len;
            at.floor = window_floor(tok, len, context, bos);
            at.start = start;
            at.cursor = cursor;
            bool has_ahead = cursor != NONE && cursor + 1u < start;
            u32 ahead = has_ahead ? tok[cursor + 1u] : NONE;
            bool copying = symbol_count > 0u && symbols[symbol_count - 1u] == COPIED;
            u32 own = AT_START;
            u32 shared = 0u;
            bool has_shared = false;
            if (copying && has_ahead) {
                own = ahead;
                shared = AFTER_COPY;
                has_shared = true;
            } else if (copying) {
                own = AFTER_COPY;
            }
            // The last sixteen symbols, thirteen bits each under a leading one.
            u32 history[8];
            for (u32 word = 0; word < 8u; ++word) {
                history[word] = (word % 4u == 3u) ? 1u : 0u;
            }
            u32 from = symbol_count > 16u ? symbol_count - 16u : 0u;
            for (u32 place = from; place < symbol_count; ++place) {
                push_bits(history + 4u * ((place - from) / 8u), 13u, symbols[place]);
            }

            Answer answer;
            answer.found = 0;
            u32 hit_own = 0;
            u32 hit_tail = 0;
            u32 hit_empty = 0;
            for (u32 rows = pass ? 1u : 0u;; --rows) {
                answer = rows_answer(row_keys, row_slots, row_count, held, held_count, history,
                                     own, rows, at, &missed);
                hit_own = 1u;
                hit_tail = own;
                hit_empty = rows;
                if (!answer.found && has_shared) {
                    answer = rows_answer(row_keys, row_slots, row_count, held, held_count,
                                         history, shared, rows, at, &missed);
                    hit_own = 0u;
                    hit_tail = shared;
                }
                if (answer.found || rows == 0u) {
                    break;
                }
            }
            if (!answer.found) {
                meta[0] = len - start;
                meta[1] = STATUS_LADDER;
                meta[2] = pass;
                meta[3] = missed;
                return;
            }

            u32* step = words + (u64)(len - start) * STEP_WORDS;
            const u32* form = forms + (u64)held[answer.index] * FORM_WORDS;
            step[0] = hit_own ? 0u : 1u;
            step[1] = answer.index;
            for (u32 word = 0; word < FORM_WORDS; ++word) {
                step[2u + word] = form[word];
            }
            step[16] = pass;
            step[17] = hit_empty;
            step[18] = hit_tail;
            for (u32 word = 0; word < 8u; ++word) {
                step[19u + word] = history[word];
            }
            step[27] = cursor;
            step[28] = ahead;
            for (u32 field = 0; field < 5u; ++field) {
                step[29u + field] = answer.entry[field];
            }
            step[34] = answer.named.source;
            step[35] = answer.named.token;

            // A run of copied ids is one symbol while each is read from the
            // place after the last.
            cursor = answer.named.source;
            if (cursor == NONE) {
                symbols[symbol_count++] = answer.named.token;
            } else {
                bool follows = copied_from != NONE && copied_from + 1u == cursor;
                if (!(copying && follows)) {
                    symbols[symbol_count++] = COPIED;
                }
            }
            copied_from = cursor;
            tok[len++] = answer.named.token;
            if (answer.named.token == eos) {
                break;
            }
        }
        meta[0] = len - start;
        meta[1] = STATUS_READ;
        meta[2] = pass;
        meta[3] = missed;
        if (!missed) {
            break;
        }
    }
}
