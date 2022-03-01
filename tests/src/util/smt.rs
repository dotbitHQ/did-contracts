use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::{blake2b::Blake2bHasher, default_store::DefaultStore, MerkleProof, SparseMerkleTree, H256};

pub type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

pub struct History {
    prev_root: H256,
    current_root: H256,
    proof: MerkleProof,
}

pub struct SMTWithHistory {
    pub smt: SMT,
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
    pub fn insert(&mut self, key: H256, value: H256) -> ([u8; 32], [u8; 32], Vec<u8>) {
        let prev_root = self.smt.root().to_owned();
        println!();
        println!("prev_root     = 0x{}", hex::encode(prev_root.as_slice()));
        let proof_bytes: Vec<u8> = self
            .smt
            .merkle_proof(vec![key])
            .expect("Should generate proof successfully")
            .compile(vec![(key, value)])
            .unwrap()
            .into();
        println!("prev_proof    = 0x{}", hex::encode(proof_bytes));

        self.smt.update(key, value);
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

        let proof_bytes = match proof.compile(vec![(key, value)]) {
            Ok(compiled_proof) => compiled_proof.into(),
            Err(e) => {
                panic!("Generate compiled proof failed: {}", e);
            }
        };

        println!("current_root  = 0x{}", hex::encode(current_root.as_slice()));
        println!("current_proof = 0x{}", hex::encode(&proof_bytes));

        (prev_root.into(), current_root.into(), proof_bytes)
    }
}
