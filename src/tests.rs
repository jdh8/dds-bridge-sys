use parking_lot::Mutex;
use std::sync::LazyLock;

static THREAD_POOL: LazyLock<Mutex<()>> = LazyLock::new(|| {
    unsafe { crate::SetMaxThreads(0) };
    Mutex::new(())
});

#[allow(clippy::large_types_passed_by_value)]
fn check(
    deal: crate::DdTableDeal,
    solution: crate::DdTableResults,
    pars: [crate::ParResultsMaster; 2],
) {
    #[allow(clippy::cast_possible_wrap)]
    const SUCCESS: i32 = crate::RETURN_NO_FAULT as i32;
    let mut tricks = crate::DdTableResults::default();
    let status = unsafe {
        let _guard = THREAD_POOL.lock();
        crate::CalcDDtable(deal, &raw mut tricks)
    };
    assert_eq!(status, SUCCESS);
    assert_eq!(tricks, solution);

    let mut result = [crate::ParResultsMaster::default(); 2];
    let status = unsafe { crate::SidesParBin(&raw mut tricks, &raw mut result[0], 0) };
    assert_eq!(status, SUCCESS);
    assert_eq!(result, pars);
}

const NO_CONTRACT: crate::ContractType = crate::ContractType {
    level: 0,
    denom: 0,
    seats: 0,
    under_tricks: 0,
    over_tricks: 0,
};

/// Everyone has a 13-card straight flush, and the par is 7SW=.
#[test]
fn solve_four_13_card_straight_flushes() {
    const MASK: core::ffi::c_uint = ((1 << 13) - 1) << 2;
    const DEAL: crate::DdTableDeal = crate::DdTableDeal {
        cards: [
            [0, 0, 0, MASK], // N
            [0, 0, MASK, 0], // E
            [0, MASK, 0, 0], // S
            [MASK, 0, 0, 0], // W
        ],
    };
    const SOLUTION: crate::DdTableResults = crate::DdTableResults {
        res_table: [
            [0, 13, 0, 13], // S
            [13, 0, 13, 0], // H
            [0, 13, 0, 13], // D
            [13, 0, 13, 0], // C
            [0, 0, 0, 0],   // NT
        ],
    };
    const CONTRACTS: [crate::ContractType; 10] = [
        crate::ContractType {
            level: 7,
            denom: 1, // spades
            seats: 5,
            under_tricks: 0,
            over_tricks: 0,
        },
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
        NO_CONTRACT,
    ];
    const NS: crate::ParResultsMaster = crate::ParResultsMaster {
        score: -1510,
        number: 1,
        contracts: CONTRACTS,
    };
    const EW: crate::ParResultsMaster = crate::ParResultsMaster {
        score: 1510,
        number: 1,
        contracts: CONTRACTS,
    };
    check(DEAL, SOLUTION, [NS, EW]);
}

/// Defenders can cash 8 tricks in every strain.
///
/// This example is taken from
/// <http://bridge.thomasoandrews.com/deals/parzero/>.
#[test]
fn solve_par_5_tricks() {
    const AKQJ: core::ffi::c_uint = 0xF << 11;
    const T987: core::ffi::c_uint = 0xF << 7;
    const XXXX: core::ffi::c_uint = 0xF << 3;
    const X: core::ffi::c_uint = 1 << 2;

    const DEAL: crate::DdTableDeal = crate::DdTableDeal {
        cards: [
            [AKQJ, X, XXXX, T987], // N
            [XXXX, T987, AKQJ, X], // E
            [X, AKQJ, T987, XXXX], // S
            [T987, XXXX, X, AKQJ], // W
        ],
    };
    const SOLUTION: crate::DdTableResults = crate::DdTableResults {
        res_table: [[5; 4]; 5],
    };
    const PAR: crate::ParResultsMaster = crate::ParResultsMaster {
        score: 0,
        number: 1,
        contracts: [NO_CONTRACT; 10],
    };
    check(DEAL, SOLUTION, [PAR; 2]);
}

/// Smoke test for the modern `SolverContext` C shim.
///
/// Solves the four-straight-flush deal via `dds_calc_dd_table` and asserts
/// the same trick table as the legacy `CalcDDtable` path.
#[test]
fn solver_context_solve_dd_table() {
    const MASK: core::ffi::c_uint = ((1 << 13) - 1) << 2;
    const DEAL: crate::DdTableDeal = crate::DdTableDeal {
        cards: [
            [0, 0, 0, MASK],
            [0, 0, MASK, 0],
            [0, MASK, 0, 0],
            [MASK, 0, 0, 0],
        ],
    };
    const SOLUTION: crate::DdTableResults = crate::DdTableResults {
        res_table: [
            [0, 13, 0, 13],
            [13, 0, 13, 0],
            [0, 13, 0, 13],
            [13, 0, 13, 0],
            [0, 0, 0, 0],
        ],
    };
    #[allow(clippy::cast_possible_wrap)]
    const SUCCESS: i32 = crate::RETURN_NO_FAULT as i32;

    let cfg = crate::DdsSolverConfig {
        tt_kind: crate::DDS_TT_KIND_LARGE.try_into().unwrap(),
        tt_mem_default_mb: 0,
        tt_mem_maximum_mb: 0,
    };
    let deal = DEAL;
    let mut tricks = crate::DdTableResults::default();
    let status = unsafe {
        let _guard = THREAD_POOL.lock();
        let ctx = crate::dds_solver_context_new(&raw const cfg);
        assert!(!ctx.is_null());
        let s = crate::dds_calc_dd_table(ctx, &raw const deal, &raw mut tricks);
        crate::dds_solver_context_free(ctx);
        s
    };
    assert_eq!(status, SUCCESS);
    assert_eq!(tricks, SOLUTION);
}

/// A symmetric deal where everyone makes 1NT but no suit contract
///
/// This example is taken from
/// <http://www.rpbridge.net/7a23.htm#2>.
#[test]
#[allow(clippy::unusual_byte_groupings)]
fn solve_everyone_makes_1nt() {
    const A54: core::ffi::c_uint = 0b10000_0000_1100_00;
    const QJ32: core::ffi::c_uint = 0b00110_0000_0011_00;
    const K976: core::ffi::c_uint = 0b01000_1011_0000_00;
    const T8: core::ffi::c_uint = 0b00001_0100_0000_00;

    const DEAL: crate::DdTableDeal = crate::DdTableDeal {
        cards: [
            [A54, QJ32, K976, T8], // N
            [T8, A54, QJ32, K976], // E
            [K976, T8, A54, QJ32], // S
            [QJ32, K976, T8, A54], // W
        ],
    };
    const SOLUTION: crate::DdTableResults = crate::DdTableResults {
        res_table: [[6; 4], [6; 4], [6; 4], [6; 4], [7; 4]],
    };
    const NS: crate::ParResultsMaster = crate::ParResultsMaster {
        score: 90,
        number: 1,
        contracts: [
            crate::ContractType {
                level: 1,
                denom: 0, // notrump
                seats: 4,
                under_tricks: 0,
                over_tricks: 0,
            },
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
        ],
    };
    const EW: crate::ParResultsMaster = crate::ParResultsMaster {
        score: 90,
        number: 1,
        contracts: [
            crate::ContractType {
                level: 1,
                denom: 0, // notrump
                seats: 5,
                under_tricks: 0,
                over_tricks: 0,
            },
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
            NO_CONTRACT,
        ],
    };
    check(DEAL, SOLUTION, [NS, EW]);
}
