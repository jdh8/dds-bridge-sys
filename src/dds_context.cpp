#include "dds_context.h"

#include <api/calc_dd_table.hpp>
#include <api/calc_par.hpp>
#include <api/solve_board.hpp>
#include <solver_context/solver_context.hpp>

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

}
