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

pub fn get_relative_flag(since: u64) -> SinceFlag {
    if is_set!(since, 63) {
        SinceFlag::Relative
    } else {
        SinceFlag::Absolute
    }
}

pub fn set_relative_flag(since: u64, flag: SinceFlag) -> u64 {
    match flag {
        SinceFlag::Absolute => unset!(since, 63),
        SinceFlag::Relative => set!(since, 63),
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
    fn test_since_relative_flag_getter() {
        let since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_relative_flag(since);

        // println!("0b{:064b}", since);
        assert_eq!(flag, SinceFlag::Absolute);

        let since = 0b10000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        let flag = get_relative_flag(since);

        assert_eq!(flag, SinceFlag::Relative);
    }

    #[test]
    fn test_since_relative_flag_setter() {
        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_relative_flag(since, SinceFlag::Absolute);

        assert!(is_not_set!(since, 63));

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_relative_flag(since, SinceFlag::Relative);

        assert!(is_set!(since, 63));

        let mut since = 0b10000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_relative_flag(since, SinceFlag::Relative);

        assert!(is_set!(since, 63));
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

        assert!(is_not_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b01000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Height);

        assert!(is_not_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b00100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Height);

        assert!(is_not_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);

        assert!(is_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b00100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);

        assert!(is_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b01100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Timestamp);

        assert!(is_set!(since, 62) && is_not_set!(since, 61));

        let mut since = 0b00000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);

        assert!(is_not_set!(since, 62) && is_set!(since, 61));

        let mut since = 0b01000000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);

        assert!(is_not_set!(since, 62) && is_set!(since, 61));

        let mut since = 0b01100000_11111110_11111110_11111111_11111111_11111111_11111111_11111111;
        since = set_metric_flag(since, SinceFlag::Epoch);

        assert!(is_not_set!(since, 62) && is_set!(since, 61));
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
