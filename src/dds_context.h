#ifndef DDS_BRIDGE_SYS_DDS_CONTEXT_H
#define DDS_BRIDGE_SYS_DDS_CONTEXT_H

#include <api/dll.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct DdsSolverContext DdsSolverContext;

typedef enum DdsTtKind {
  DDS_TT_KIND_SMALL = 0,
  DDS_TT_KIND_LARGE = 1
} DdsTtKind;

typedef struct DdsSolverConfig {
  int tt_kind;
  int tt_mem_default_mb;
  int tt_mem_maximum_mb;
} DdsSolverConfig;

DdsSolverContext* dds_solver_context_new(const DdsSolverConfig* cfg);
void              dds_solver_context_free(DdsSolverContext* ctx);

void dds_solver_context_reset_for_solve(DdsSolverContext* ctx);
void dds_solver_context_clear_tt(DdsSolverContext* ctx);
void dds_solver_context_resize_tt(DdsSolverContext* ctx, int def_mb, int max_mb);
void dds_solver_context_configure_tt(DdsSolverContext* ctx, int kind, int def_mb, int max_mb);
void dds_solver_context_dispose_trans_table(DdsSolverContext* ctx);

int dds_solve_board(DdsSolverContext* ctx, const struct Deal* dl,
                    int target, int solutions, int mode,
                    struct FutureTricks* fut);

int dds_calc_dd_table(DdsSolverContext* ctx, const struct DdTableDeal* d,
                      struct DdTableResults* out);

int dds_calc_dd_table_pbn(DdsSolverContext* ctx, const struct DdTableDealPBN* d,
                          struct DdTableResults* out);

int dds_calc_par(DdsSolverContext* ctx, const struct DdTableDeal* d, int vul,
                 struct DdTableResults* tab, struct ParResults* par);

int dds_calc_par_from_table(const struct DdTableResults* tab, int vul,
                            struct ParResults* par);

#ifdef __cplusplus
}
#endif

#endif
