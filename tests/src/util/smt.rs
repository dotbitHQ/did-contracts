use ckb_hash::{Blake2b, Blake2bBuilder};
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::traits::Hasher;
pub use sparse_merkle_tree::MerkleProof;
use sparse_merkle_tree::{SparseMerkleTree, H256};

type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

struct Blake2bHasher(Blake2b);

impl Default for Blake2bHasher {
    fn default() -> Self {
        // THe smt C use "ckb-default-hash" as personalization argument, so we must implement the Blake2bHasher by ourself.
        let blake2b = Blake2bBuilder::new(32).personal(b"ckb-default-hash").key(&[]).build();
        Blake2bHasher(blake2b)
    }
}

impl Hasher for Blake2bHasher {
    fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice());
    }
    fn write_byte(&mut self, b: u8) {
        self.0.update(&[b][..]);
    }
    fn finish(self) -> H256 {
        let mut hash = [0u8; 32];
        self.0.finalize(&mut hash);
        hash.into()
    }
}

pub struct History {
    prev_root: H256,
    current_root: H256,
    proof: MerkleProof,
}

pub struct SMTWithHistory {
    smt: SMT,
    pub history: Vec<History>,
}

impl SMTWithHistory {
    pub fn new() -> SMTWithHistory {
        let smt = SMT::default();

        return SMTWithHistory {
            smt,
            history: Vec::new(),
        };
    }

    /// Return current root of the sparse-merkle-tree.
    pub fn current_root(&self) -> [u8; 32] {
        self.smt.root().to_owned().into()
    }

    /// Restore the spare-merkle-tree to a specific state by inserting multiple leaves.
    pub fn restore_state(&mut self, leaves: Vec<(H256, H256)>) {
        self.smt.update_all(leaves).expect("Should restore SMT successfully");
    }

    /// Insert a leaf(a pair of key and value) into the sparse-merkle-tree and return prev_root, current_root, proof of the inserted leaf.
    ///
    /// The returned value is exactly what a sub_account witness want, so use it when you need to construct sub_account witness.
    pub fn insert(&mut self, key: H256, value: H256) -> ([u8; 32], [u8; 32], MerkleProof) {
        let prev_root = self.smt.root().to_owned();
        self.smt.update(key, value).expect("Should update successfully");
        let current_root = self.smt.root().to_owned();
        let proof = self
            .smt
            .merkle_proof(vec![key])
            .expect("Should generate proof successfully");

        self.history.push(History {
            prev_root,
            current_root,
            proof: proof.clone(),
        });

        // println!("current_root  = 0x{}", hex::encode(current_root.as_slice()));
        // println!("current_proof = 0x{}", hex::encode(&proof_bytes));
        (prev_root.into(), current_root.into(), proof)
    }

    pub fn get_proof(&self, keys: Vec<H256>) -> MerkleProof {
        self.smt.merkle_proof(keys).expect("Should generate proof successfully")
    }

    pub fn get_compiled_proof(&self, leaves: Vec<(H256, H256)>) -> Vec<u8> {
        let keys = leaves.iter().map(|(k, _)| k.to_owned()).collect();
        let proof = self.smt.merkle_proof(keys).expect("Should generate proof successfully");
        proof
            .compile(leaves)
            .expect("Proof should be compiled successfully")
            .into()
    }

    pub fn compile_proof(proof: MerkleProof, leaves: Vec<(H256, H256)>) -> Vec<u8> {
        proof
            .compile(leaves)
            .expect("Proof should be compiled successfully")
            .into()
    }
}

#[test]
fn smt_test_current_root() {
    let mut smt = SMTWithHistory::new();
    let key_1 = H256::from([1u8; 32]);
    let value_1 = H256::from([1u8; 32]);
    let key_2 = H256::from([2u8; 32]);
    let value_2 = H256::from([2u8; 32]);
    smt.insert(key_1, value_1);
    let (_, current_root, _) = smt.insert(key_2, value_2);

    let current_root = hex::encode(current_root);
    let expected_root = hex::encode([
        189, 54, 5, 77, 115, 196, 110, 91, 204, 52, 94, 3, 153, 190, 52, 225, 253, 27, 39, 149, 102, 196, 214, 87, 36,
        202, 18, 184, 14, 109, 74, 10,
    ]);
    assert!(
        expected_root == current_root,
        "Expected root: 0x{}, actual root: 0x{}",
        expected_root,
        current_root
    );
}

