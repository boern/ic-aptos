pub mod authenticator;
pub mod script;
pub mod user_transaction_context;

use super::chain_id::ChainId;
use anyhow::Result;
use aptos_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use aptos_crypto::hash::{CryptoHash, HashValue};
use aptos_crypto::traits::CryptoMaterialError;
use aptos_crypto_derive::{BCSCryptoHash, CryptoHasher};
use authenticator::TransactionAuthenticator;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::{AbortLocation, StatusCode};
use once_cell::sync::OnceCell;
pub use script::{EntryFunction, Script};

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::{hash::Hash, ops::Deref, sync::atomic::AtomicU64};

pub type Version = u64; // Height - also used for MVCC in StateDB
pub type AtomicVersion = AtomicU64;

/// RawTransaction is the portion of a transaction that a client signs.
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash,
)]
pub struct RawTransaction {
    /// Sender's address.
    sender: AccountAddress,

    /// Sequence number of this transaction. This must match the sequence number
    /// stored in the sender's account at the time the transaction executes.
    sequence_number: u64,

    /// The transaction payload, e.g., a script to execute.
    payload: TransactionPayload,

    /// Maximal total gas to spend for this transaction.
    max_gas_amount: u64,

    /// Price to be paid per gas unit.
    gas_unit_price: u64,

    /// Expiration timestamp for this transaction, represented
    /// as seconds from the Unix Epoch. If the current blockchain timestamp
    /// is greater than or equal to this time, then the transaction has
    /// expired and will be discarded. This can be set to a large value far
    /// in the future to indicate that a transaction does not expire.
    expiration_timestamp_secs: u64,

    /// Chain ID of the Aptos network this transaction is intended for.
    chain_id: ChainId,
}

