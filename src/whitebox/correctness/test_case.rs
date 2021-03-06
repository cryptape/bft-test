use crate::whitebox::correctness::util::*;
use rand::random;
use std::collections::HashMap;
// use std::time::Duration;

/// A basic test unit.
pub type BftTestUnit = [u8; 6];
/// A BFT test case.
pub type BftTest = Vec<BftTestUnit>;

pub(crate) const OFFLINE: u8 = 0;
pub(crate) const NORMAL: u8 = 1;
pub(crate) const BYZANTINE: u8 = 2;
pub(crate) const NIL: u8 = 3;
pub(crate) const NULL_ROUND: [u8; 6] = [7, 7, 7, 7, 7, 7];
pub(crate) const SHOULD_COMMIT: [u8; 6] = [8, 8, 8, 8, 8, 8];
pub(crate) const SHOULD_NOT_COMMIT: [u8; 6] = [9, 9, 9, 9, 9, 9];

pub(crate) fn byzantine_proposal() -> Vec<Vec<u8>> {
    vec![
        vec![0, 0, 0, 0, 0, 0],
        vec![1, 1, 1, 1, 1, 1],
        vec![2, 2, 2, 2, 2, 2],
    ]
}

/// A function to generate no byzantine test cases.
pub fn no_byzantine_cases() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..100 {
        cases.push([1, 1, 1, 1, 1, 1]);
        cases.push(SHOULD_COMMIT);
    }
    cases
}

/// A function to generate one node offline test cases.
pub fn one_offline_cases() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..100 {
        cases.push(rand_attribute(OFFLINE, NORMAL));
        cases.push(SHOULD_COMMIT);
    }
    cases
}

/// A function to generate one byzantine node test cases.
pub fn one_byzantine_cases() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..100 {
        cases.push(rand_attribute(BYZANTINE, NORMAL));
        cases.push(SHOULD_COMMIT);
    }
    cases
}

/// A function to generate two byzantine node test cases.
pub fn two_byzantine_cases() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..20 {
        cases.push(rand_two_attribute(BYZANTINE, NORMAL, NORMAL));
        cases.push(SHOULD_NOT_COMMIT);
    }
    cases.push([1, 1, 1, 1, 1, 1]);
    cases.push(SHOULD_COMMIT);
    cases
}

/// A function to generate two byzantine node and one offline node test cases.
pub fn two_byzantine_one_offline() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..10 {
        cases.push(rand_two_attribute(BYZANTINE, OFFLINE, NIL));
        cases.push(SHOULD_NOT_COMMIT);
    }
    cases.push([1, 1, 1, 1, 1, 1]);
    cases.push(SHOULD_COMMIT);
    cases
}

/// A function to generate round leap test cases.
pub fn round_leap_cases() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..3 {
        cases.push([1, 2, 2, 3, 3, 3]);
        cases.push(SHOULD_NOT_COMMIT);
        cases.push(NULL_ROUND);
        cases.push(NULL_ROUND);
        cases.push(NULL_ROUND);
        cases.push([1, 1, 1, 1, 1, 1]);
        cases.push(SHOULD_COMMIT);
    }
    cases
}

/// A function to generate proposal with lock test cases.
pub fn lock_proposal() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..10 {
        if random::<bool>() {
            cases.push([1, 1, 1, 1, 0, 2]);
        } else {
            cases.push([1, 2, 0, 1, 2, 0]);
        }
        cases.push(SHOULD_NOT_COMMIT);
    }
    cases.push([1, 1, 1, 1, 1, 1]);
    cases.push(SHOULD_COMMIT);
    cases
}

///  A function to generate proposal with lock test cases.
pub fn proposal_with_lock() -> BftTest {
    let mut cases = Vec::new();
    for _ in 0..10 {
        cases.push([1, 3, 0, 1, 3, 3]);
        cases.push(SHOULD_NOT_COMMIT);
        cases.push([1, 3, 1, 1, 3, 3]);
        cases.push(SHOULD_NOT_COMMIT);
        cases.push([1, 3, 0, 1, 3, 3]);
        cases.push(SHOULD_NOT_COMMIT);
        cases.push([0, 3, 1, 1, 3, 3]);
        cases.push(SHOULD_NOT_COMMIT);
        cases.push([1, 1, 1, 1, 1, 1]);
        cases.push(SHOULD_COMMIT);
    }
    cases
}

pub(crate) fn all_cases() -> HashMap<String, BftTest> {
    let mut test_cases = HashMap::new();

    test_cases
        .entry("test no byzantine case".to_string())
        .or_insert_with(no_byzantine_cases);
    test_cases
        .entry("test one byzantine case".to_string())
        .or_insert_with(one_byzantine_cases);
    test_cases
        .entry("test one offline case".to_string())
        .or_insert_with(one_offline_cases);
    test_cases
        .entry("test two byzantine case".to_string())
        .or_insert_with(two_byzantine_cases);
    test_cases
        .entry("test two byzantine and one offline case".to_string())
        .or_insert_with(two_byzantine_one_offline);
    test_cases
        .entry("test lock proposal".to_string())
        .or_insert_with(lock_proposal);
    test_cases
}

#[cfg(test)]
mod test {
    use super::*;

    fn should_commit(mut prevote: Vec<u8>, mut precommit: Vec<u8>) -> bool {
        prevote.push(1);
        precommit.push(1);
        let mut prevote_count: usize = 0;
        let mut precommit_count: usize = 0;
        for v in prevote.iter() {
            if v == &1 {
                prevote_count += 1;
            }
        }
        if prevote_count >= 3 {
            for v in precommit.iter() {
                if v == &1 {
                    precommit_count += 1
                }
            }
            if precommit_count >= 3 {
                return true;
            }
        }
        false
    }

    #[test]
    fn test_devide() {
        let unit: BftTestUnit = [1, 1, 1, 2, 2, 2];
        let cases: BftTest = vec![unit];
        for case in cases.iter() {
            let prevote = case[0..3].to_vec();
            let precommit = case[3..6].to_vec();
            assert_eq!(prevote, vec![1, 1, 1]);
            assert_eq!(precommit, vec![2, 2, 2]);
        }
    }

    #[test]
    fn test_cases_retional() {
        let all_test_cases = all_cases();
        let mut commit_flag: bool = true;
        for (test_name, test_case) in all_test_cases.into_iter() {
            println!("Test retional of {:?}", test_name);
            for case in test_case.iter() {
                if case == &SHOULD_COMMIT || case == &SHOULD_NOT_COMMIT {
                    if commit_flag {
                        assert_eq!(case.to_vec(), SHOULD_COMMIT);
                    } else {
                        assert_eq!(case.to_vec(), SHOULD_NOT_COMMIT);
                    }
                } else if case != &NULL_ROUND {
                    let prevote = case[0..3].to_vec();
                    let precommit = case[3..6].to_vec();
                    commit_flag = should_commit(prevote, precommit);
                }
            }
        }
    }
}
