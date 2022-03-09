
use blake2b_ref::{Blake2b, Blake2bBuilder};

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn new_blake2b() -> Blake2b {
	Blake2bBuilder::new(CKB_HASH_DIGEST)
	    .personal(CKB_HASH_PERSONALIZATION)
	    .build()
}

pub fn blake2b_256(s: &[u8]) -> [u8; 32] {
	let mut result = [0u8; CKB_HASH_DIGEST];
	let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
		.personal(CKB_HASH_PERSONALIZATION)
		.build();
	blake2b.update(s);
	blake2b.finalize(&mut result);
	result
}