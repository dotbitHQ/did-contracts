use blake2b_ref::{Blake2b, Blake2bBuilder};
use std::prelude::v1::*;

fn make_hashers() -> Vec<Blake2b> {
    vec![
        Blake2bBuilder::new(8).personal(b"das-hasher-1").build(),
        Blake2bBuilder::new(8).personal(b"das-hasher-2").build(),
    ]
}

pub struct BloomFilter {
    /// The bit array of _m_ bits that stores existence information of elements.
    pub bits: Vec<bool>,
    /// Count of hash functions. Denoted by _k_.
    hash_fn_count: usize,
}

impl BloomFilter {
    /// Creates an empty Bloom filter with desired capacity and error rate.
    ///
    /// This constructor would give an optimal size for bit array based on
    /// provided `capacity` and `err_rate`.
    ///
    /// # Parameters
    ///
    /// * `capacity` - Expected size of elements will put in.
    pub fn new(bits_count: u64, hash_fn_count: u64) -> Self {
        Self {
            bits: vec![false; bits_count as usize],
            hash_fn_count: (hash_fn_count as usize),
        }
    }

    pub fn new_with_data(bits_count: u64, hash_fn_count: u64, b_u8: &[u8]) -> Self {
        let mut bv = vec![false; bits_count as usize];
        let mut i = 0;
        for c in b_u8 {
            if i > bits_count as usize {
                break;
            }
            bv[i] = if c == &0u8 { false } else { true };
            i = i + 1;
        }
        Self {
            bits: bv,
            hash_fn_count: hash_fn_count as usize,
        }
    }

    pub fn export_bit_u8(&self) -> Vec<u8> {
        let l: usize = self.bits.len();
        let mut b_u8 = vec![0; l];
        for i in 0..l {
            b_u8[i] = if self.bits[i] { 1 } else { 0 };
        }
        b_u8
    }

    /// Inserts an element into the container.
    ///
    /// This function simulates multiple hashers with only two hashers using
    /// the following formula:
    ///
    /// > g_i(x) = h1(x) + i * h2(x)
    ///
    /// # Parameters
    ///
    /// * `elem` - Element to be inserted.
    ///
    /// # Complexity
    ///
    /// Linear in the size of `hash_fn_count` _k_.
    pub fn insert(&mut self, elem: &[u8]) {
        // g_i(x) = h1(x) + i * h2(x)
        let hashes = self.calcu_hashes(elem);
        let mut indexes = Vec::new();
        for fn_i in 0..self.hash_fn_count {
            let index = self.get_index(&hashes, fn_i as u64);
            indexes.push(index);
            self.bits[index] = true;
        }

        // #[cfg(debug_assertions)]
        // println!("Insert {:?} to {:?}", elem, indexes);
    }

    /// Returns whether an element is present in the container.
    ///
    /// # Parameters
    ///
    /// * `elem` - Element to be checked whether is in the container.
    ///
    /// # Complexity
    ///
    /// Linear in the size of `hash_fn_count` _k_.
    pub fn contains(&self, elem: &[u8]) -> bool {
        let hashes = self.calcu_hashes(elem);
        let mut indexes = Vec::new();
        let result = (0..self.hash_fn_count).all(|fn_i| {
            let index = self.get_index(&hashes, fn_i as u64);
            indexes.push(index);
            self.bits[index]
        });

        // #[cfg(debug_assertions)]
        // println!("Check {:?} contains by {:?}", elem, indexes);
        result
    }

    /// Gets index of the bit array for a single hash iteration.
    ///
    /// As a part of multiple hashers simulation for this formula:
    ///
    /// > g_i(x) = h1(x) + i * h2(x)
    ///
    /// This function calculate the right hand side of the formula.
    ///
    /// Note that the usage fo `wrapping_` is acceptable here for a hash
    /// algorithm to get a valid slot.
    fn get_index(&self, hashes: &Vec<u64>, fn_i: u64) -> usize {
        let h1 = hashes[0];
        let h2 = hashes[1];
        (h1.wrapping_add(fn_i.wrapping_mul(h2)) % self.bits.len() as u64) as usize
    }

    /// Hashes the element.
    ///
    /// As a part of multiple hashers simulation for this formula:
    ///
    /// > g_i(x) = h1(x) + i * h2(x)
    ///
    /// This function do the actual `hash` work with two independant hashers,
    /// returing both h1(x) and h2(x) within a tuple.
    fn calcu_hashes(&self, elem: &[u8]) -> Vec<u64> {
        let mut results = Vec::new();
        let hashers = make_hashers();
        for mut hasher in hashers {
            let mut result = [0u8; 8];
            hasher.update(elem);
            hasher.finalize(&mut result);

            let num = u64::from_le_bytes(result);
            results.push(num);
        }

        // #[cfg(debug_assertions)]
        // println!("Hash {:?} to {:?}", elem, results);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    const BITS_COUNT: u64 = 1918;
    const HASH_FN_COUNT: u64 = 14;

    #[cfg(feature = "std")]
    fn optimal_bits_count(capacity: f64, err_rate: f64) -> f64 {
        let ln_2_2 = std::f64::consts::LN_2.powf(2f64);

        // m = -1 * (n * ln ε) / (ln 2)^2
        (-1f64 * capacity * err_rate.ln() / ln_2_2).ceil()
    }

    #[cfg(feature = "std")]
    fn optimal_hashers_count(err_rate: f64) -> f64 {
        // k = -log_2 ε
        (-1f64 * err_rate.log2()).ceil()
    }

    /// This test is just used for calculating bloom filter params.
    #[cfg(feature = "std")]
    // #[test]
    fn calculate_params() {
        let bits_count = optimal_bits_count(100f64, 0.0001);
        let hash_fn_count = optimal_hashers_count(0.0001);

        println!("\nbits_count = {:#?}", bits_count);
        println!("hash_fn_count = {:#?}", hash_fn_count);
    }

    // #[test]
    fn test_create_bloom_filter() {
        let mut bf = BloomFilter::new(100, 17);
        bf.insert(b"google");
        bf.insert(b"facebook");
        bf.insert(b"twitter");

        assert!(bf.contains(b"google"));
        assert!(!bf.contains(b"das"));
    }

    #[test]
    fn test_export_and_import_bloom_filter() {
        let mut all_items = Vec::new();
        let mut bf = BloomFilter::new(BITS_COUNT, HASH_FN_COUNT);

        // Insert random items.
        for _ in 1..100 {
            let item: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(8)
                .collect();
            all_items.push(item.to_owned());
            bf.insert(item.as_bytes());
        }
        bf.insert(b"das");

        let filter = bf.export_bit_u8();

        let bf2 = BloomFilter::new_with_data(BITS_COUNT, HASH_FN_COUNT, filter.as_slice());

        // println!("0x{}", hex::encode(filter));

        assert!(
            bf.bits == bf2.bits,
            "Bits of BloomFilter should be equal before and after exporting."
        );

        let mut all_items_found = true;
        for (i, item) in all_items.into_iter().enumerate() {
            if !bf2.contains(item.as_bytes()) {
                println!("Item [{}]{} is missing.", i, item);
                all_items_found = false;
            }
        }

        assert!(all_items_found, "All items should contains in filter.");
        assert!(
            bf2.contains(b"das"),
            "Item 'das' should contains in filter."
        );
        assert!(
            !bf2.contains(b"link"),
            "Item 'link' should not contains in filter."
        );
    }
}
