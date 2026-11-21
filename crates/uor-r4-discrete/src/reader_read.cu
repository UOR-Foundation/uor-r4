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
#include "reader_common.cuh"

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
