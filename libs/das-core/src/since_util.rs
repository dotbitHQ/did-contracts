/// WARNING! This is copy from `tests/src/util/since_util.rs`, so please do not modify it here.
/// All tests can be run by `docker.sh test-...`

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SinceFlag {
    Relative,
    Absolute,
    Epoch,
    Timestamp,
    Height,
}

macro_rules! is_set {
    ($n:expr, $b:expr) => {
        $n & (1 << $b) != 0
    };
}

macro_rules! is_not_set {
    ($n:expr, $b:expr) => {
        $n & (1 << $b) == 0
    };
}

macro_rules! set {
    ($n:expr, $b:expr) => {
        $n | 1 << $b
    };
}

macro_rules! unset {
    ($n:expr, $b:expr) => {
        $n & !(1 << $b)
    };
}

pub fn get_absolute_flag(since: u64) -> SinceFlag {
    if is_set!(since, 63) {
        SinceFlag::Absolute
    } else {
        SinceFlag::Relative
    }
}

pub fn set_absolute_flag(since: u64, flag: SinceFlag) -> u64 {
    match flag {
        SinceFlag::Absolute => set!(since, 63),
        SinceFlag::Relative => unset!(since, 63),
        _ => panic!(),
    }
}

pub fn get_metric_flag(since: u64) -> SinceFlag {
    if is_not_set!(since, 62) && is_not_set!(since, 61) {
        SinceFlag::Height
    } else if is_set!(since, 62) && is_not_set!(since, 61) {
        SinceFlag::Timestamp
    } else if is_not_set!(since, 62) && is_set!(since, 61) {
        SinceFlag::Epoch
    } else {
        panic!();
    }
}

pub fn set_metric_flag(since: u64, flag: SinceFlag) -> u64 {
    match flag {
        SinceFlag::Timestamp => set!(unset!(since, 61), 62),
        SinceFlag::Epoch => set!(unset!(since, 62), 61),
        SinceFlag::Height => unset!(unset!(since, 61), 62),
        _ => panic!(),
    }
}

pub fn get_value(since: u64) -> u64 {
    since & 0b00000000_11111111_11111111_11111111_11111111_11111111_11111111_11111111
}

pub fn set_value(since: u64, value: u64) -> u64 {
    since & 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000 | value
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_since_absolute_flag_getter() {
        let since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_absolute_flag(since);

        // println!("0b{:064b}", since);
        assert_eq!(flag, SinceFlag::Relative);

        let since = 0b10000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_absolute_flag(since);

        assert_eq!(flag, SinceFlag::Absolute);
    }

    #[test]
    fn test_since_absolute_flag_setter() {
        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_absolute_flag(since, SinceFlag::Absolute);
        let flag = get_absolute_flag(since);

        assert_eq!(flag, SinceFlag::Absolute);

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_absolute_flag(since, SinceFlag::Relative);
        let flag = get_absolute_flag(since);

        assert_eq!(flag, SinceFlag::Relative);

        let mut since = 0b10000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_absolute_flag(since, SinceFlag::Relative);
        let flag = get_absolute_flag(since);

        assert_eq!(flag, SinceFlag::Relative);
    }

    #[test]
    fn test_since_metric_flag_getter() {
        let since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Height);

        let since = 0b01000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Timestamp);

        let since = 0b00100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Epoch);
    }

    #[test]
    #[should_panic]
    fn test_since_metric_flag_getter_panic() {
        let since = 0b01100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        get_metric_flag(since);
    }

    #[test]
    fn test_since_metric_flag_setter() {
        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Height);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Height);

        let mut since = 0b01000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Height);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Height);

        let mut since = 0b00100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Height);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Height);

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Timestamp);

        let mut since = 0b00100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Timestamp);

        let mut since = 0b01100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Timestamp);

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Epoch);

        let mut since = 0b01000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Epoch);

        let mut since = 0b01100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);
        let flag = get_metric_flag(since);

        assert_eq!(flag, SinceFlag::Epoch);
    }

    #[test]
    fn test_since_value_getter() {
        let since = 0b11000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let value = get_value(since);
        // println!("0b{:064b}", since);

        let expected_value = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        assert_eq!(expected_value, value);
    }

    #[test]
    fn test_since_value_setter() {
        let since = 0b11000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
        let value = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let new_value = set_value(since, value);
        // println!("0b{:064b}", since);

        let expected_value = 0b11000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        assert_eq!(expected_value, new_value);
    }
}
