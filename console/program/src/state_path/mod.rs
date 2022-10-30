// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

mod configuration;
pub use configuration::*;

mod header_leaf;
pub use header_leaf::*;

mod transaction_leaf;
pub use transaction_leaf::*;

pub mod transition_leaf;
pub use transition_leaf::*;

mod bytes;
mod parse;
mod serialize;

use snarkvm_console_network::prelude::*;
use snarkvm_console_types::Field;

#[derive(Clone, PartialEq, Eq)]
pub struct StatePath<N: Network> {
    /// The state root.
    state_root: N::StateRoot,
    /// The Merkle path for the block hash.
    block_path: BlockPath<N>,
    /// The block hash.
    block_hash: N::BlockHash,
    /// The previous block hash.
    previous_block_hash: N::BlockHash,
    /// The block header root.
    header_root: Field<N>,
    /// The Merkle path for the block header leaf.
    header_path: HeaderPath<N>,
    /// The block header leaf.
    header_leaf: HeaderLeaf<N>,
    /// The Merkle path for the transaction ID.
    transactions_path: TransactionsPath<N>,
    /// The transaction ID.
    transaction_id: N::TransactionID,
    /// The Merkle path for the transaction leaf.
    transaction_path: TransactionPath<N>,
    /// The transaction leaf.
    transaction_leaf: TransactionLeaf<N>,
    /// The Merkle path for the transition leaf.
    transition_path: TransitionPath<N>,
    /// The transition leaf.
    transition_leaf: TransitionLeaf<N>,
}

impl<N: Network> StatePath<N> {
    /// Initializes a new instance of `StatePath`.
    #[allow(clippy::too_many_arguments)]
    pub fn from(
        state_root: N::StateRoot,
        block_path: BlockPath<N>,
        block_hash: N::BlockHash,
        previous_block_hash: N::BlockHash,
        header_root: Field<N>,
        header_path: HeaderPath<N>,
        header_leaf: HeaderLeaf<N>,
        transactions_path: TransactionsPath<N>,
        transaction_id: N::TransactionID,
        transaction_path: TransactionPath<N>,
        transaction_leaf: TransactionLeaf<N>,
        transition_path: TransitionPath<N>,
        transition_leaf: TransitionLeaf<N>,
    ) -> Result<Self> {
        // Ensure the transition path is valid.
        ensure!(
            N::verify_merkle_path_bhp(&transition_path, &transaction_leaf.id(), &transition_leaf.to_bits_le()),
            "'{}' (an input or output ID) does not belong to '{}' (a function or transition)",
            transition_leaf.id(),
            transaction_leaf.id()
        );
        // Ensure the transaction path is valid.
        ensure!(
            N::verify_merkle_path_bhp(&transaction_path, &transaction_id, &transaction_leaf.to_bits_le()),
            "'{}' (a function or transition) does not belong to transaction '{transaction_id}'",
            transaction_leaf.id(),
        );
        // Ensure the transactions path is valid.
        ensure!(
            N::verify_merkle_path_bhp(&transactions_path, &header_leaf.id(), &transaction_id.to_bits_le()),
            "Transaction '{transaction_id}' does not belong to '{header_leaf}' (a header leaf)",
        );
        // Ensure the header path is valid.
        ensure!(
            N::verify_merkle_path_bhp(&header_path, &header_root, &header_leaf.to_bits_le()),
            "'{header_leaf}' (a header leaf) does not belong to '{block_hash}' (a block header)",
        );
        // Ensure the block hash is correct.
        let preimage = (*previous_block_hash).to_bits_le().into_iter().chain(header_root.to_bits_le().into_iter());
        ensure!(
            *block_hash == N::hash_bhp1024(&preimage.collect::<Vec<_>>())?,
            "Block hash '{block_hash}' is incorrect. Double-check the previous block hash and block header root."
        );
        // Ensure the state root is correct.
        ensure!(
            N::verify_merkle_path_bhp(&block_path, &state_root, &block_hash.to_bits_le()),
            "'{block_hash}' (a block hash) does not belong to '{state_root}' (a state root)",
        );
        // Return the state path.
        Ok(Self {
            state_root,
            block_path,
            block_hash,
            previous_block_hash,
            header_root,
            header_path,
            header_leaf,
            transactions_path,
            transaction_id,
            transaction_path,
            transaction_leaf,
            transition_path,
            transition_leaf,
        })
    }

    /// Returns the state root.
    pub const fn state_root(&self) -> N::StateRoot {
        self.state_root
    }

    /// Returns the block path.
    pub const fn block_path(&self) -> &BlockPath<N> {
        &self.block_path
    }

    /// Returns the block hash.
    pub const fn block_hash(&self) -> N::BlockHash {
        self.block_hash
    }

    /// Returns the previous block hash.
    pub const fn previous_block_hash(&self) -> N::BlockHash {
        self.previous_block_hash
    }

    /// Returns the block header root.
    pub const fn header_root(&self) -> &Field<N> {
        &self.header_root
    }

    /// Returns the header path.
    pub const fn header_path(&self) -> &HeaderPath<N> {
        &self.header_path
    }

    /// Returns the header leaf.
    pub const fn header_leaf(&self) -> &HeaderLeaf<N> {
        &self.header_leaf
    }

    /// Returns the transactions path.
    pub const fn transactions_path(&self) -> &TransactionsPath<N> {
        &self.transactions_path
    }

    /// Returns the transaction ID.
    pub const fn transaction_id(&self) -> &N::TransactionID {
        &self.transaction_id
    }

    /// Returns the Merkle path for the transaction leaf.
    pub const fn transaction_path(&self) -> &TransactionPath<N> {
        &self.transaction_path
    }

