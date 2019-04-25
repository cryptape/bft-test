use crate::whitebox::correctness::test_case::BftTestUnit;
use rand::{thread_rng, Rng};

pub(crate) fn rand_attribute(attri: u8, base: u8) -> BftTestUnit {
    let mut rng = thread_rng();
    let index_1: usize = rng.gen_range(0, 3);
    let index_2: usize = rng.gen_range(3, 6);
    let mut unit = [base; 6];
    for (index, item) in unit.iter_mut().enumerate() {
        if index == index_1 || index == index_2 {
            *item = attri;
        }
    }
    unit
}

pub(crate) fn rand_two_attribute(attri: u8, base_prevote: u8, base_precommit: u8) -> BftTestUnit {
    let mut rng = thread_rng();
    let index_1: usize = rng.gen_range(0, 3);
    let index_2: usize = rng.gen_range(3, 6);
    let mut unit = [
        base_prevote,
        base_prevote,
        base_prevote,
        base_precommit,
        base_precommit,
        base_precommit,
    ];
    for (index, item) in unit.iter_mut().enumerate() {
        if index != index_1 && index != index_2 {
            *item = attri;
        }
    }
    unit
}

#[cfg(test)]
mod test {
    use super::*;

    fn sum(a: [u8; 6]) -> usize {
        let mut sum = 0;
        for i in 0..6 {
            sum += a[i] as usize;
        }
        sum
    }

    #[test]
    fn unit_test() {
        assert_eq!(sum(rand_attribute(0, 1)), 4);
        assert_eq!(sum(rand_two_attribute(0, 1, 1)), 2);
    }
}
