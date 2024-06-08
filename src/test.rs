fn lock_thread_pool() -> std::sync::MutexGuard<'static, ()> {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static POOL: Lazy<Mutex<()>> = Lazy::new(|| {
        unsafe { crate::SetMaxThreads(0) };
        Mutex::new(())
    });

    POOL.lock().unwrap_or_else(|e| {
        POOL.clear_poison();
        e.into_inner()
    })
}

fn check(deal: crate::ddTableDeal, solution: crate::ddTableResults) {
    #[allow(clippy::cast_possible_wrap)]
    const SUCCESS: i32 = crate::RETURN_NO_FAULT as i32;
    let mut result = crate::ddTableResults::default();
    let _guard = lock_thread_pool();
    let status = unsafe { crate::CalcDDtable(deal, &mut result) };
    assert_eq!(status, SUCCESS);
    assert_eq!(result, solution);
}

/// Everyone has a 13-card straight flush, and the par is 7SW=.
#[test]
fn solve_four_13_card_straight_flushes() {
    const MASK: core::ffi::c_uint = ((1 << 13) - 1) << 2;
    const DEAL: crate::ddTableDeal = crate::ddTableDeal {
        cards: [
            [0, 0, 0, MASK], // N
            [0, 0, MASK, 0], // E
            [0, MASK, 0, 0], // S
            [MASK, 0, 0, 0], // W
        ],
    };
    const SOLUTION: crate::ddTableResults = crate::ddTableResults {
        resTable: [
            [0, 13, 0, 13], // S
            [13, 0, 13, 0], // H
            [0, 13, 0, 13], // D
            [13, 0, 13, 0], // C
            [0, 0, 0, 0],   // NT
        ],
    };
    check(DEAL, SOLUTION);
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

    const DEAL: crate::ddTableDeal = crate::ddTableDeal {
        cards: [
            [AKQJ, X,    XXXX, T987], // N
            [XXXX, T987, AKQJ, X   ], // E
            [X,    AKQJ, T987, XXXX], // S
            [T987, XXXX, X,    AKQJ], // W
        ],
    };
    const SOLUTION: crate::ddTableResults = crate::ddTableResults {
        resTable: [[5; 4]; 5],
    };
    check(DEAL, SOLUTION);
}