    /// Returns the transaction leaf.
    pub const fn transaction_leaf(&self) -> &TransactionLeaf<N> {
        &self.transaction_leaf
    }

    /// Returns the Merkle path for the transition leaf.
    pub const fn transition_path(&self) -> &TransitionPath<N> {
        &self.transition_path
    }

    /// Returns the transition leaf.
    pub const fn transition_leaf(&self) -> &TransitionLeaf<N> {
        &self.transition_leaf
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use snarkvm_console_network::prelude::TestRng;

    /// Randomly sample a state path.
    pub fn sample_state_path<N: Network>(rng: &mut TestRng) -> Result<StatePath<N>> {
        // Construct the transition path and transaction leaf.
        let transition_leaf = TransitionLeaf::new(0, 0, rng.gen(), rng.gen());
        let transition_tree: TransitionTree<N> = N::merkle_tree_bhp(&[transition_leaf.to_bits_le()])?;
        let transition_id = transition_tree.root();
        let transition_path = transition_tree.prove(0, &transition_leaf.to_bits_le())?;

        // Construct the transaction path and transaction leaf.
        let transaction_leaf = TransactionLeaf::new(rng.gen(), 0, *transition_id);
        let transaction_tree: TransactionTree<N> = N::merkle_tree_bhp(&[transaction_leaf.to_bits_le()])?;
        let transaction_id = *transaction_tree.root();
        let transaction_path = transaction_tree.prove(0, &transaction_leaf.to_bits_le())?;

        // Construct the transactions path.
        let transactions_tree: TransactionsTree<N> = N::merkle_tree_bhp(&[transaction_id.to_bits_le()])?;
        let transactions_root = transactions_tree.root();
        let transactions_path = transactions_tree.prove(0, &transaction_id.to_bits_le())?;

        // Construct the block header path.
        let header_leaf = HeaderLeaf::<N>::new(0, *transactions_root);
        let header_tree: HeaderTree<N> = N::merkle_tree_bhp(&[header_leaf.to_bits_le()])?;
        let header_root = header_tree.root();
        let header_path = header_tree.prove(0, &header_leaf.to_bits_le())?;

        let previous_block_hash: N::BlockHash = Field::<N>::rand(rng).into();
        let preimage = (*previous_block_hash).to_bits_le().into_iter().chain(header_root.to_bits_le().into_iter());
        let block_hash = N::hash_bhp1024(&preimage.collect::<Vec<_>>())?;

        // Construct the state root and block path.
        let block_tree: BlockTree<N> = N::merkle_tree_bhp(&[block_hash.to_bits_le()])?;
        let state_root = *block_tree.root();
        let block_path = block_tree.prove(0, &block_hash.to_bits_le())?;

        StatePath::<N>::from(
            state_root.into(),
            block_path,
            block_hash.into(),
            previous_block_hash,
            *header_root,
            header_path,
            header_leaf,
            transactions_path,
            transaction_id.into(),
            transaction_path,
            transaction_leaf,
            transition_path,
            transition_leaf,
        )
    }
}

//     #[derive(Clone)]
//     pub struct TestLedger<N: Network> {
//         /// The VM state.
//         vm: VM<N, ConsensusMemory<N>>,
//         /// The current block tree.
//         block_tree: BlockTree<N>,
//     }
//
//     impl TestLedger<CurrentNetwork> {
//         /// Initializes a new instance of the ledger.
//         pub fn new(rng: &mut TestRng) -> Result<Self> {
//             // Initialize the genesis block.
//             let genesis = crate::vm::test_helpers::sample_genesis_block(rng);
//
//             // Initialize the consensus store.
//             let store = ConsensusStore::<CurrentNetwork, ConsensusMemory<CurrentNetwork>>::open(None)?;
//             // Initialize a new VM.
//             let vm = VM::from(store)?;
//
//             // Initialize the ledger.
//             let mut ledger = Self { block_tree: CurrentNetwork::merkle_tree_bhp(&[])? };
//
//             // Add the genesis block.
//             ledger.add_next_block(&genesis)?;
//
//             // Return the ledger.
//             Ok(ledger)
//         }
//     }
//
//     impl<N: Network> TestLedger<N> {
//         /// Adds the given block as the next block in the chain.
//         pub fn add_next_block(&mut self, block: &Block<N>) -> Result<()> {
//             /* ATOMIC CODE SECTION */
//
//             // Add the block to the ledger. This code section executes atomically.
//             {
//                 let mut ledger = self.clone();
//
//                 // Update the blocks.
//                 ledger.block_tree.append(&[block.hash().to_bits_le()])?;
//                 ledger.vm.block_store().insert(*ledger.block_tree.root(), block)?;
//
//                 // Update the VM.
//                 for transaction in block.transactions().values() {
//                     ledger.vm.finalize(transaction)?;
//                 }
//
//                 *self = Self { vm: ledger.vm, block_tree: ledger.block_tree };
//             }
//
//             Ok(())
//         }
//
//         /// Returns the block for the given block height.
//         pub fn get_block(&self, height: u32) -> Result<Block<N>> {
//             // Retrieve the block hash.
//             let block_hash = match self.vm.block_store().get_block_hash(height)? {
//                 Some(block_hash) => block_hash,
//                 None => bail!("Block {height} does not exist in storage"),
//             };
//             // Retrieve the block.
//             match self.vm.block_store().get_block(&block_hash)? {
//                 Some(block) => Ok(block),
//                 None => bail!("Block {height} ('{block_hash}') does not exist in storage"),
//             }
//         }
//
//         /// Returns a state path for the given commitment.
//         pub fn to_state_path(&self, commitment: &Field<N>) -> Result<StatePath<N>> {
//             StatePath::new_commitment(&self.block_tree, self.vm.block_store(), commitment)
//         }
//     }
// }
