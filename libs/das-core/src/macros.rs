#[macro_export]
macro_rules! debug {
    ($fmt:literal) => {
        #[cfg(not(feature = "mainnet"))]
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(not(feature = "mainnet"))]
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal) => {
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}

#[macro_export]
macro_rules! assert {
    ($condition:expr, $error_code:expr, $fmt:literal) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return Err($error_code);
        }
    };
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        if !$condition {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return Err($error_code);
        }
    };
}

#[macro_export]
macro_rules! parse_witness {
    ($entity:expr, $entity_reader:expr, $parser:expr, $index:expr, $source:expr, $entity_type:ty) => {{
        let (_, _, mol_bytes) = $parser.verify_and_get($index, $source)?;
        $entity = <$entity_type>::from_slice(mol_bytes.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        $entity_reader = $entity.as_reader();
    }};
}

#[macro_export]
macro_rules! parse_account_cell_witness {
    ($entity:expr, $entity_reader:expr, $parser:expr, $index:expr, $source:expr) => {{
        let (version, _, mol_bytes) = $parser.verify_and_get($index, $source)?;
        if version == 1 {
            $entity = Box::new(
                AccountCellDataV1::from_slice(mol_bytes.as_reader().raw_data())
                    .map_err(|_| Error::WitnessEntityDecodingError)?,
            );
            $entity_reader = $entity.as_reader();
        } else {
            $entity = Box::new(
                AccountCellData::from_slice(mol_bytes.as_reader().raw_data())
                    .map_err(|_| Error::WitnessEntityDecodingError)?,
            );
            $entity_reader = $entity.as_reader();
        }
    }};
}
