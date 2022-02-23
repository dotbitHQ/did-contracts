#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
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
macro_rules! assert_lock_equal {
    (($cell_a_index:expr, $cell_a_source:expr), ($cell_b_index:expr, $cell_b_source:expr), $error_code:expr, $fmt:literal) => {{
        let cell_a_lock_hash = high_level::load_cell_lock_hash($cell_a_index, $cell_a_source).map_err(Error::from)?;
        let cell_b_lock_hash = high_level::load_cell_lock_hash($cell_b_index, $cell_b_source).map_err(Error::from)?;

        if cell_a_lock_hash != cell_b_lock_hash {
            ckb_std::syscalls::debug(alloc::format!($fmt));
            return Err($error_code);
        }
    }};
    ($condition:expr, $error_code:expr, $fmt:literal, $($args:expr),+) => {
        let cell_a_lock_hash = high_level::load_cell_lock_hash($cell_a_index, $cell_a_source).map_err(Error::from)?;
        let cell_b_lock_hash = high_level::load_cell_lock_hash($cell_b_index, $cell_b_source).map_err(Error::from)?;

        if cell_a_lock_hash != cell_b_lock_hash {
            ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
            return Err($error_code);
        }
    };
}

#[macro_export]
macro_rules! parse_witness {
    ($entity:expr, $entity_reader:expr, $parser:expr, $index:expr, $source:expr, $data_type:expr, $entity_type:ty) => {{
        let (_, _, mol_bytes) = $parser.verify_and_get($data_type, $index, $source)?;
        $entity = <$entity_type>::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
            $crate::warn!("Decoding {} failed", stringify!($entity_type));
            Error::WitnessEntityDecodingError
        })?;
        $entity_reader = $entity.as_reader();
    }};
}

#[macro_export]
macro_rules! parse_account_cell_witness {
    ($entity:expr, $entity_reader:expr, $parser:expr, $index:expr, $source:expr) => {{
        let (version, _, mol_bytes) =
            $parser.verify_and_get(das_types::constants::DataType::AccountCellData, $index, $source)?;
        if version <= 1 {
            // CAREFUL! The early versions will no longer be supported.
            return Err(Error::InvalidTransactionStructure);
        } else if version == 2 {
            $entity = Box::new(
                das_types::packed::AccountCellDataV2::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                    $crate::warn!("Decoding AccountCellDataV2 failed");
                    Error::WitnessEntityDecodingError
                })?,
            );
            $entity_reader = $entity.as_reader();
        } else {
            $entity = Box::new(
                das_types::packed::AccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                    $crate::warn!("Decoding AccountCellData failed");
                    Error::WitnessEntityDecodingError
                })?,
            );
            $entity_reader = $entity.as_reader();
        }
    }};
}

#[macro_export]
macro_rules! parse_account_sale_cell_witness {
    ($entity:expr, $entity_reader:expr, $parser:expr, $index:expr, $source:expr) => {{
        let (version, _, mol_bytes) =
            $parser.verify_and_get(das_types::constants::DataType::AccountSaleCellData, $index, $source)?;
        if version == 1 {
            $entity = Box::new(
                das_types::packed::AccountSaleCellDataV1::from_slice(mol_bytes.as_reader().raw_data()).map_err(
                    |_| {
                        $crate::warn!("Decoding AccountSaleCellDataV1 failed");
                        Error::WitnessEntityDecodingError
                    },
                )?,
            );
            $entity_reader = $entity.as_reader();
        } else {
            $entity = Box::new(
                das_types::packed::AccountSaleCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                    $crate::warn!("Decoding AccountSaleCellData failed");
                    Error::WitnessEntityDecodingError
                })?,
            );
            $entity_reader = $entity.as_reader();
        }
    }};
}
