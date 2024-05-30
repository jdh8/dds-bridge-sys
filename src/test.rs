/// Solve a weird deal whose par is 7SW=.
#[test]
fn solve_a_special_deal() {
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
    #[allow(clippy::cast_possible_wrap)]
    const SUCCESS: i32 = crate::RETURN_NO_FAULT as i32;
    let mut result = crate::ddTableResults::default();
    let status = unsafe { crate::CalcDDtable(DEAL, &mut result) };
    assert_eq!(status, SUCCESS);
    assert_eq!(result, SOLUTION);
}
