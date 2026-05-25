#include "dds_context.h"

#include <api/calc_dd_table.hpp>
#include <api/calc_par.hpp>
#include <api/solve_board.hpp>
#include <solver_context/solver_context.hpp>

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
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

inline int resolved_hw_workers() {
  unsigned hw = std::thread::hardware_concurrency();
  return hw == 0 ? 1 : static_cast<int>(hw);
}

// Record first non-success status; later workers do not clobber it.
inline void record_err(std::atomic<int>& slot, int status) {
  if (status == RETURN_NO_FAULT) return;
  int expected = RETURN_NO_FAULT;
  slot.compare_exchange_strong(expected, status, std::memory_order_relaxed,
                               std::memory_order_relaxed);
}

// Persistent worker pool: a fixed-size set of std::threads, each owning its
// own SolverContext, that drain work from the same atomic counter. We pay
// the thread-creation cost exactly once instead of on every batched FFI
// call, which avoids the heavy glibc-allocator churn that was triggering
// SIGSEGVs in TransTableL::lookup_suit / Moves::MergeSort during
// solve_deals(N>=200) on >=8-core Linux boxes. This mirrors the persistent
// pool used by the ddss fork, which doesn't reproduce the crash.
//
// Side benefit: per-worker SolverContexts (and thus their transposition
// tables) persist across batches, so the TT can warm up.
//
// Config caveat: the pool's per-worker SolverContexts are constructed from
// the SolverConfig seen on the first call to run(). Later calls with a
// different cfg silently keep using the original config. The Rust caller
// today always passes SolverConfig::default(), so this is fine in practice;
// callers that need varying configs should use the single-shot
// SolverContext API instead.
class WorkerPool {
 public:
  static WorkerPool& instance() {
    static WorkerPool pool;
    return pool;
  }

  // Block until all n_total work units have been processed.
  // n_threads_requested is honored only on first call (to size the pool);
  // <= 0 means hardware_concurrency().
  int run(int n_total, int n_threads_requested, SolverConfig const& sc,
          std::function<int(SolverContext&, int)> per_item) {
    if (n_total <= 0) return RETURN_NO_FAULT;

    // Serialize concurrent run() calls. We could pool work across them, but
    // the FFI is normally driven from a single Rust thread per batch and the
    // mutex keeps the state machine trivial.
    std::lock_guard<std::mutex> submit_lk(submit_mtx_);
    ensure_workers(n_threads_requested, sc);

    std::unique_lock<std::mutex> state_lk(state_mtx_);
    per_item_ = std::move(per_item);
    n_total_ = n_total;
    next_idx_.store(0, std::memory_order_relaxed);
    first_err_.store(RETURN_NO_FAULT, std::memory_order_relaxed);
    workers_done_.store(0, std::memory_order_relaxed);
    ++epoch_;
    cv_work_.notify_all();

    cv_done_.wait(state_lk, [this] {
      return workers_done_.load(std::memory_order_acquire) == n_workers_;
    });

    return first_err_.load(std::memory_order_relaxed);
  }

  ~WorkerPool() { shutdown(); }

 private:
  WorkerPool() = default;
  WorkerPool(WorkerPool const&) = delete;
  WorkerPool& operator=(WorkerPool const&) = delete;

  void ensure_workers(int n_threads_requested, SolverConfig const& sc) {
    if (n_workers_ > 0) return;
    n_workers_ = (n_threads_requested > 0) ? n_threads_requested
                                           : resolved_hw_workers();
    contexts_.reserve(static_cast<size_t>(n_workers_));
    threads_.reserve(static_cast<size_t>(n_workers_));
    for (int i = 0; i < n_workers_; ++i) {
      contexts_.emplace_back(std::make_unique<SolverContext>(sc));
      threads_.emplace_back([this, i] { worker_loop(i); });
    }
  }

  void worker_loop(int worker_id) {
    std::uint64_t my_epoch = 0;
    SolverContext& ctx = *contexts_[worker_id];
    for (;;) {
      std::unique_lock<std::mutex> state_lk(state_mtx_);
      cv_work_.wait(state_lk, [this, &my_epoch] {
        return shutdown_ || epoch_ > my_epoch;
      });
      if (shutdown_) return;
      my_epoch = epoch_;
      int const local_n_total = n_total_;
      // per_item_ is stable for the duration of this epoch (submit_mtx_
      // held by main until cv_done_ fires), so we can safely use the
      // member without copying.
      state_lk.unlock();

      for (;;) {
        int i = next_idx_.fetch_add(1, std::memory_order_relaxed);
        if (i >= local_n_total) break;
        int status = per_item_(ctx, i);
        record_err(first_err_, status);
      }

      int const done = workers_done_.fetch_add(1, std::memory_order_release) + 1;
      if (done == n_workers_) {
        std::lock_guard<std::mutex> lk(state_mtx_);
        cv_done_.notify_one();
      }
    }
  }

  void shutdown() {
    {
      std::lock_guard<std::mutex> lk(state_mtx_);
      shutdown_ = true;
      cv_work_.notify_all();
    }
    for (auto& t : threads_) {
      if (t.joinable()) t.join();
    }
  }

  int n_workers_ = 0;
  std::vector<std::unique_ptr<SolverContext>> contexts_;
  std::vector<std::thread> threads_;

  std::mutex submit_mtx_;
  std::mutex state_mtx_;
  std::condition_variable cv_work_;
  std::condition_variable cv_done_;

  std::function<int(SolverContext&, int)> per_item_;
  int n_total_ = 0;
  std::uint64_t epoch_ = 0;
  bool shutdown_ = false;

  std::atomic<int> next_idx_{0};
  std::atomic<int> first_err_{RETURN_NO_FAULT};
  std::atomic<int> workers_done_{0};
};

template <typename Fn>
int run_batched(int n_total, int n_threads_requested,
                const DdsSolverConfig* cfg, Fn&& per_item) {
  if (n_total <= 0) return RETURN_NO_FAULT;
  SolverConfig sc = to_cpp_or_default(cfg);

  // Single-threaded path: drive the work inline in the caller's thread
  // and skip the pool entirely. Useful for callers that explicitly want
  // a serial loop (n_threads_requested == 1) and for n_total == 1 where
  // spinning up workers is pure overhead.
  if (n_threads_requested == 1 || n_total == 1) {
    SolverContext ctx(sc);
    int first_err = RETURN_NO_FAULT;
    for (int i = 0; i < n_total; ++i) {
      int status = per_item(ctx, i);
      if (status != RETURN_NO_FAULT && first_err == RETURN_NO_FAULT) {
        first_err = status;
      }
    }
    return first_err;
  }

  return WorkerPool::instance().run(
      n_total, n_threads_requested, sc, std::forward<Fn>(per_item));
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
