// The saved question reader's TRAIN (conjunction/saved.rs Reader::train) as
// CUDA kernels: the pair rows, the count of every question form, the kept
// forms in key order, the held rows of every TRAIN reply and the rows for
// replies whose own read came up empty. Integer compares, shifts, counts and
// table reads only.
//
// Held rows are written in the host's order wherever the order can matter.
// Two replies can meet in a row only when they hold a form in common (a
// row's key starts with its form), so the host puts replies in levels: a
// reply's level is one past the highest level of an earlier reply that holds
// one of its forms. One launch writes one level, a thread a reply.
//
// Laid out for the choices of the saved M2 readers alone: no class codes, no
// suffix orders above the pair, the pair stand, slots ranked precise, a
// place answered across its forms.
#include "reader_common.cuh"

#define FORM_SEEN 8u
#define POINTER_BACK 16u
#define POINTER_AHEAD 4u
#define POINTERS_ADDED 3u
#define SLOT_EMPTY 0u
#define SLOT_LOCKED 1u
#define SLOT_READY 2u
#define PAIR_WORDS 18u
#define JOB_WORDS 9u
#define RUN_WORDS 14u
#define HISTORY 16u

// cfg: vocabulary, context, bos, eos, form slots, row slots, limit, stride,
// ladder slots, kept forms the buffer holds, count open forms.
#define CFG_CONTEXT 1
#define CFG_BOS 2
#define CFG_EOS 3
#define CFG_FORM_SLOTS 4
#define CFG_ROW_SLOTS 5
#define CFG_LIMIT 6
#define CFG_STRIDE 7
#define CFG_LADDER_SLOTS 8
#define CFG_KEPT 9
#define CFG_OPEN 10

// flags: the form table is full, the row table is full, more forms were kept
// than the buffer holds, a prompt is longer than its work row.
#define FLAG_FORMS 0
#define FLAG_ROWS 1
#define FLAG_KEPT 2
#define FLAG_WORK 3

__device__ static u32 hash_words(const u32* key, u32 width) {
    u32 hash = 2166136261u;
    for (u32 word = 0; word < width; ++word) {
        hash = (hash ^ key[word]) * 16777619u;
    }
    hash ^= hash >> 15;
    hash *= 0x2c1b3c6du;
    hash ^= hash >> 12;
    return hash;
}

// The slot of `key` in an open-addressed table of `capacity` (a power of
// two) slots; with `insert` a missing key claims the first empty slot. A
// claimed slot is locked while its key is written. NONE: the key is absent,
// or the table is full.
__device__ static u32 slot_of(u32* state, u32* keys, u32 capacity, u32 width, const u32* key,
                              bool insert) {
    u32 mask = capacity - 1u;
    u32 slot = hash_words(key, width) & mask;
    for (u32 probe = 0; probe < capacity; ++probe) {
        u32 held = ((volatile u32*)state)[slot];
        if (held == SLOT_EMPTY) {
            if (!insert) {
                return NONE;
            }
            held = atomicCAS(state + slot, SLOT_EMPTY, SLOT_LOCKED);
            if (held == SLOT_EMPTY) {
                u32* at = keys + (u64)slot * width;
                for (u32 word = 0; word < width; ++word) {
                    at[word] = key[word];
                }
                __threadfence();
                atomicExch(state + slot, SLOT_READY);
                return slot;
            }
        }
        while (held == SLOT_LOCKED) {
            held = ((volatile u32*)state)[slot];
        }
        // The key is read only after the slot was seen ready, and as volatile
        // loads: a plain load may be moved ahead of the wait above, and then
        // compares against a key another thread is still writing.
        const volatile u32* at = keys + (u64)slot * width;
        bool same = true;
        for (u32 word = 0; word < width; ++word) {
            u32 theirs = at[word];
            same = same && theirs == key[word];
        }
        if (same) {
            return slot;
        }
        slot = (slot + 1u) & mask;
    }
    return NONE;
}

