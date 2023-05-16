use ckb_hash::{Blake2b, Blake2bBuilder};
use sparse_merkle_tree::{default_store::DefaultStore, CompiledMerkleProof};
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
    leaves: Vec<(H256, H256)>,
    pub history: Vec<History>,
}

impl SMTWithHistory {
    pub fn new() -> SMTWithHistory {
        let smt = SMT::default();

        return SMTWithHistory {
            smt,
            leaves: vec![],
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
        self.smt
            .update(key.clone(), value.clone())
            .expect("Should update successfully");
        let current_root = self.smt.root().to_owned();
        let proof = self
            .smt
            .merkle_proof(vec![key])
            .expect("Should generate proof successfully");

        self.leaves.push((key, value));
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

    pub fn get_compiled_proof(&self, keys: Vec<H256>) -> Vec<u8> {
        let proof = self.smt.merkle_proof(keys.clone()).expect("Should generate proof successfully");
        proof
            .compile(keys)
            .expect("Proof should be compiled successfully")
            .into()
    }

    pub fn compile_proof(proof: MerkleProof, keys: Vec<H256>) -> Vec<u8> {
        proof
            .compile(keys)
            .expect("Proof should be compiled successfully")
            .into()
    }

    pub fn verify(&self, compiled_proof: &[u8], leaves: Vec<(H256, H256)>) -> bool {
        let root = H256::from(self.current_root());
        let compiled_proof = CompiledMerkleProof(compiled_proof.to_vec());
        compiled_proof.verify::<Blake2bHasher>(&root, leaves).is_ok()
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
        250, 217, 253, 241, 90, 196, 134, 16, 171, 121, 226, 215, 222, 119, 20, 22, 51, 170, 64, 76, 187, 141, 238, 74,
        60, 253, 178, 162, 211, 138, 87, 221,
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
        250, 217, 253, 241, 90, 196, 134, 16, 171, 121, 226, 215, 222, 119, 20, 22, 51, 170, 64, 76, 187, 141, 238, 74,
        60, 253, 178, 162, 211, 138, 87, 221,
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
        89, 141, 136, 18, 208, 19, 19, 77, 44, 97, 74, 58, 20, 214, 25, 114, 233, 147, 145, 149, 88, 208, 6, 47, 60,
        71, 118, 85, 125, 70, 232, 70,
    ]);
    assert!(
        expected_root_1 == root_1_hex,
        "Expected root_1: 0x{}, actual root_1: 0x{}",
        expected_root_1,
        root_1_hex
    );
    let root_2_hex = hex::encode(root_2);
    let expected_root_2 = hex::encode([
        250, 217, 253, 241, 90, 196, 134, 16, 171, 121, 226, 215, 222, 119, 20, 22, 51, 170, 64, 76, 187, 141, 238, 74,
        60, 253, 178, 162, 211, 138, 87, 221,
    ]);
    assert!(
        expected_root_2 == root_2_hex,
        "Expected root_2: 0x{}, actual root_2: 0x{}",
        expected_root_2,
        root_2_hex
    );
    let root_3_hex = hex::encode(root_3);
    let expected_root_3 = hex::encode([
        202, 179, 188, 233, 182, 189, 192, 125, 97, 119, 102, 17, 35, 0, 62, 17, 111, 103, 138, 40, 107, 245, 45, 5,
        66, 202, 70, 236, 4, 149, 169, 99,
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

// #[test]
// fn smt_test_compile_proof() {
//     let mut smt = SMTWithHistory::new();
//     let key_1 = H256::from([1u8; 32]);
//     let value_1 = H256::from([1u8; 32]);
//     let key_2 = H256::from([2u8; 32]);
//     let value_2 = H256::from([2u8; 32]);
//     let key_3 = H256::from([3u8; 32]);
//     let value_3 = H256::from([3u8; 32]);

//     smt.insert(key_1, value_1);
//     smt.insert(key_2, value_2);
//     smt.insert(key_3, value_3);

//     let proof = smt.get_proof(vec![key_1, key_2, key_3]);
//     let compiled_proof =
//         SMTWithHistory::compile_proof(proof, vec![(key_1, value_1), (key_2, value_2), (key_3, value_3)]);
//     println!("compiled_proof = 0x{}", hex::encode(&compiled_proof));

//     let ret = compiled_proof.verify::<Blake2bHasher>(&H256::from(smt.current_root()), vec![
//         (&key_1, &value_1),
//         (&key_2, &value_2),
//         (&key_3, &value_3),
//     ]);
//     assert!(ret.is_ok(), "Proof should be verified");
// }