#[test]
fn smt_test_restore_state() {
    let mut smt = SMTWithHistory::new();
    let key_1 = H256::from([1u8; 32]);
    let value_1 = H256::from([1u8; 32]);
    let key_2 = H256::from([2u8; 32]);
    let value_2 = H256::from([2u8; 32]);
    smt.restore_state(vec![(key_1, value_1), (key_2, value_2)]);

    let current_root = hex::encode(smt.current_root());
    let expected_root = hex::encode([
        189, 54, 5, 77, 115, 196, 110, 91, 204, 52, 94, 3, 153, 190, 52, 225, 253, 27, 39, 149, 102, 196, 214, 87, 36,
        202, 18, 184, 14, 109, 74, 10,
    ]);
    assert!(
        expected_root == current_root,
        "Expected root: 0x{}, actual root: 0x{}",
        expected_root,
        current_root
    );
}

#[test]
fn smt_test_insert() {
    let mut smt = SMTWithHistory::new();
    let key_1 = H256::from([1u8; 32]);
    let value_1 = H256::from([1u8; 32]);
    let key_2 = H256::from([2u8; 32]);
    let value_2 = H256::from([2u8; 32]);
    let key_3 = H256::from([3u8; 32]);
    let value_3 = H256::from([3u8; 32]);

    let (_, root_1, proof_1) = smt.insert(key_1, value_1);
    let (_, root_2, proof_2) = smt.insert(key_2, value_2);
    let (_, root_3, proof_3) = smt.insert(key_3, value_3);

    let root_1_hex = hex::encode(root_1);
    let expected_root_1 = hex::encode([
        53, 234, 182, 66, 181, 102, 7, 156, 17, 211, 36, 42, 149, 110, 212, 223, 127, 132, 159, 211, 119, 142, 56, 101,
        155, 146, 50, 81, 197, 7, 5, 11,
    ]);
    assert!(
        expected_root_1 == root_1_hex,
        "Expected root_1: 0x{}, actual root_1: 0x{}",
        expected_root_1,
        root_1_hex
    );
    let root_2_hex = hex::encode(root_2);
    let expected_root_2 = hex::encode([
        189, 54, 5, 77, 115, 196, 110, 91, 204, 52, 94, 3, 153, 190, 52, 225, 253, 27, 39, 149, 102, 196, 214, 87, 36,
        202, 18, 184, 14, 109, 74, 10,
    ]);
    assert!(
        expected_root_2 == root_2_hex,
        "Expected root_2: 0x{}, actual root_2: 0x{}",
        expected_root_2,
        root_2_hex
    );
    let root_3_hex = hex::encode(root_3);
    let expected_root_3 = hex::encode([
        112, 38, 186, 222, 170, 241, 57, 204, 124, 249, 91, 200, 44, 181, 47, 191, 114, 126, 19, 80, 72, 240, 140, 225,
        225, 149, 212, 97, 15, 228, 71, 103,
    ]);
    assert!(
        expected_root_3 == root_3_hex,
        "Expected root_3: 0x{}, actual root_3: 0x{}",
        expected_root_3,
        root_3_hex
    );

    assert!(
        proof_1
            .verify::<Blake2bHasher>(&H256::from(root_1), vec![(key_1, value_1)])
            .is_ok(),
        "Proof_1 should be verified"
    );
    assert!(
        proof_2
            .verify::<Blake2bHasher>(&H256::from(root_2), vec![(key_2, value_2)])
            .is_ok(),
        "Proof_2 should be verified"
    );
    assert!(
        proof_3
            .verify::<Blake2bHasher>(&H256::from(root_3), vec![(key_3, value_3)])
            .is_ok(),
        "Proof_3 should be verified"
    );
}

#[test]
fn smt_test_compile_proof() {
    use sparse_merkle_tree::SMTBuilder;

    let mut smt = SMTWithHistory::new();
    let key_1 = H256::from([1u8; 32]);
    let value_1 = H256::from([1u8; 32]);
    let key_2 = H256::from([2u8; 32]);
    let value_2 = H256::from([2u8; 32]);
    let key_3 = H256::from([3u8; 32]);
    let value_3 = H256::from([3u8; 32]);

    smt.insert(key_1, value_1);
    smt.insert(key_2, value_2);
    smt.insert(key_3, value_3);

    let proof = smt.get_proof(vec![key_1, key_2, key_3]);
    let compiled_proof =
        SMTWithHistory::compile_proof(proof, vec![(key_1, value_1), (key_2, value_2), (key_3, value_3)]);
    println!("compiled_proof = 0x{}", hex::encode(&compiled_proof));

    let mut smt_c_builder = SMTBuilder::new();
    smt_c_builder = smt_c_builder.insert(&key_1, &value_1).unwrap();
    smt_c_builder = smt_c_builder.insert(&key_2, &value_2).unwrap();
    smt_c_builder = smt_c_builder.insert(&key_3, &value_3).unwrap();

    let root = H256::from(smt.current_root());
    let smt_c = smt_c_builder.build().unwrap();
    let ret = smt_c.verify(&root, &compiled_proof);

    assert!(ret.is_ok(), "Proof should be verified");
}