// The asked turn before a reply at `start`: the ids after the latest EOS, at
// most thirty (Asked::turn). Returns its first place.
__device__ static u32 turn_first(const u32* tok, u32 start, u32 context, u32 bos, u32 eos) {
    u32 floor = window_floor(tok, start, context, bos);
    u32 low = start > FORM_TOKENS ? start - FORM_TOKENS : 0u;
    if (low < floor) {
        low = floor;
    }
    for (u32 place = start; place > low;) {
        --place;
        if (tok[place] == eos) {
            return place + 1u;
        }
    }
    return low;
}

// The whole asked turn when more of it is left out than a gap holds
// (Asked::long_turn). Returns its first place, NONE for a short turn.
__device__ static u32 long_first(const u32* tok, u32 start, u32 context, u32 bos, u32 eos) {
    u32 floor = window_floor(tok, start, context, bos);
    u32 first = floor;
    for (u32 place = start; place > floor;) {
        --place;
        if (tok[place] == eos) {
            first = place + 1u;
            break;
        }
    }
    if (first < start && tok[first] == bos) {
        ++first;
    }
    return start - first > (u32)OPEN_HEAD_LEAST + FORM_GAP ? first : NONE;
}

__device__ static void count_form(const u32* cfg, u32* state, u32* keys, u32* counts, u32* flags,
                                  const u32* key) {
    u32 slot = slot_of(state, keys, cfg[CFG_FORM_SLOTS], FORM_WORDS, key, true);
    if (slot == NONE) {
        flags[FLAG_FORMS] = 1u;
        return;
    }
    atomicAdd(counts + slot, 1u);
}

// First pass (Asked::count): a thread a reply counts every form of its turn.
extern "C" __global__ void train_count(const u32* cfg, const u32* tok, const u32* run_start,
                                       u32* state, u32* keys, u32* counts, u32* flags, u32 runs) {
    u32 run = blockIdx.x * blockDim.x + threadIdx.x;
    if (run >= runs) {
        return;
    }
    u32 start = run_start[run];
    u32 first = turn_first(tok, start, cfg[CFG_CONTEXT], cfg[CFG_BOS], cfg[CFG_EOS]);
    const u32* turn = tok + first;
    u32 len = start - first;
    u32 key[FORM_WORDS];
    form_key(turn, 0u, 0u, len, 0, 0u, 0u, key);
    count_form(cfg, state, keys, counts, flags, key);
    u32 longest = len < FORM_GAP ? len : FORM_GAP;
    for (u32 gap = 1; gap <= longest; ++gap) {
        for (u32 at = 0; at + gap <= len; ++at) {
            form_key(turn, at, at + gap, len, 0, at, gap, key);
            count_form(cfg, state, keys, counts, flags, key);
        }
    }
    if (!cfg[CFG_OPEN]) {
        return;
    }
    u32 whole = long_first(tok, start, cfg[CFG_CONTEXT], cfg[CFG_BOS], cfg[CFG_EOS]);
    if (whole == NONE) {
        return;
    }
    u32 all = start - whole;
    for (u32 head = OPEN_HEAD_LEAST; head <= (u32)OPEN_HEAD; ++head) {
        for (u32 tail = 0; tail <= (u32)OPEN_TAIL; ++tail) {
            if (head + tail + FORM_GAP < all) {
                form_key(tok + whole, head, all - tail, all, 0, head, OPEN_FORM | tail, key);
                count_form(cfg, state, keys, counts, flags, key);
            }
        }
    }
}

// The forms seen at least eight times, in no order (Asked::seal, first half).
extern "C" __global__ void train_keep(const u32* cfg, const u32* state, const u32* keys,
                                      const u32* counts, u32* kept, u32* kept_count, u32* flags) {
    u32 slot = blockIdx.x * blockDim.x + threadIdx.x;
    if (slot >= cfg[CFG_FORM_SLOTS] || state[slot] != SLOT_READY || counts[slot] < FORM_SEEN) {
        return;
    }
    u32 at = atomicAdd(kept_count, 1u);
    if (at >= cfg[CFG_KEPT]) {
        flags[FLAG_KEPT] = 1u;
        return;
    }
    for (u32 word = 0; word < FORM_WORDS; ++word) {
        kept[(u64)at * FORM_WORDS + word] = keys[(u64)slot * FORM_WORDS + word];
    }
}

