// We allow this file to be messy for now, as we're importing a lot from Neptune Core.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::vec_init_then_push)]

use tasm_lib::structure::tasm_object::TasmObject;
use tasm_lib::VmHasher as H;
use triton_vm::twenty_first::shared_math::bfield_codec::BFieldCodec;
use triton_vm::twenty_first::util_types::algebraic_hasher::AlgebraicHasher;
use triton_vm::twenty_first::util_types::merkle_tree::CpuParallel;
use triton_vm::twenty_first::util_types::merkle_tree_maker::MerkleTreeMaker;
use triton_vm::BFieldElement;
use triton_vm::Digest;

use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;

#[derive(Clone, Debug, PartialEq, Eq, BFieldCodec, TasmObject)]
pub struct TransactionKernel {
    pub mutator_set_hash: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, BFieldCodec, TasmObject)]
pub struct RemovalRecordsIntegrityWitness {
    pub kernel: TransactionKernel,
}

impl TransactionKernel {
    pub fn mast_sequences(&self) -> Vec<Vec<BFieldElement>> {
        let mutator_set_hash_sequence: Vec<BFieldElement> = self.mutator_set_hash.encode();

        // TODO: Adjust this capacity when more sequences are added!
        let mut ret: Vec<Vec<BFieldElement>> = Vec::<Vec<BFieldElement>>::with_capacity(1);
        ret.push(mutator_set_hash_sequence);

        return ret;
        // return vec![mutator_set_hash_sequence];
    }

    pub fn mast_hash(&self) -> Digest {
        // get a sequence of BFieldElements for each field
        // Note that this is a super stupid way to calculate the MAST hash, as the relevant
        // vectors are already present in memory and we could just hash them directly.
        // Here we're reconstructing those lists.
        let sequences: Vec<Vec<BFieldElement>> = self.mast_sequences();

        let sequence_count: usize = sequences.len();
        let mut mt_leafs: Vec<Digest> = Vec::<Digest>::with_capacity(sequence_count);
        let mut i: usize = 0usize;
        while i < sequence_count {
            mt_leafs.push(H::hash_varlen(&sequences[i]));
            i += 1usize;
        }

        // pad until power of two
        while mt_leafs.len() & (mt_leafs.len() - 1usize) != 0usize {
            mt_leafs.push(Digest::default());
        }

        // compute Merkle tree and return hash
        return <CpuParallel as MerkleTreeMaker<H>>::from_digests(&mt_leafs).get_root();
    }
}