impl RawTransaction {
    /// Create a new `RawTransaction` with a payload.
    ///
    /// It can be either to publish a module, to execute a script, or to issue a writeset
    /// transaction.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        payload: TransactionPayload,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawTransaction` with a script.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(
        sender: AccountAddress,
        sequence_number: u64,
        script: Script,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Script(script),
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawTransaction` with an entry function.
    pub fn new_entry_function(
        sender: AccountAddress,
        sequence_number: u64,
        entry_function: EntryFunction,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::EntryFunction(entry_function),
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Signs the given `RawTransaction`. Note that this consumes the `RawTransaction` and turns it
    /// into a `SignatureCheckedTransaction`.
    ///
    /// For a transaction that has just been signed, its signature is expected to be valid.
    // pub fn sign(
    //     self,
    //     private_key: &Ed25519PrivateKey,
    //     public_key: Ed25519PublicKey,
    // ) -> Result<SignatureCheckedTransaction> {
    //     let signature = private_key.sign(&self)?;
    //     Ok(SignatureCheckedTransaction(SignedTransaction::new(
    //         self, public_key, signature,
    //     )))
    // }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    /// Return the sender of this transaction.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    /// Return the signing message for creating transaction signature.
    pub fn signing_message(&self) -> Result<Vec<u8>, CryptoMaterialError> {
        // signing_message(self)
        todo!()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RawTransactionWithData {
    MultiAgent {
        raw_txn: RawTransaction,
        secondary_signer_addresses: Vec<AccountAddress>,
    },
    MultiAgentWithFeePayer {
        raw_txn: RawTransaction,
        secondary_signer_addresses: Vec<AccountAddress>,
        fee_payer_address: AccountAddress,
    },
}

impl RawTransactionWithData {
    // pub fn new_fee_payer(
    //     raw_txn: RawTransaction,
    //     secondary_signer_addresses: Vec<AccountAddress>,
    //     fee_payer_address: AccountAddress,
    // ) -> Self {
    //     Self::MultiAgentWithFeePayer {
    //         raw_txn,
    //         secondary_signer_addresses,
    //         fee_payer_address,
    //     }
    // }

    // pub fn new_multi_agent(
    //     raw_txn: RawTransaction,
    //     secondary_signer_addresses: Vec<AccountAddress>,
    // ) -> Self {
    //     Self::MultiAgent {
    //         raw_txn,
    //         secondary_signer_addresses,
    //     }
    // }
}

/// Marks payload as deprecated. We need to use it to ensure serialization or
/// deserialization is not broken.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeprecatedPayload {
    // Used because 'analyze_serde_formats' complains with "Please avoid 0-sized containers".
    dummy_value: u64,
}

/// Different kinds of transactions.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A transaction that executes code.
    Script(Script),
    /// Deprecated.
    ModuleBundle(DeprecatedPayload),
    /// A transaction that executes an existing entry function published on-chain.
    EntryFunction(EntryFunction),
    // / A multisig transaction that allows an owner of a multisig account to execute a pre-approved
    // / transaction as the multisig account.
    // Multisig(Multisig),
}

impl TransactionPayload {
    pub fn into_entry_function(self) -> EntryFunction {
        match self {
            Self::EntryFunction(f) => f,
            payload => panic!("Expected EntryFunction(_) payload, found: {:#?}", payload),
        }
    }
}

// /// Two different kinds of WriteSet transactions.
// #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
// pub enum WriteSetPayload {
//     /// Directly passing in the WriteSet.
//     Direct(ChangeSet),
//     /// Generate the WriteSet by running a script.
//     Script {
//         /// Execute the script as the designated signer.
//         execute_as: AccountAddress,
//         /// Script body that gets executed.
//         script: Script,
//     },
// }

// impl WriteSetPayload {
//     pub fn should_trigger_reconfiguration_by_default(&self) -> bool {
//         match self {
//             Self::Direct(_) => true,
//             Self::Script { .. } => false,
//         }
//     }
// }

/// A transaction that has been signed.
///
/// A `SignedTransaction` is a single transaction that can be atomically executed. Clients submit
/// these to validator nodes, and the validator and executor submits these to the VM.
///
/// **IMPORTANT:** The signature of a `SignedTransaction` is not guaranteed to be verified. For a
/// transaction whose signature is statically guaranteed to be verified, see
/// [`SignatureCheckedTransaction`].
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Public key and signature to authenticate
    authenticator: TransactionAuthenticator,

    /// A cached size of the raw transaction bytes.
    /// Prevents serializing the same transaction multiple times to determine size.
    #[serde(skip)]
    raw_txn_size: OnceCell<usize>,

    /// A cached size of the authenticator.
    /// Prevents serializing the same authenticator multiple times to determine size.
    #[serde(skip)]
    authenticator_size: OnceCell<usize>,

    /// A cached hash of the transaction.
    #[serde(skip)]
    committed_hash: OnceCell<HashValue>,
}

/// PartialEq ignores the cached OnceCell fields that may or may not be initialized.
impl PartialEq for SignedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.raw_txn == other.raw_txn && self.authenticator == other.authenticator
    }
}

/// A transaction for which the signature has been verified. Created by
/// [`SignedTransaction::check_signature`] and [`RawTransaction::sign`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureCheckedTransaction(SignedTransaction);

impl SignatureCheckedTransaction {
    /// Returns the `SignedTransaction` within.
    pub fn into_inner(self) -> SignedTransaction {
        self.0
    }

    /// Returns the `RawTransaction` within.
    pub fn into_raw_transaction(self) -> RawTransaction {
        self.0.into_raw_transaction()
    }
}

impl Deref for SignatureCheckedTransaction {
    type Target = SignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for SignedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SignedTransaction {{ \n \
             {{ raw_txn: {:#?}, \n \
             authenticator: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.authenticator
        )
    }
}

impl SignedTransaction {
    pub fn new_signed_transaction(
        raw_txn: RawTransaction,
        authenticator: TransactionAuthenticator,
    ) -> SignedTransaction {
        SignedTransaction {
            raw_txn,
            authenticator,
            raw_txn_size: OnceCell::new(),
            authenticator_size: OnceCell::new(),
            committed_hash: OnceCell::new(),
        }
    }

    pub fn new(
        raw_txn: RawTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> SignedTransaction {
        let authenticator = TransactionAuthenticator::ed25519(public_key, signature);
        SignedTransaction {
            raw_txn,
            authenticator,
            raw_txn_size: OnceCell::new(),
            authenticator_size: OnceCell::new(),
            committed_hash: OnceCell::new(),
        }
    }

    pub fn authenticator(&self) -> TransactionAuthenticator {
        self.authenticator.clone()
    }

    pub fn authenticator_ref(&self) -> &TransactionAuthenticator {
        &self.authenticator
    }

    pub fn sender(&self) -> AccountAddress {
        self.raw_txn.sender
    }

    pub fn into_raw_transaction(self) -> RawTransaction {
        self.raw_txn
    }

    pub fn raw_transaction_ref(&self) -> &RawTransaction {
        &self.raw_txn
    }

    pub fn sequence_number(&self) -> u64 {
        self.raw_txn.sequence_number
    }

    pub fn chain_id(&self) -> ChainId {
        self.raw_txn.chain_id
    }

    pub fn payload(&self) -> &TransactionPayload {
        &self.raw_txn.payload
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.raw_txn.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> u64 {
        self.raw_txn.gas_unit_price
    }

    pub fn expiration_timestamp_secs(&self) -> u64 {
        self.raw_txn.expiration_timestamp_secs
    }

    pub fn raw_txn_bytes_len(&self) -> usize {
        *self.raw_txn_size.get_or_init(|| {
            bcs::serialized_size(&self.raw_txn).expect("Unable to serialize RawTransaction")
        })
    }

    pub fn txn_bytes_len(&self) -> usize {
        let authenticator_size = *self.authenticator_size.get_or_init(|| {
            bcs::serialized_size(&self.authenticator)
                .expect("Unable to serialize TransactionAuthenticator")
        });
        self.raw_txn_bytes_len() + authenticator_size
    }

    // / Checks that the signature of given transaction. Returns `Ok(SignatureCheckedTransaction)` if
    // / the signature is valid.
    pub fn check_signature(self) -> Result<SignatureCheckedTransaction> {
        self.authenticator.verify(&self.raw_txn)?;
        Ok(SignatureCheckedTransaction(self))
    }

    pub fn verify_signature(&self) -> Result<()> {
        self.authenticator.verify(&self.raw_txn)?;
        Ok(())
    }

    /// Returns the hash when the transaction is committed onchain.
    pub fn committed_hash(&self) -> HashValue {
        *self
            .committed_hash
            .get_or_init(|| Transaction::UserTransaction(self.clone()).hash())
    }
}

/// The status of VM execution, which contains more detailed failure info
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
// #[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub enum ExecutionStatus {
    Success,
    OutOfGas,
    MoveAbort {
        location: AbortLocation,
        code: u64,
        info: Option<AbortInfo>,
    },
    ExecutionFailure {
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError(Option<StatusCode>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
// #[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct AbortInfo {
    pub reason_name: String,
    pub description: String,
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, CryptoHasher, BCSCryptoHash, Debug, Eq, PartialEq, Serialize, Deserialize)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum TransactionInfo {
    V0(TransactionInfoV0),
}

// impl TransactionInfo {
//     pub fn new(
//         transaction_hash: HashValue,
//         state_change_hash: HashValue,
//         event_root_hash: HashValue,
//         state_checkpoint_hash: Option<HashValue>,
//         gas_used: u64,
//         status: ExecutionStatus,
//     ) -> Self {
//         Self::V0(TransactionInfoV0::new(
//             transaction_hash,
//             state_change_hash,
//             event_root_hash,
//             state_checkpoint_hash,
//             gas_used,
//             status,
//         ))
//     }

//     #[cfg(any(test, feature = "fuzzing"))]
//     pub fn new_placeholder(
//         gas_used: u64,
//         state_checkpoint_hash: Option<HashValue>,
//         status: ExecutionStatus,
//     ) -> Self {
//         Self::new(
//             HashValue::default(),
//             HashValue::default(),
//             HashValue::default(),
//             state_checkpoint_hash,
//             gas_used,
//             status,
//         )
//     }

//     #[cfg(any(test, feature = "fuzzing"))]
//     fn dummy() -> Self {
//         Self::new(
//             HashValue::default(),
//             HashValue::default(),
//             HashValue::default(),
//             None,
//             0,
//             ExecutionStatus::Success,
//         )
//     }
// }

// impl Deref for TransactionInfo {
//     type Target = TransactionInfoV0;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::V0(txn_info) => txn_info,
//         }
//     }
// }

#[derive(Clone, CryptoHasher, BCSCryptoHash, Debug, Eq, PartialEq, Serialize, Deserialize)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TransactionInfoV0 {
    /// The amount of gas used.
    gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    status: ExecutionStatus,

    /// The hash of this transaction.
    transaction_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The hash value summarizing all changes caused to the world state by this transaction.
    /// i.e. hash of the output write set.
    state_change_hash: HashValue,

    /// The root hash of the Sparse Merkle Tree describing the world state at the end of this
    /// transaction. Depending on the protocol configuration, this can be generated periodical
    /// only, like per block.
    state_checkpoint_hash: Option<HashValue>,

    /// Potentially summarizes all evicted items from state. Always `None` for now.
    state_cemetery_hash: Option<HashValue>,
}

/// `Transaction` will be the transaction type used internally in the aptos node to represent the
/// transaction to be processed and persisted.
///
/// We suppress the clippy warning here as we would expect most of the transaction to be user
/// transaction.
#[allow(clippy::large_enum_variant)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub enum Transaction {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    /// TODO: We need to rename SignedTransaction to SignedUserTransaction, as well as all the other
    ///       transaction types we had in our codebase.
    UserTransaction(SignedTransaction),
    // / Transaction that applies a WriteSet to the current storage, it's applied manually via aptos-db-bootstrapper.
    // GenesisTransaction(WriteSetPayload),

    // /// Transaction to update the block metadata resource at the beginning of a block,
    // /// when on-chain randomness is disabled.
    // BlockMetadata(BlockMetadata),

    // /// Transaction to let the executor update the global state tree and record the root hash
    // /// in the TransactionInfo
    // /// The hash value inside is unique block id which can generate unique hash of state checkpoint transaction
    // StateCheckpoint(HashValue),

    // /// Transaction that only proposed by a validator mainly to update on-chain configs.
    // ValidatorTransaction(ValidatorTransaction),

    // /// Transaction to update the block metadata resource at the beginning of a block,
    // /// when on-chain randomness is enabled.
    // BlockMetadataExt(BlockMetadataExt),

    // /// Transaction to let the executor update the global state tree and record the root hash
    // /// in the TransactionInfo
    // /// The hash value inside is unique block id which can generate unique hash of state checkpoint transaction
    // /// Replaces StateCheckpoint, with optionally having more data.
    // BlockEpilogue(BlockEpiloguePayload),
}