// Each kept form's id is the number of kept forms below it (Asked::seal,
// second half): the forms land in key order.
extern "C" __global__ void train_rank(const u32* kept, u32* forms, u32 kept_count) {
    u32 mine = blockIdx.x * blockDim.x + threadIdx.x;
    if (mine >= kept_count) {
        return;
    }
    const u32* key = kept + (u64)mine * FORM_WORDS;
    u32 below = 0;
    for (u32 other = 0; other < kept_count; ++other) {
        const u32* row = kept + (u64)other * FORM_WORDS;
        for (u32 word = 0; word < FORM_WORDS; ++word) {
            if (row[word] != key[word]) {
                below += row[word] < key[word] ? 1u : 0u;
                break;
            }
        }
    }
    for (u32 word = 0; word < FORM_WORDS; ++word) {
        forms[(u64)below * FORM_WORDS + word] = key[word];
    }
}

// The kept forms a reply is written under: its token forms (count, six ids)
// and its open forms (count, six ids). Asked::held, Asked::held_open.
extern "C" __global__ void train_forms(const u32* cfg, const u32* tok, const u32* run_start,
                                       const u32* forms, u32* out, u32 form_count, u32 runs) {
    u32 run = blockIdx.x * blockDim.x + threadIdx.x;
    if (run >= runs) {
        return;
    }
    u32 start = run_start[run];
    u32* mine = out + (u64)run * RUN_WORDS;
    u32 first = turn_first(tok, start, cfg[CFG_CONTEXT], cfg[CFG_BOS], cfg[CFG_EOS]);
    mine[0] = held_forms(tok + first, start - first, 0, 0u, forms, form_count, mine + 1);
    u32 whole = long_first(tok, start, cfg[CFG_CONTEXT], cfg[CFG_BOS], cfg[CFG_EOS]);
    mine[7] = whole == NONE ? 0u
                            : open_forms(tok + whole, start - whole, forms, form_count, mine + 8);
}

struct Rows {
    u32* state;
    u32* keys;
    u32* slots;
    u32 capacity;
};

// The reply so far as a held row sees it (History): the last sixteen
// symbols, a run of copied ids as one, and the place the last id was copied
// from.
struct Hist {
    u32 symbol[HISTORY];
    u32 count;
    u32 last;
};

__device__ static bool hist_copying(const Hist& hist) {
    return hist.count > 0u && hist.symbol[hist.count - 1u] == COPIED;
}

__device__ static void hist_add(Hist& hist, u32 symbol) {
    if (hist.count == HISTORY) {
        for (u32 place = 1; place < HISTORY; ++place) {
            hist.symbol[place - 1u] = hist.symbol[place];
        }
        hist.count = HISTORY - 1u;
    }
    hist.symbol[hist.count++] = symbol;
}

__device__ static void hist_push(Hist& hist, u32 token, u32 source) {
    if (source == NONE) {
        hist_add(hist, token);
    } else {
        bool follows = hist.last != NONE && hist.last + 1u == source;
        if (!(hist_copying(hist) && follows)) {
            hist_add(hist, COPIED);
        }
    }
    hist.last = source;
}

// Thirteen bits a symbol under a leading one, eight symbols a word.
__device__ static void hist_address(const Hist& hist, u32* history) {
    for (u32 word = 0; word < 8u; ++word) {
        history[word] = (word % 4u == 3u) ? 1u : 0u;
    }
    for (u32 place = 0; place < hist.count; ++place) {
        push_bits(history + 4u * (place / 8u), 13u, hist.symbol[place]);
    }
}

__device__ static void row_key(u32 form, u32 empty, u32 tail, const u32* history, u32* key) {
    key[0] = form;
    key[1] = empty;
    key[2] = tail;
    for (u32 word = 0; word < 8u; ++word) {
        key[3u + word] = history[word];
    }
}