fn main() {
    // get hash of tx kernel from standard-in
    let hash_of_kernel: Digest = tasm::tasm_io_read_stdin___digest();

    let witness: Box<RemovalRecordsIntegrityWitness> =
        RemovalRecordsIntegrityWitness::decode(&tasm::load_from_memory(BFieldElement::new(12000)))
            .unwrap();

    let kernel_mast_hash: Digest = witness.kernel.mast_hash();

    assert!(hash_of_kernel == kernel_mast_hash);

    return;

    // // Read transaction kernel hash
    // let _s5: Digest = tasm::tasm_io_read_stdin___digest();

    // Calling `decode` could load a struct, but from where?
    // If the struct is already present in memory, we probaly only need a pointer to it.
    // I guess, we can just assume that the struct we are loading is just placed on memory
    // pointer 1. So the `decode` function call just returns the pointer 1.

    // Once we have the struct pointer, we need to map the Rust field getters `obj.field` to
    // the `tasm_lib::field!(<Struct>::<field>)` macro. The `tasm_lib::field!` macro outputs
    // VM instructions, so a field getter can be replaced with the output of that macro.
    // Let's assume that we can get the output of that macro.
    // Then we need to *know* what type the field has, and be able to call methods on that type.
    // Some of these methods will already exist in `tasm-lib`. E.g. witness.kernel.mast_hash()
    // is present in `tasm-lib` through `library.import(Box::new(TransactionKernelMastHash));`
    // and *that* function just takes a pointer to the kernel, so with the above code in place,
    // we have support for that.
    // So to move forward, we only need to write a test that handles the most basic struct
    // and see if we can return (write to stdout) a pointer to the 2nd field of that struct.

    // If we can do that, from code that's valid Rust code, then we can handle structs well enough
    // for our current needs.

    // We should probably start testing this in isolation. So we should create a test with
    // a very simple struct and just output its field values to stdout.
    // Test procedure in VM:
    // - Rust field getters 'obj.field' maps to 'tasm_lib::field!(<Struct>::<field>)'
    // - then how do we read the field value onto the stack? We probably need
    // We can start the VM with this struct in memory but the native execution has to
    // have support for this through some helper functions. Those helper functions then probably
    // have to be called the same as

    // // 1. read and process witness data
    // let witness = *RemovalRecordsIntegrityWitness::decode(
    //     &secret_input.iter().skip(1).copied().collect_vec(),
    // )
    // .unwrap();
    // let hash_of_kernel = *Digest::decode(
    //     &public_input
    //         .individual_tokens
    //         .iter()
    //         .copied()
    //         .take(DIGEST_LENGTH)
    //         .rev()
    //         .collect_vec(),
    // )
    // .expect("Could not decode public input in Removal Records Integrity :: verify_raw");

    // // 1. read and process witness data
    // let memory_length = nondeterminism.ram.len() as u64;
    // let memory_vector = (1u64..memory_length)
    //     .map(BFieldElement::new)
    //     .map(|b| *nondeterminism.ram.get(&b).unwrap_or(&BFieldElement::new(0)))
    //     .collect_vec();
    // let witness = *RemovalRecordsIntegrityWitness::decode(&memory_vector).unwrap();

    // println!("first element of witness: {}", witness.encode()[0]);
    // println!("first element of kernel: {}", witness.kernel.encode()[0]);

    // // 2. assert that the kernel from the witness matches the hash in the public input
    // // now we can trust all data in kernel
    // assert_eq!(
    //     hash_of_kernel,
    //     witness.kernel.mast_hash(),
    //     "hash of kernel ({})\nwitness kernel ({})",
    //     hash_of_kernel,
    //     witness.kernel.mast_hash()
    // );

    // // 3. assert that the mutator set's MMRs in the witness match the kernel
    // // now we can trust all data in these MMRs as well
    // let mutator_set_hash = Hash::hash_pair(
    //     Hash::hash_pair(witness.aocl.bag_peaks(), witness.swbfi.bag_peaks()),
    //     Hash::hash_pair(witness.swbfa_hash, Digest::default()),
    // );
    // assert_eq!(witness.kernel.mutator_set_hash, mutator_set_hash);
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::marker::PhantomData;

    use anyhow::bail;
    use itertools::Itertools;
    use num::One;
    use num::Zero;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::RngCore;
    use rand::SeedableRng;
    use tasm_lib::structure::tasm_object::TasmObject;
    use tasm_lib::VmHasher;
    use triton_vm::twenty_first::amount::u32s::U32s;
    use triton_vm::twenty_first::shared_math::bfield_codec::BFieldCodec;
    use triton_vm::twenty_first::shared_math::other::log_2_floor;
    use triton_vm::twenty_first::shared_math::tip5::Digest;
    use triton_vm::twenty_first::shared_math::tip5::DIGEST_LENGTH;
    use triton_vm::twenty_first::util_types::algebraic_hasher::AlgebraicHasher;
    use triton_vm::twenty_first::util_types::algebraic_hasher::SpongeHasher;
    use triton_vm::twenty_first::util_types::mmr::mmr_accumulator::MmrAccumulator;
    use triton_vm::twenty_first::util_types::mmr::mmr_membership_proof::MmrMembershipProof;
    use triton_vm::twenty_first::util_types::mmr::mmr_trait::Mmr;
    use triton_vm::twenty_first::util_types::mmr::shared_basic::leaf_index_to_mt_index_and_peak_index;
    use triton_vm::BFieldElement;

    use crate::tests_and_benchmarks::ozk::ozk_parsing;
    use crate::tests_and_benchmarks::ozk::ozk_parsing::EntrypointLocation;
    use crate::tests_and_benchmarks::ozk::rust_shadows;
    use crate::tests_and_benchmarks::test_helpers::shared_test::execute_compiled_with_stack_and_ins_for_test;
    use crate::tests_and_benchmarks::test_helpers::shared_test::init_memory_from;

    use super::*;

    pub const NATIVE_COIN_TYPESCRIPT_DIGEST: Digest = Digest::new([
        BFieldElement::new(4843866011885844809),
        BFieldElement::new(16618866032559590857),
        BFieldElement::new(18247689143239181392),
        BFieldElement::new(7637465675240023996),
        BFieldElement::new(9104890367162237026),
    ]);

    #[test]
    #[ignore = "Doesn't work yet"]
    fn removal_record_integrity_partial_test() {
        let mut seed = [0u8; 32];
        seed[0] = 0xa0;
        seed[1] = 0xf1;
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let removal_record_integrity_witness: RemovalRecordsIntegrityWitness =
            pseudorandom_removal_record_integrity_witness(rng.gen());

        let stdin: Vec<BFieldElement> = removal_record_integrity_witness
            .kernel
            .mast_hash()
            .reversed()
            .values()
            .to_vec();

        let non_determinism =
            init_memory_from(&removal_record_integrity_witness, BFieldElement::new(12000));

        let expected_output = vec![];
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(stdin.clone(), non_determinism.clone());
        assert_eq!(native_output, expected_output);

        // Run test on Triton-VM
        let entrypoint_location =
            EntrypointLocation::disk("other", "removal_record_integrity_partial", "main");
        let test_program =
            ozk_parsing::compile_for_test(&entrypoint_location, crate::ast_types::ListType::Unsafe);
        println!("executing:\n{}", test_program.iter().join("\n"));
        let vm_output = execute_compiled_with_stack_and_ins_for_test(
            &test_program,
            vec![],
            stdin,
            non_determinism,
            0,
        )
        .unwrap();
    }

    #[derive(Clone, Debug, PartialEq, Eq, BFieldCodec)]
    pub struct Chunk {
        pub relative_indices: Vec<u32>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ChunkDictionary {
        // {chunk index => (MMR membership proof for the whole chunk to which index belongs, chunk value)}
        pub dictionary: HashMap<u64, (MmrMembershipProof<VmHasher>, Chunk)>,
    }

    impl ChunkDictionary {
        pub fn new(dictionary: HashMap<u64, (MmrMembershipProof<VmHasher>, Chunk)>) -> Self {
            Self { dictionary }
        }
    }

    impl BFieldCodec for ChunkDictionary {
        type Error = anyhow::Error;

        fn decode(sequence: &[BFieldElement]) -> anyhow::Result<Box<Self>> {
            if sequence.is_empty() {
                bail!("Cannot decode empty sequence of BFieldElements as ChunkDictionary");
            }
            let num_entries = sequence[0].value() as usize;
            let mut read_index = 1;
            let mut dictionary = HashMap::new();
            for _ in 0..num_entries {
                // read key
                let key_length = 2;
                if sequence.len() < read_index + key_length {
                    bail!(
                        "Cannot decode sequence of BFieldElements as ChunkDictionary: missing key"
                    );
                }
                let key = *u64::decode(&sequence[read_index..read_index + key_length])?;
                read_index += key_length;

                // read membership proof
                if sequence.len() <= read_index {
                    bail!("Cannot decode sequence of BFieldElements as ChunkDictionary: missing membership proof");
                }
                let memproof_length = sequence[read_index].value() as usize;
                read_index += 1;
                let membership_proof = *MmrMembershipProof::<VmHasher>::decode(
                    &sequence[read_index..read_index + memproof_length],
                )?;
                read_index += memproof_length;

                // read chunk
                if sequence.len() <= read_index {
                    bail!("Cannot decode sequence of BFieldElements as ChunkDictionary: missing chunk");
                }
                let chunk_length = sequence[read_index].value() as usize;
                read_index += 1;
                let chunk = *Chunk::decode(&sequence[read_index..read_index + chunk_length])?;
                read_index += chunk_length;

                dictionary.insert(key, (membership_proof, chunk));
            }

            Ok(Box::new(ChunkDictionary { dictionary }))
        }

        fn encode(&self) -> Vec<BFieldElement> {
            let mut string = vec![BFieldElement::new(self.dictionary.keys().len() as u64)];
            for key in self.dictionary.keys().sorted() {
                string.append(&mut key.encode());
                let mut membership_proof_encoded = self.dictionary[key].0.encode();
                string.push(BFieldElement::new(membership_proof_encoded.len() as u64));
                string.append(&mut membership_proof_encoded);
                let mut chunk_encoded = self.dictionary[key].1.encode();
                string.push(BFieldElement::new(chunk_encoded.len() as u64));
                string.append(&mut chunk_encoded);
            }
            string
        }

        fn static_length() -> Option<usize> {
            None
        }
    }

    pub fn pseudorandom_merkle_root_with_authentication_paths(
        seed: [u8; 32],
        tree_height: usize,
        leafs_and_indices: &[(Digest, u64)],
    ) -> (Digest, Vec<Vec<Digest>>) {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut nodes: HashMap<u64, Digest> = HashMap::new();

        // populate nodes dictionary with leafs
        for (leaf, index) in leafs_and_indices.iter() {
            nodes.insert(*index, *leaf);
        }

        // walk up tree layer by layer
        // when we need nodes not already present, sample at random
        let mut depth = tree_height + 1;
        while depth > 0 {
            let mut working_indices = nodes
                .keys()
                .copied()
                .filter(|i| {
                    (*i as u128) < (1u128 << (depth)) && (*i as u128) >= (1u128 << (depth - 1))
                })
                .collect_vec();
            working_indices.sort();
            working_indices.dedup();
            for wi in working_indices {
                let wi_odd = wi | 1;
                if nodes.get(&wi_odd).is_none() {
                    nodes.insert(wi_odd, rng.gen::<Digest>());
                }
                let wi_even = wi_odd ^ 1;
                if nodes.get(&wi_even).is_none() {
                    nodes.insert(wi_even, rng.gen::<Digest>());
                }
                let hash = VmHasher::hash_pair(nodes[&wi_even], nodes[&wi_odd]);
                nodes.insert(wi >> 1, hash);
            }
            depth -= 1;
        }

        // read out root
        let root = *nodes.get(&1).unwrap_or(&rng.gen());

        // read out paths
        let paths = leafs_and_indices
            .iter()
            .map(|(_d, i)| {
                (0..tree_height)
                    .map(|j| *nodes.get(&((*i >> j) ^ 1)).unwrap())
                    .collect_vec()
            })
            .collect_vec();

        (root, paths)
    }

    fn merkle_verify_tester_helper(
        root: Digest,
        index: u64,
        path: &[Digest],
        leaf: Digest,
    ) -> bool {
        let mut acc = leaf;
        for (shift, &p) in path.iter().enumerate() {
            if (index >> shift) & 1 == 1 {
                acc = VmHasher::hash_pair(p, acc);
            } else {
                acc = VmHasher::hash_pair(acc, p);
            }
        }
        acc == root
    }

    pub fn pseudorandom_mmra_with_mps(
        seed: [u8; 32],
        leafs: &[Digest],
    ) -> (MmrAccumulator<VmHasher>, Vec<MmrMembershipProof<VmHasher>>) {
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        // sample size of MMR
        let mut leaf_count = rng.next_u64();
        while leaf_count < leafs.len() as u64 {
            leaf_count = rng.next_u64();
        }
        let num_peaks = leaf_count.count_ones();

        // sample mmr leaf indices and calculate matching derived indices
        let leaf_indices = leafs
            .iter()
            .enumerate()
            .map(|(original_index, _leaf)| (original_index, rng.next_u64() % leaf_count))
            .map(|(original_index, mmr_index)| {
                let (mt_index, peak_index) =
                    leaf_index_to_mt_index_and_peak_index(mmr_index, leaf_count);
                (original_index, mmr_index, mt_index, peak_index)
            })
            .collect_vec();
        let leafs_and_indices = leafs.iter().copied().zip(leaf_indices).collect_vec();

        // iterate over all trees
        let mut peaks = vec![];
        let dummy_mp = MmrMembershipProof::new(0u64, vec![]);
        let mut mps: Vec<MmrMembershipProof<VmHasher>> =
            (0..leafs.len()).map(|_| dummy_mp.clone()).collect_vec();
        for tree in 0..num_peaks {
            // select all leafs and merkle tree indices for this tree
            let leafs_and_mt_indices = leafs_and_indices
                .iter()
                .copied()
                .filter(
                    |(_leaf, (_original_index, _mmr_index, _mt_index, peak_index))| {
                        *peak_index == tree
                    },
                )
                .map(
                    |(leaf, (original_index, _mmr_index, mt_index, _peak_index))| {
                        (leaf, mt_index, original_index)
                    },
                )
                .collect_vec();
            if leafs_and_mt_indices.is_empty() {
                peaks.push(rng.gen());
                continue;
            }

            // generate root and authentication paths
            let tree_height =
                log_2_floor(*leafs_and_mt_indices.first().map(|(_l, i, _o)| i).unwrap() as u128)
                    as usize;
            let (root, authentication_paths) = pseudorandom_merkle_root_with_authentication_paths(
                rng.gen(),
                tree_height,
                &leafs_and_mt_indices
                    .iter()
                    .map(|(l, i, _o)| (*l, *i))
                    .collect_vec(),
            );

            // sanity check
            for ((leaf, mt_index, _original_index), auth_path) in
                leafs_and_mt_indices.iter().zip(authentication_paths.iter())
            {
                assert!(merkle_verify_tester_helper(
                    root, *mt_index, auth_path, *leaf
                ));
            }

            // update peaks list
            peaks.push(root);

            // generate membership proof objects
            let membership_proofs = leafs_and_indices
                .iter()
                .copied()
                .filter(
                    |(_leaf, (_original_index, _mmr_index, _mt_index, peak_index))| {
                        *peak_index == tree
                    },
                )
                .zip(authentication_paths.into_iter())
                .map(
                    |(
                        (_leaf, (_original_index, mmr_index, _mt_index, _peak_index)),
                        authentication_path,
                    )| {
                        MmrMembershipProof::<VmHasher>::new(mmr_index, authentication_path)
                    },
                )
                .collect_vec();

            // sanity check: test if membership proofs agree with peaks list (up until now)
            let dummy_remainder: Vec<Digest> = (peaks.len()..num_peaks as usize)
                .map(|_| rng.gen())
                .collect_vec();
            let dummy_peaks = [peaks.clone(), dummy_remainder].concat();
            for (&(leaf, _mt_index, _original_index), mp) in
                leafs_and_mt_indices.iter().zip(membership_proofs.iter())
            {
                assert!(mp.verify(&dummy_peaks, leaf, leaf_count).0);
            }

            // collect membership proofs in vector, with indices matching those of the supplied leafs
            for ((_leaf, _mt_index, original_index), mp) in
                leafs_and_mt_indices.iter().zip(membership_proofs.iter())
            {
                mps[*original_index] = mp.clone();
            }
        }

        let mmra = MmrAccumulator::<VmHasher>::init(peaks, leaf_count);

        // sanity check
        for (&leaf, mp) in leafs.iter().zip(mps.iter()) {
            assert!(mp.verify(&mmra.get_peaks(), leaf, mmra.count_leaves()).0);
        }

        (mmra, mps)
    }

    #[derive(Clone, Debug, PartialEq, Eq, BFieldCodec)]

    pub struct Coin {
        pub type_script_hash: Digest,
        pub state: Vec<BFieldElement>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, BFieldCodec)]
    pub struct Utxo {
        pub lock_script_hash: Digest,
        pub coins: Vec<Coin>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, BFieldCodec, TasmObject)]
    pub struct MsMembershipProof {
        pub sender_randomness: Digest,
        pub receiver_preimage: Digest,
        pub auth_path_aocl: MmrMembershipProof<VmHasher>,
        pub target_chunks: ChunkDictionary,
    }

    pub const NUM_TRIALS: u32 = 45;

    #[derive(Debug, Clone, PartialEq, Eq, BFieldCodec)]
    pub struct AbsoluteIndexSet([u128; NUM_TRIALS as usize]);

    impl AbsoluteIndexSet {
        pub fn new(indices: &[u128; NUM_TRIALS as usize]) -> Self {
            Self(*indices)
        }

        pub fn sort_unstable(&mut self) {
            self.0.sort_unstable();
        }

        pub fn to_vec(&self) -> Vec<u128> {
            self.0.to_vec()
        }

        pub fn to_array(&self) -> [u128; NUM_TRIALS as usize] {
            self.0
        }

        pub fn to_array_mut(&mut self) -> &mut [u128; NUM_TRIALS as usize] {
            &mut self.0
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, BFieldCodec, TasmObject)]
    pub struct RemovalRecord {
        pub absolute_indices: AbsoluteIndexSet,
        pub target_chunks: ChunkDictionary,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, BFieldCodec)]
    pub struct AdditionRecord {
        pub canonical_commitment: Digest,
    }

    impl AdditionRecord {
        pub fn new(canonical_commitment: Digest) -> Self {
            Self {
                canonical_commitment,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, BFieldCodec)]
    pub struct PubScriptHashAndInput {
        pub pubscript_hash: Digest,
        pub pubscript_input: Vec<BFieldElement>,
    }
    pub const AMOUNT_SIZE_FOR_U32: usize = 4;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, BFieldCodec)]
    pub struct Amount(pub U32s<AMOUNT_SIZE_FOR_U32>);

    impl From<u32> for Amount {
        fn from(value: u32) -> Self {
            let mut limbs = [0u32; AMOUNT_SIZE_FOR_U32];
            limbs[0] = value;
            Amount(U32s::new(limbs))
        }
    }

    impl Amount {
        /// Return the element that corresponds to 1. Use in tests only.
        pub fn one() -> Amount {
            let mut values = [0u32; AMOUNT_SIZE_FOR_U32];
            values[0] = 1;
            Amount(U32s::new(values))
        }

        pub fn div_two(&mut self) {
            self.0.div_two();
        }

        pub fn to_native_coins(self) -> Vec<Coin> {
            let dictionary = vec![Coin {
                type_script_hash: NATIVE_COIN_TYPESCRIPT_DIGEST,
                state: self.encode(),
            }];
            dictionary
        }
    }

    pub fn pseudorandom_utxo(seed: [u8; 32]) -> Utxo {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        Utxo {
            lock_script_hash: rng.gen(),
            coins: Amount::from(rng.next_u32()).to_native_coins(),
        }
    }

    pub fn commit(
        item: Digest,
        sender_randomness: Digest,
        receiver_digest: Digest,
    ) -> AdditionRecord {
        let canonical_commitment = VmHasher::hash_pair(
            VmHasher::hash_pair(item, sender_randomness),
            receiver_digest,
        );

        AdditionRecord::new(canonical_commitment)
    }

    pub fn pseudorandom_mmr_membership_proof(seed: [u8; 32]) -> MmrMembershipProof<VmHasher> {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let leaf_index: u64 = rng.gen();
        let authentication_path: Vec<Digest> =
            (0..rng.gen_range(0..15)).map(|_| rng.gen()).collect_vec();
        MmrMembershipProof {
            leaf_index,
            authentication_path,
            _hasher: PhantomData,
        }
    }

    pub fn pseudorandom_chunk_dictionary(seed: [u8; 32]) -> ChunkDictionary {
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let mut dictionary = HashMap::new();
        for _ in 0..37 {
            let key = rng.next_u64();
            let authpath: Vec<Digest> = (0..rng.gen_range(0..6)).map(|_| rng.gen()).collect_vec();
            let chunk: Vec<u32> = (0..rng.gen_range(0..17)).map(|_| rng.gen()).collect_vec();

            dictionary.insert(
                key,
                (
                    MmrMembershipProof::new(key, authpath),
                    Chunk {
                        relative_indices: chunk,
                    },
                ),
            );
        }
        ChunkDictionary::new(dictionary)
    }

    pub fn pseudorandom_mutator_set_membership_proof(seed: [u8; 32]) -> MsMembershipProof {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let sender_randomness: Digest = rng.gen();
        let receiver_preimage: Digest = rng.gen();
        let auth_path_aocl: MmrMembershipProof<VmHasher> =
            pseudorandom_mmr_membership_proof(rng.gen());
        let target_chunks: ChunkDictionary = pseudorandom_chunk_dictionary(rng.gen());
        MsMembershipProof {
            sender_randomness,
            receiver_preimage,
            auth_path_aocl,
            target_chunks,
        }
    }

    pub fn pseudorandom_mmra(seed: [u8; 32]) -> MmrAccumulator<VmHasher> {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let leaf_count = rng.next_u32() as u64;
        let num_peaks = rng.next_u32() % 10;
        let peaks: Vec<Digest> = (0..num_peaks).map(|_| rng.gen()).collect_vec();
        MmrAccumulator::init(peaks, leaf_count)
    }

    pub fn pseudorandom_removal_record(seed: [u8; 32]) -> RemovalRecord {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let absolute_indices = AbsoluteIndexSet::new(
            &(0..NUM_TRIALS as usize)
                .map(|_| ((rng.next_u64() as u128) << 64) ^ rng.next_u64() as u128)
                .collect_vec()
                .try_into()
                .unwrap(),
        );
        let target_chunks = pseudorandom_chunk_dictionary(rng.gen::<[u8; 32]>());

        RemovalRecord {
            absolute_indices,
            target_chunks,
        }
    }

    pub fn pseudorandom_addition_record(seed: [u8; 32]) -> AdditionRecord {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let ar: Digest = rng.gen();
        AdditionRecord {
            canonical_commitment: ar,
        }
    }

    pub fn pseudorandom_pubscript_struct(seed: [u8; 32]) -> PubScriptHashAndInput {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let digest: Digest = rng.gen();
        let len = 10 + (rng.next_u32() % 50) as usize;
        let input: Vec<BFieldElement> = (0..len).map(|_| rng.gen()).collect_vec();
        PubScriptHashAndInput {
            pubscript_hash: digest,
            pubscript_input: input,
        }
    }

    pub fn pseudorandom_amount(seed: [u8; 32]) -> Amount {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let number: [u32; 4] = rng.gen();
        Amount(U32s::new(number))
    }

    pub fn pseudorandom_option<T>(seed: [u8; 32], thing: T) -> Option<T> {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        if rng.next_u32() % 2 == 0 {
            None
        } else {
            Some(thing)
        }
    }

    pub fn pseudorandom_transaction_kernel(
        seed: [u8; 32],
        num_inputs: usize,
        num_outputs: usize,
        num_pubscripts: usize,
    ) -> TransactionKernel {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let inputs = (0..num_inputs)
            .map(|_| pseudorandom_removal_record(rng.gen::<[u8; 32]>()))
            .collect_vec();
        let outputs = (0..num_outputs)
            .map(|_| pseudorandom_addition_record(rng.gen::<[u8; 32]>()))
            .collect_vec();
        let pubscripts = (0..num_pubscripts)
            .map(|_| pseudorandom_pubscript_struct(rng.gen::<[u8; 32]>()))
            .collect_vec();
        let fee = pseudorandom_amount(rng.gen::<[u8; 32]>());
        let coinbase = pseudorandom_option(rng.gen(), pseudorandom_amount(rng.gen::<[u8; 32]>()));
        let timestamp: BFieldElement = rng.gen();
        let mutator_set_hash: Digest = rng.gen();

        TransactionKernel {
            // inputs,
            // outputs,
            // pubscript_hashes_and_inputs: pubscripts,
            // fee,
            // coinbase,
            // timestamp,
            mutator_set_hash,
        }
    }

    pub const BATCH_SIZE: u32 = 8;
    pub const CHUNK_SIZE: u32 = 4096;
    pub const WINDOW_SIZE: u32 = 1048576;

    pub fn get_swbf_indices(
        item: Digest,
        sender_randomness: Digest,
        receiver_preimage: Digest,
        aocl_leaf_index: u64,
    ) -> [u128; NUM_TRIALS as usize] {
        let batch_index: u128 = aocl_leaf_index as u128 / BATCH_SIZE as u128;
        let batch_offset: u128 = batch_index * CHUNK_SIZE as u128;
        let leaf_index_bfes = aocl_leaf_index.encode();
        let input = [
            item.encode(),
            sender_randomness.encode(),
            receiver_preimage.encode(),
            leaf_index_bfes,
            // Pad according to spec
            vec![
                BFieldElement::one(),
                BFieldElement::zero(),
                BFieldElement::zero(),
            ],
        ]
        .concat();
        assert_eq!(
            input.len() % DIGEST_LENGTH,
            0,
            "Input to sponge must be a multiple digest length"
        );

        let mut sponge = <VmHasher as SpongeHasher>::init();
        VmHasher::absorb_repeatedly(&mut sponge, input.iter());
        VmHasher::sample_indices(&mut sponge, WINDOW_SIZE, NUM_TRIALS as usize)
            .into_iter()
            .map(|sample_index| sample_index as u128 + batch_offset)
            .collect_vec()
            .try_into()
            .unwrap()
    }

    pub fn pseudorandom_removal_record_integrity_witness(
        seed: [u8; 32],
    ) -> RemovalRecordsIntegrityWitness {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let num_inputs = 2;
        let num_outputs = 2;
        let num_pubscripts = 1;

        let input_utxos = (0..num_inputs)
            .map(|_| pseudorandom_utxo(rng.gen::<[u8; 32]>()))
            .collect_vec();
        let mut membership_proofs = (0..num_inputs)
            .map(|_| pseudorandom_mutator_set_membership_proof(rng.gen::<[u8; 32]>()))
            .collect_vec();
        let addition_records = input_utxos
            .iter()
            .zip(membership_proofs.iter())
            .map(|(utxo, msmp)| {
                commit(
                    VmHasher::hash(utxo),
                    msmp.sender_randomness,
                    msmp.receiver_preimage.hash::<VmHasher>(),
                )
            })
            .collect_vec();
        let canonical_commitments = addition_records
            .iter()
            .map(|ar| ar.canonical_commitment)
            .collect_vec();
        let (aocl, mmr_mps) =
            pseudorandom_mmra_with_mps(rng.gen::<[u8; 32]>(), &canonical_commitments);
        assert_eq!(num_inputs, mmr_mps.len());
        assert_eq!(num_inputs, canonical_commitments.len());

        for (mp, &cc) in mmr_mps.iter().zip_eq(canonical_commitments.iter()) {
            assert!(
                mp.verify(&aocl.get_peaks(), cc, aocl.count_leaves()).0,
                "Returned MPs must be valid for returned AOCL"
            );
        }

        for (ms_mp, mmr_mp) in membership_proofs.iter_mut().zip(mmr_mps.iter()) {
            ms_mp.auth_path_aocl = mmr_mp.clone();
        }
        let swbfi = pseudorandom_mmra(rng.gen::<[u8; 32]>());
        let swbfa_hash: Digest = rng.gen();
        let mut kernel =
            pseudorandom_transaction_kernel(rng.gen(), num_inputs, num_outputs, num_pubscripts);
        kernel.mutator_set_hash = VmHasher::hash_pair(
            VmHasher::hash_pair(aocl.bag_peaks(), swbfi.bag_peaks()),
            VmHasher::hash_pair(swbfa_hash, Digest::default()),
        );
        // kernel.inputs = input_utxos
        //     .iter()
        //     .zip(membership_proofs.iter())
        //     .map(|(utxo, msmp)| {
        //         (
        //             VmHasher::hash(utxo),
        //             msmp.sender_randomness,
        //             msmp.receiver_preimage,
        //             msmp.auth_path_aocl.leaf_index,
        //         )
        //     })
        //     .map(|(item, sr, rp, li)| get_swbf_indices(&item, &sr, &rp, li))
        //     .map(|ais| RemovalRecord {
        //         absolute_indices: AbsoluteIndexSet::new(&ais),
        //         target_chunks: pseudorandom_chunk_dictionary(rng.gen()),
        //     })
        //     .rev()
        //     .collect_vec();

        // let mut kernel_index_set_hashes = kernel
        //     .inputs
        //     .iter()
        //     .map(|rr| VmHasher::hash(&rr.absolute_indices))
        //     .collect_vec();
        // kernel_index_set_hashes.sort();

        RemovalRecordsIntegrityWitness {
            // input_utxos,
            // membership_proofs,
            // aocl,
            // swbfi,
            // swbfa_hash,
            kernel,
        }
    }
}
