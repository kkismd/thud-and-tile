use crate::srs::{WALL_KICKS_I, WALL_KICKS_JLSZT};

// JLSZT Mino Wall Kicks
#[test]
fn test_jlszt_wall_kicks_0_to_r() {
    let expected_kicks = [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[0], expected_kicks,
        "JLSZT 0->R kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_r_to_0() {
    let expected_kicks = [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[1], expected_kicks,
        "JLSZT R->0 kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_r_to_2() {
    let expected_kicks = [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[2], expected_kicks,
        "JLSZT R->2 kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_2_to_r() {
    let expected_kicks = [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[3], expected_kicks,
        "JLSZT 2->R kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_2_to_l() {
    let expected_kicks = [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[4], expected_kicks,
        "JLSZT 2->L kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_l_to_2() {
    let expected_kicks = [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[5], expected_kicks,
        "JLSZT L->2 kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_l_to_0() {
    let expected_kicks = [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[6], expected_kicks,
        "JLSZT L->0 kicks are incorrect"
    );
}

#[test]
fn test_jlszt_wall_kicks_0_to_l() {
    let expected_kicks = [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];
    assert_eq!(
        WALL_KICKS_JLSZT.kicks[7], expected_kicks,
        "JLSZT 0->L kicks are incorrect"
    );
}

// I Mino Wall Kicks
#[test]
fn test_i_wall_kicks_0_to_r() {
    let expected_kicks = [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)];
    assert_eq!(
        WALL_KICKS_I.kicks[0], expected_kicks,
        "I-Mino 0->R kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_r_to_0() {
    let expected_kicks = [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)];
    assert_eq!(
        WALL_KICKS_I.kicks[1], expected_kicks,
        "I-Mino R->0 kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_r_to_2() {
    let expected_kicks = [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)];
    assert_eq!(
        WALL_KICKS_I.kicks[2], expected_kicks,
        "I-Mino R->2 kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_2_to_r() {
    let expected_kicks = [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)];
    assert_eq!(
        WALL_KICKS_I.kicks[3], expected_kicks,
        "I-Mino 2->R kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_2_to_l() {
    let expected_kicks = [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)];
    assert_eq!(
        WALL_KICKS_I.kicks[4], expected_kicks,
        "I-Mino 2->L kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_l_to_2() {
    let expected_kicks = [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)];
    assert_eq!(
        WALL_KICKS_I.kicks[5], expected_kicks,
        "I-Mino L->2 kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_l_to_0() {
    let expected_kicks = [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)];
    assert_eq!(
        WALL_KICKS_I.kicks[6], expected_kicks,
        "I-Mino L->0 kicks are incorrect"
    );
}

#[test]
fn test_i_wall_kicks_0_to_l() {
    let expected_kicks = [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)];
    assert_eq!(
        WALL_KICKS_I.kicks[7], expected_kicks,
        "I-Mino 0->L kicks are incorrect"
    );
}
