// Host build of the FBS core, for the finite parity tests against the Rust
// integer session only. Panels are evaluated by src/evaluator.cu on the GPU
// (CHARTER §3.6); the CLI never calls this.
#include "fbs_core.h"

#include <vector>

extern "C" int64_t fbs_host_sizeof_model(void) { return (int64_t)sizeof(fbs_model); }
extern "C" int64_t fbs_host_sizeof_layer(void) { return (int64_t)sizeof(fbs_layer); }

// `sequences` sequences of `len` tokens each, row-major; outputs per position.
extern "C" int fbs_host_sequences(const fbs_model *m, const uint32_t *tokens, uint32_t sequences,
                                  uint32_t len, uint32_t *out_tokens, uint32_t *out_kept,
                                  int32_t *out_query) {
  uint32_t d = m->width, mlp = m->mlp, wide = d > mlp ? d : mlp, g = d / 4 + d;
  std::vector<int32_t> x(d), branches(2 * d), gate_out(g), proj(d), gate(mlp), up(mlp), query(d);
  std::vector<int64_t> scratch(wide), wide_buf(d), cells(FBS_CELLS), state((size_t)m->layer_count * d);
  std::vector<int16_t> act(wide);
  std::vector<int32_t> history((size_t)m->layer_count * 3 * d);
  fbs_work w = {x.data(),    scratch.data(), branches.data(), gate_out.data(), wide_buf.data(),
                act.data(),  proj.data(),    gate.data(),     up.data(),       query.data(),
                cells.data(), state.data(),  history.data()};
  for (uint32_t s = 0; s < sequences; s++) {
    uint64_t at = (uint64_t)s * len;
    int err = fbs_sequence(m, &w, tokens + at, len, out_tokens + at, out_kept + at * m->kept,
                           out_query + at * d);
    if (err != FBS_OK) return err;
  }
  return FBS_OK;
}