// The best-standing answer of the held forms' rows of one key; the earlier
// form keeps a tie (Asked::answer, `first` under across).
__device__ static Answer held_answer(const Rows& rows, const u32* held, u32 held_count,
                                     const u32* history, u32 tail, u32 empty, const Place& at,
                                     u32* missed) {
    Answer best;
    best.found = 0;
    u32 key[KEY_WORDS];
    for (u32 index = 0; index < held_count; ++index) {
        row_key(held[index], empty, tail, history, key);
        u32 found = slot_of(rows.state, rows.keys, rows.capacity, KEY_WORDS, key, false);
        if (found == NONE) {
            continue;
        }
        Answer answer = row_answer(rows.slots + (u64)found * ROW_WORDS, at, missed);
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

// The held answer at a place: the rows for an empty read first when asked
// for, each time the place's own row and then the row every copied id shares.
__device__ static Answer place_answer(const Rows& rows, const u32* held, u32 held_count,
                                      const u32* history, u32 own, u32 shared, bool has_shared,
                                      u32 empty, const Place& at, u32* missed) {
    Answer answer;
    answer.found = 0;
    for (u32 pass = empty ? 1u : 0u;; --pass) {
        answer = held_answer(rows, held, held_count, history, own, pass, at, missed);
        if (!answer.found && has_shared) {
            answer = held_answer(rows, held, held_count, history, shared, pass, at, missed);
        }
        if (answer.found || pass == 0u) {
            break;
        }
    }
    return answer;
}

// A new slot: the first unused one; with none unused the weakest (the
// highest slot on a tie) loses a count and is taken when it reaches zero
// (HeldRow::insert).
__device__ static void row_insert(u32* row, const u32* entry) {
    u32 at = NONE;
    for (u32 slot = 0; slot < MAX_SLOTS; ++slot) {
        if (row[slot * SLOT_WORDS + 5u] == 0u) {
            at = slot;
            break;
        }
    }
    if (at == NONE) {
        u32 weakest = MAX_SLOTS - 1u;
        for (u32 slot = MAX_SLOTS - 1u; slot > 0u;) {
            --slot;
            if (row[slot * SLOT_WORDS + 5u] < row[weakest * SLOT_WORDS + 5u]) {
                weakest = slot;
            }
        }
        row[weakest * SLOT_WORDS + 5u] -= 1u;
        if (row[weakest * SLOT_WORDS + 5u] != 0u) {
            return;
        }
        at = weakest;
    }
    u32* slot = row + at * SLOT_WORDS;
    for (u32 field = 0; field < 5u; ++field) {
        slot[field] = entry[field];
    }
    slot[5] = 1u;
    slot[6] = 0u;
}

__device__ static bool names(const u32* entry, const Place& at, u32 target) {
    Named named = name(entry, at);
    return named.ok && named.token == target;
}

// The pointer of this back, ahead and width that reads `target` here, bound
// to the id before it (Entry::pointer under the pair stand).
__device__ static bool pointer_to(const Place& at, u32 back, u32 ahead, u32 wide, u32 target,
                                  u32* entry) {
    if (at.start < 1u + back) {
        return false;
    }
    const u32* tok = at.tok;
    u32 anchor = at.start - 1u - back;
    u32 low = at.floor + wide;
    u32 before = 0u;
    if (ahead == 0u) {
        if (anchor < low) {
            return false;
        }
    } else {
        u32 stand = NONE;
        for (u32 place = anchor; place > low;) {
            --place;
            if (place + ahead >= anchor || tok[place] != tok[anchor]) {
                continue;
            }
            if (wide && tok[place - 1u] != tok[anchor - 1u]) {
                continue;
            }
            if (tok[place + ahead] == target) {
                stand = place;
                break;
            }
        }
        if (stand == NONE) {
            return false;
        }
        before = tok[stand + ahead - 1u];
    }
    entry[0] = 2u;
    entry[1] = back;
    entry[2] = ahead;
    entry[3] = wide;
    entry[4] = before;
    return true;
}

// The pointers one output error adds at a place: the first three, by back,
// then ahead, the wide one first, that read the target.
struct Pointers {
    u32 ready;
    u32 count;
    u32 entry[POINTERS_ADDED][5];
};

__device__ static void pointers_at(const Place& at, u32 target, Pointers& found) {
    found.ready = 1u;
    found.count = 0u;
    for (u32 back = 0; back < POINTER_BACK; ++back) {
        for (u32 ahead = 0; ahead <= POINTER_AHEAD; ++ahead) {
            for (u32 narrow = 0; narrow < 2u; ++narrow) {
                u32 wide = 1u - narrow;
                if (ahead == 0u && wide) {
                    continue;
                }
                u32* entry = found.entry[found.count];
                if (!pointer_to(at, back, ahead, wide, target, entry)) {
                    continue;
                }
                if (!names(entry, at, target)) {
                    continue;
                }
                if (++found.count == POINTERS_ADDED) {
                    return;
                }
            }
        }
    }
}

// Every slot that names the target grows and every other slot that names
// anything is marked wrong. When none names it the id and each read that
// names it are added (HeldRow::take, precise, not explain).
__device__ static void row_take(u32* row, const Place& at, u32 target, Pointers& pointers) {
    bool hit = false;
    for (u32 slot = 0; slot < MAX_SLOTS; ++slot) {
        u32* entry = row + slot * SLOT_WORDS;
        if (entry[5] == 0u) {
            continue;
        }
        Named named = name(entry, at);
        if (!named.ok) {
            continue;
        }
        if (named.token == target) {
            if (entry[5] != 0xFFFFFFFFu) {
                entry[5] += 1u;
            }
            hit = true;
        } else if (entry[6] != 0xFFFFFFFFu) {
            entry[6] += 1u;
        }
    }
    if (hit) {
        return;
    }
    u32 entry[5] = {1u, 0u, 0u, 0u, target};
    row_insert(row, entry);
    for (u32 kind = 3u; kind <= 4u; ++kind) {
        u32 read[5] = {kind, 0u, 0u, 0u, 0u};
        if (names(read, at, target)) {
            row_insert(row, read);
        }
    }
    if (!pointers.ready) {
        pointers_at(at, target, pointers);
    }
    for (u32 index = 0; index < pointers.count; ++index) {
        row_insert(row, pointers.entry[index]);
    }
}

// The place the first ranked live copying slot read `target` from
// (HeldRow::source_of with a least count of one).
__device__ static u32 row_source(const u32* row, const Place& at, u32 target) {
    u32 order[MAX_SLOTS];
    u32 live = rank(row, order);
    for (u32 turn = 0; turn < live; ++turn) {
        Named named = name(row + order[turn] * SLOT_WORDS, at);
        if (named.ok && named.token == target && named.source != NONE) {
            return named.source;
        }
    }
    return NONE;
}

// The tails of a place's own row and, after a copied id, of the shared row
// (History::tails over Place::ahead).
__device__ static void tails(const Hist& hist, const Place& at, u32* own, u32* shared,
                             bool* has_shared) {
    bool has_ahead = at.cursor != NONE && at.cursor + 1u < at.start;
    bool copying = hist_copying(hist);
    *own = AT_START;
    *shared = 0u;
    *has_shared = false;
    if (copying && has_ahead) {
        *own = at.tok[at.cursor + 1u];
        *shared = AFTER_COPY;
        *has_shared = true;
    } else if (copying) {
        *own = AFTER_COPY;
    }
}

// Second and third pass (Asked::write_forms): the thread's reply is written
// under its forms, place by place, each place read before it is written.
// jobs: start, end, form count, six form ids. `order` lists the level's jobs.
extern "C" __global__ void train_write(const u32* cfg, const u32* tok, const u32* jobs,
                                       const u32* order, u32* state, u32* keys, u32* slots,
                                       u32* flags, u32 first, u32 count, u32 empty) {
    u32 mine = blockIdx.x * blockDim.x + threadIdx.x;
    if (mine >= count) {
        return;
    }
    const u32* job = jobs + (u64)order[first + mine] * JOB_WORDS;
    u32 start = job[0];
    u32 end = job[1];
    u32 held_count = job[2];
    const u32* held = job + 3;
    Rows rows;
    rows.state = state;
    rows.keys = keys;
    rows.slots = slots;
    rows.capacity = cfg[CFG_ROW_SLOTS];
    Hist hist;
    hist.count = 0u;
    hist.last = NONE;
    u32 cursor = NONE;
    for (u32 position = start; position < end; ++position) {
        u32 target = tok[position];
        Place at;
        at.tok = tok;
        at.len = position;
        at.floor = window_floor(tok, position, cfg[CFG_CONTEXT], cfg[CFG_BOS]);
        at.start = start;
        at.cursor = cursor;
        u32 own;
        u32 shared;
        bool has_shared;
        tails(hist, at, &own, &shared, &has_shared);
        u32 history[8];
        hist_address(hist, history);
        u32 unused = 0u;
        Answer read = place_answer(rows, held, held_count, history, own, shared, has_shared,
                                   empty, at, &unused);
        Pointers pointers;
        pointers.ready = 0u;
        pointers.count = 0u;
        u32 written = NONE;
        u32 key[KEY_WORDS];
        for (u32 index = 0; index < held_count; ++index) {
            for (u32 which = 0; which < (has_shared ? 2u : 1u); ++which) {
                row_key(held[index], empty, which ? shared : own, history, key);
                u32 slot = slot_of(state, keys, rows.capacity, KEY_WORDS, key, true);
                if (slot == NONE) {
                    flags[FLAG_ROWS] = 1u;
                    return;
                }
                u32* row = slots + (u64)slot * ROW_WORDS;
                row_take(row, at, target, pointers);
                if (written == NONE) {
                    written = row_source(row, at, target);
                }
            }
        }
        cursor = read.found && read.named.token == target ? read.named.source : written;
        hist_push(hist, target, cursor);
    }
}

// The exact copy read on the window [floor, len) (copy_at).
__device__ static u32 copy_read(const u32* tok, u32 floor, u32 len) {
    if (len - floor < 3u) {
        return NONE;
    }
    u32 last = len - 1u;
    for (u32 place = last; place > floor + 1u;) {
        --place;
        if (tok[place - 1u] == tok[last - 1u] && tok[place] == tok[last]) {
            return tok[place + 1u];
        }
    }
    return NONE;
}

// The pointer with the largest count, the earliest slot on a tie (Row::output).
// A pair row: eight pointers, eight counts, two trust counts.
__device__ static u32 pair_output(const u32* row) {
    u32 best = NONE;
    u32 most = 0u;
    for (u32 slot = 0; slot < MAX_SLOTS; ++slot) {
        if (row[MAX_SLOTS + slot] > most) {
            most = row[MAX_SLOTS + slot];
            best = slot;
        }
    }
    return best == NONE ? NONE : row[best];
}

// The pair rows (Ladder::write with no order above the pair): a thread a
// pair replays that pair's TRAIN targets in stream order. Each target judges
// the row against the copy read, then the row takes it.
extern "C" __global__ void train_pairs(const u32* cfg, const u32* tok, const u32* offsets,
                                       const u32* positions, u32* pair_rows, u32 pairs) {
    u32 pair = blockIdx.x * blockDim.x + threadIdx.x;
    if (pair >= pairs) {
        return;
    }
    u32 ladder_slots = cfg[CFG_LADDER_SLOTS];
    u32 row[PAIR_WORDS];
    for (u32 word = 0; word < PAIR_WORDS; ++word) {
        row[word] = 0u;
    }
    bool held = false;
    for (u32 index = offsets[pair]; index < offsets[pair + 1u]; ++index) {
        u32 position = positions[index];
        u32 target = tok[position];
        if (!held) {
            row[0] = target;
            row[MAX_SLOTS] = 1u;
            held = true;
            continue;
        }
        u32 floor = window_floor(tok, position, cfg[CFG_CONTEXT], cfg[CFG_BOS]);
        u32 copy = copy_read(tok, floor, position);
        u32 output = pair_output(row);
        if (copy != NONE && output != NONE && copy != output) {
            u32 side = copy == target ? 0u : (output == target ? 1u : NONE);
            if (side != NONE && row[2u * MAX_SLOTS + side] != 0xFFFFFFFFu) {
                row[2u * MAX_SLOTS + side] += 1u;
            }
        }
        // Row::take.
        bool grew = false;
        for (u32 slot = 0; slot < ladder_slots; ++slot) {
            if (row[MAX_SLOTS + slot] > 0u && row[slot] == target) {
                if (row[MAX_SLOTS + slot] != 0xFFFFFFFFu) {
                    row[MAX_SLOTS + slot] += 1u;
                }
                grew = true;
                break;
            }
        }
        if (grew) {
            continue;
        }
        bool full = true;
        for (u32 slot = 0; slot < ladder_slots; ++slot) {
            full = full && row[MAX_SLOTS + slot] > 0u;
        }
        if (full) {
            for (u32 slot = 0; slot < ladder_slots; ++slot) {
                row[MAX_SLOTS + slot] -= 1u;
            }
        }
        for (u32 slot = 0; slot < ladder_slots; ++slot) {
            if (row[MAX_SLOTS + slot] == 0u) {
                row[slot] = target;
                row[MAX_SLOTS + slot] = 1u;
                break;
            }
        }
    }
    for (u32 word = 0; word < PAIR_WORDS; ++word) {
        pair_rows[(u64)pair * PAIR_WORDS + word] = row[word];
    }
}

// The next id after the window [floor, len) from the pair row and the copy
// read (Ladder::next with no order above the pair). NONE: nothing answers.
__device__ static u32 ladder_next(const u32* tok, u32 floor, u32 len, const u32* pair_keys,
                                  const u32* pair_rows, u32 pairs) {
    u32 copy = copy_read(tok, floor, len);
    const u32* row = 0;
    if (len - floor >= 2u) {
        u32 key = tok[len - 2u] * 4096u + tok[len - 1u];
        u32 found = find(pair_keys, pairs, 1u, &key);
        if (found != NONE) {
            row = pair_rows + (u64)found * PAIR_WORDS;
        }
    }
    u32 output = row ? pair_output(row) : NONE;
    if (output == NONE) {
        return copy;
    }
    if (copy == NONE || copy == output) {
        return output;
    }
    return row[2u * MAX_SLOTS] > row[2u * MAX_SLOTS + 1u] ? copy : output;
}

// Before the third pass (Asked::write_empty over Asked::generate): a thread a
// job replies to its own TRAIN prompt from the rows of the second pass and
// reports whether any place met an empty read. work: one row of `stride` ids
// a job. out: 1 where a read came up empty.
extern "C" __global__ void train_missed(const u32* cfg, const u32* tok, const u32* jobs,
                                        u32* state, u32* keys, u32* slots, const u32* pair_keys,
                                        const u32* pair_rows, u32* work, u32* out, u32* flags,
                                        u32 pairs, u32 job_count) {
    u32 mine = blockIdx.x * blockDim.x + threadIdx.x;
    if (mine >= job_count) {
        return;
    }
    const u32* job = jobs + (u64)mine * JOB_WORDS;
    u32 held_count = job[2];
    const u32* held = job + 3;
    u32 context = cfg[CFG_CONTEXT];
    u32 bos = cfg[CFG_BOS];
    u32 eos = cfg[CFG_EOS];
    u32 limit = cfg[CFG_LIMIT];
    // The reader sees the prompt's own window, not the stream before it.
    u32 from = window_floor(tok, job[0], context, bos);
    u32 start = job[0] - from;
    out[mine] = 0u;
    if (start + limit > cfg[CFG_STRIDE]) {
        flags[FLAG_WORK] = 1u;
        return;
    }
    u32* reply = work + (u64)mine * cfg[CFG_STRIDE];
    for (u32 place = 0; place < start; ++place) {
        reply[place] = tok[from + place];
    }
    Rows rows;
    rows.state = state;
    rows.keys = keys;
    rows.slots = slots;
    rows.capacity = cfg[CFG_ROW_SLOTS];
    Hist hist;
    hist.count = 0u;
    hist.last = NONE;
    u32 cursor = NONE;
    u32 missed = 0u;
    u32 len = start;
    while (len - start < limit) {
        Place at;
        at.tok = reply;
        at.len = len;
        at.floor = window_floor(reply, len, context, bos);
        at.start = start;
        at.cursor = cursor;
        u32 own;
        u32 shared;
        bool has_shared;
        tails(hist, at, &own, &shared, &has_shared);
        u32 history[8];
        hist_address(hist, history);
        Answer answer = place_answer(rows, held, held_count, history, own, shared, has_shared, 0u,
                                     at, &missed);
        u32 token;
        if (answer.found) {
            token = answer.named.token;
            cursor = answer.named.source;
        } else {
            token = ladder_next(reply, at.floor, len, pair_keys, pair_rows, pairs);
            if (token == NONE) {
                break;
            }
            cursor = NONE;
        }
        hist_push(hist, token, cursor);
        reply[len++] = token;
        if (token == eos) {
            break;
        }
    }
    out[mine] = missed;
}
