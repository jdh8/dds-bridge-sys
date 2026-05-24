#include "dds_context.h"

#include <api/calc_dd_table.hpp>
#include <api/calc_par.hpp>
#include <api/solve_board.hpp>
#include <solver_context/solver_context.hpp>

#include <atomic>
#include <thread>
#include <vector>

namespace {

inline SolverContext* as_ctx(DdsSolverContext* h) {
  return reinterpret_cast<SolverContext*>(h);
}

inline SolverConfig to_cpp(const DdsSolverConfig* c) {
  SolverConfig out;
  out.tt_kind_ = static_cast<TTKind>(c->tt_kind);
  out.tt_mem_default_mb_ = c->tt_mem_default_mb;
  out.tt_mem_maximum_mb_ = c->tt_mem_maximum_mb;
  return out;
}

inline SolverConfig to_cpp_or_default(const DdsSolverConfig* c) {
  if (c == nullptr) return SolverConfig{};
  return to_cpp(c);
}

inline int resolve_workers(int requested, int work_units) {
  if (work_units <= 0) return 0;
  int n = requested;
  if (n <= 0) {
    unsigned hw = std::thread::hardware_concurrency();
    n = hw == 0 ? 1 : static_cast<int>(hw);
  }
  if (n > work_units) n = work_units;
  return n;
}

// Record first non-success status; later workers do not clobber it.
inline void record_err(std::atomic<int>& slot, int status) {
  if (status == RETURN_NO_FAULT) return;
  int expected = RETURN_NO_FAULT;
  slot.compare_exchange_strong(expected, status, std::memory_order_relaxed,
                               std::memory_order_relaxed);
}

template <typename Fn>
int run_batched(int n_total, int n_threads_requested,
                const DdsSolverConfig* cfg, Fn&& per_item) {
  if (n_total <= 0) return RETURN_NO_FAULT;
  int n_workers = resolve_workers(n_threads_requested, n_total);

  SolverConfig sc = to_cpp_or_default(cfg);
  std::atomic<int> next_idx{0};
  std::atomic<int> first_err{RETURN_NO_FAULT};

  auto worker = [&]() {
    SolverContext ctx(sc);
    for (;;) {
      int i = next_idx.fetch_add(1, std::memory_order_relaxed);
      if (i >= n_total) break;
      int status = per_item(ctx, i);
      record_err(first_err, status);
    }
  };

  if (n_workers <= 1) {
    worker();
    return first_err.load(std::memory_order_relaxed);
  }

  std::vector<std::thread> threads;
  threads.reserve(static_cast<size_t>(n_workers));
  for (int t = 0; t < n_workers; ++t) threads.emplace_back(worker);
  for (auto& th : threads) th.join();
  return first_err.load(std::memory_order_relaxed);
}

}

extern "C" {

DdsSolverContext* dds_solver_context_new(const DdsSolverConfig* cfg) {
  return reinterpret_cast<DdsSolverContext*>(new SolverContext(to_cpp(cfg)));
}

void dds_solver_context_free(DdsSolverContext* ctx) {
  delete as_ctx(ctx);
}

void dds_solver_context_reset_for_solve(DdsSolverContext* ctx) {
  as_ctx(ctx)->reset_for_solve();
}

void dds_solver_context_clear_tt(DdsSolverContext* ctx) {
  as_ctx(ctx)->clear_tt();
}

void dds_solver_context_resize_tt(DdsSolverContext* ctx, int def_mb, int max_mb) {
  as_ctx(ctx)->resize_tt(def_mb, max_mb);
}

void dds_solver_context_configure_tt(DdsSolverContext* ctx, int kind, int def_mb, int max_mb) {
  as_ctx(ctx)->configure_tt(static_cast<TTKind>(kind), def_mb, max_mb);
}

void dds_solver_context_dispose_trans_table(DdsSolverContext* ctx) {
  as_ctx(ctx)->dispose_trans_table();
}

int dds_solve_board(DdsSolverContext* ctx, const Deal* dl,
                    int target, int solutions, int mode,
                    FutureTricks* fut) {
  return solve_board(*as_ctx(ctx), *dl, target, solutions, mode, fut);
}

int dds_calc_dd_table(DdsSolverContext* ctx, const DdTableDeal* d,
                      DdTableResults* out) {
  return calc_dd_table(*as_ctx(ctx), *d, out);
}

int dds_calc_dd_table_pbn(DdsSolverContext* ctx, const DdTableDealPBN* d,
                          DdTableResults* out) {
  return calc_dd_table_pbn(*as_ctx(ctx), *d, out);
}

int dds_calc_par(DdsSolverContext* ctx, const DdTableDeal* d, int vul,
                 DdTableResults* tab, ParResults* par) {
  return calc_par(*as_ctx(ctx), *d, vul, tab, par);
}

int dds_calc_par_from_table(const DdTableResults* tab, int vul,
                            ParResults* par) {
  return calc_par_from_table(tab, vul, par);
}

int dds_calc_dd_tables_batched(int n_deals,
                               const DdTableDeal* deals,
                               DdTableResults* results,
                               int n_threads,
                               const DdsSolverConfig* cfg) {
  return run_batched(n_deals, n_threads, cfg,
                     [deals, results](SolverContext& ctx, int i) {
                       return calc_dd_table(ctx, deals[i], &results[i]);
                     });
}

int dds_solve_boards_batched(int n_boards,
                             const Deal* deals,
                             const int* targets,
                             const int* solutions,
                             const int* modes,
                             FutureTricks* results,
                             int n_threads,
                             const DdsSolverConfig* cfg) {
  return run_batched(
      n_boards, n_threads, cfg,
      [deals, targets, solutions, modes, results](SolverContext& ctx, int i) {
        return solve_board(ctx, deals[i], targets[i], solutions[i], modes[i],
                           &results[i]);
      });
}

}
