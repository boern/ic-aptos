// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Error, Result};
use aptos_crypto::{Signature, ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use aptos_crypto_derive::{CryptoHasher, DeserializeKey, SerializeKey};
// use aptos_crypto::{
//     CryptoMaterialError, HashValue, ValidCryptoMaterial, ValidCryptoMaterialStringExt,
// };
// use aptos_crypto_derive::{CryptoHasher, DeserializeKey, SerializeKey};
// #[cfg(any(test, feature = "fuzzing"))]
// use proptest_derive::Arbitrary;
// use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use thiserror::Error;

use super::RawTransaction;
use aptos_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::HashValue,
    traits::CryptoMaterialError,
};
use candid::CandidType;
use move_core_types::account_address::AccountAddress;

/// Maximum number of signatures supported in `TransactionAuthenticator`,
/// across all `AccountAuthenticator`s included.
pub const MAX_NUM_OF_SIGS: usize = 32;

/// An error enum for issues related to transaction or account authentication.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{:?}", self)]
pub enum AuthenticationError {
    /// The number of signatures exceeds the maximum supported.
    MaxSignaturesExceeded,
}

/// Each transaction submitted to the Aptos blockchain contains a `TransactionAuthenticator`. During
/// transaction execution, the executor will check if every `AccountAuthenticator`'s signature on
/// the transaction hash is well-formed and whether the sha3 hash of the
/// `AccountAuthenticator`'s `AuthenticationKeyPreimage` matches the `AuthenticationKey` stored
/// under the participating signer's account address.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Single Ed25519 signature
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
}

impl TransactionAuthenticator {
    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }

    /// Create a (optional) multi-agent fee payer authenticator
    // pub fn fee_payer(
    //     sender: AccountAuthenticator,
    //     secondary_signer_addresses: Vec<AccountAddress>,
    //     secondary_signers: Vec<AccountAuthenticator>,
    //     fee_payer_address: AccountAddress,
    //     fee_payer_signer: AccountAuthenticator,
    // ) -> Self {
    //     Self::FeePayer {
    //         sender,
    //         secondary_signer_addresses,
    //         secondary_signers,
    //         fee_payer_address,
    //         fee_payer_signer,
    //     }
    // }

    /// Return Ok if all AccountAuthenticator's public keys match their signatures, Err otherwise
    pub fn verify(&self, raw_txn: &RawTransaction) -> Result<()> {
        // let num_sigs: usize = self.sender().number_of_signatures()
        //     + self
        //         .secondary_signers()
        //         .iter()
        //         .map(|auth| auth.number_of_signatures())
        //         .sum::<usize>();
        // if num_sigs > MAX_NUM_OF_SIGS {
        //     return Err(Error::new(AuthenticationError::MaxSignaturesExceeded));
        // }
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => signature.verify(raw_txn, public_key),
        }
    }

    pub fn sender(&self) -> AccountAuthenticator {
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => AccountAuthenticator::ed25519(public_key.clone(), signature.clone()),
        }
    }

    // pub fn secondary_signer_addresses(&self) -> Vec<AccountAddress> {
    //     match self {
    //         Self::Ed25519 { .. } | Self::MultiEd25519 { .. } | Self::SingleSender { .. } => {
    //             vec![]
    //         }
    //         Self::FeePayer {
    //             sender: _,
    //             secondary_signer_addresses,
    //             ..
    //         } => secondary_signer_addresses.to_vec(),
    //         Self::MultiAgent {
    //             sender: _,
    //             secondary_signer_addresses,
    //             ..
    //         } => secondary_signer_addresses.to_vec(),
    //     }
    // }

    // pub fn secondary_signers(&self) -> Vec<AccountAuthenticator> {
    //     match self {
    //         Self::Ed25519 { .. } | Self::MultiEd25519 { .. } | Self::SingleSender { .. } => {
    //             vec![]
    //         }
    //         Self::FeePayer {
    //             sender: _,
    //             secondary_signer_addresses: _,
    //             secondary_signers,
    //             ..
    //         } => secondary_signers.to_vec(),
    //         Self::MultiAgent {
    //             sender: _,
    //             secondary_signer_addresses: _,
    //             secondary_signers,
    //         } => secondary_signers.to_vec(),
    //     }
    // }

    // pub fn fee_payer_address(&self) -> Option<AccountAddress> {
    //     match self {
    //         Self::Ed25519 { .. }
    //         | Self::MultiEd25519 { .. }
    //         | Self::MultiAgent { .. }
    //         | Self::SingleSender { .. } => None,
    //         Self::FeePayer {
    //             sender: _,
    //             secondary_signer_addresses: _,
    //             secondary_signers: _,
    //             fee_payer_address,
    //             ..
    //         } => Some(*fee_payer_address),
    //     }
    // }

    // pub fn fee_payer_signer(&self) -> Option<AccountAuthenticator> {
    //     match self {
    //         Self::Ed25519 { .. }
    //         | Self::MultiEd25519 { .. }
    //         | Self::MultiAgent { .. }
    //         | Self::SingleSender { .. } => None,
    //         Self::FeePayer {
    //             sender: _,
    //             secondary_signer_addresses: _,
    //             secondary_signers: _,
    //             fee_payer_address: _,
    //             fee_payer_signer,
    //         } => Some(fee_payer_signer.clone()),
    //     }
    // }

    // pub fn all_signers(&self) -> Vec<AccountAuthenticator> {
    //     match self {
    //         // This is to ensure that any new TransactionAuthenticator variant must update this function.
    //         Self::Ed25519 { .. }
    //         | Self::MultiEd25519 { .. }
    //         | Self::MultiAgent { .. }
    //         | Self::FeePayer { .. }
    //         | Self::SingleSender { .. } => {
    //             let mut account_authenticators: Vec<AccountAuthenticator> = vec![];
    //             account_authenticators.push(self.sender());
    //             account_authenticators.extend(self.secondary_signers());
    //             if let Some(fee_payer) = self.fee_payer_signer() {
    //                 account_authenticators.push(fee_payer);
    //             }
    //             account_authenticators
    //         }
    //     }
    // }

    // pub fn to_single_key_authenticators(&self) -> Result<Vec<SingleKeyAuthenticator>> {
    //     let account_authenticators = self.all_signers();
    //     let mut single_key_authenticators: Vec<SingleKeyAuthenticator> =
    //         Vec::with_capacity(MAX_NUM_OF_SIGS);
    //     for account_authenticator in account_authenticators {
    //         match account_authenticator {
    //             AccountAuthenticator::Ed25519 {
    //                 public_key,
    //                 signature,
    //             } => {
    //                 let authenticator = SingleKeyAuthenticator {
    //                     public_key: AnyPublicKey::ed25519(public_key.clone()),
    //                     signature: AnySignature::ed25519(signature.clone()),
    //                 };
    //                 single_key_authenticators.push(authenticator);
    //             }
    //             AccountAuthenticator::MultiEd25519 {
    //                 public_key,
    //                 signature,
    //             } => {
    //                 let public_keys = MultiKey::from(public_key);
    //                 let signatures: Vec<AnySignature> = signature
    //                     .signatures()
    //                     .iter()
    //                     .map(|sig| AnySignature::ed25519(sig.clone()))
    //                     .collect();
    //                 let signatures_bitmap = aptos_bitvec::BitVec::from(signature.bitmap().to_vec());
    //                 let authenticator = MultiKeyAuthenticator {
    //                     public_keys,
    //                     signatures,
    //                     signatures_bitmap,
    //                 };
    //                 single_key_authenticators.extend(authenticator.to_single_key_authenticators()?);
    //             }
    //             AccountAuthenticator::SingleKey { authenticator } => {
    //                 single_key_authenticators.push(authenticator);
    //             }
    //             AccountAuthenticator::MultiKey { authenticator } => {
    //                 single_key_authenticators.extend(authenticator.to_single_key_authenticators()?);
    //             }
    //             AccountAuthenticator::NoAccountAuthenticator => {
    //                 //  This case adds no single key authenticators to the vector.
    //             }
    //         };
    //     }
    //     Ok(single_key_authenticators)
    // }
}

// impl fmt::Display for TransactionAuthenticator {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Ed25519 { .. } => {
//                 write!(
//                     f,
//                     "TransactionAuthenticator[scheme: Ed25519, sender: {}]",
//                     self.sender()
//                 )
//             }
//             Self::FeePayer {
//                 sender,
//                 secondary_signer_addresses,
//                 secondary_signers,
//                 fee_payer_address,
//                 fee_payer_signer,
//             } => {
//                 let mut sec_addrs: String = "".to_string();
//                 for sec_addr in secondary_signer_addresses {
//                     sec_addrs = format!("{}\n\t\t\t{:#?},", sec_addrs, sec_addr);
//                 }
//                 let mut sec_signers: String = "".to_string();
//                 for sec_signer in secondary_signers {
//                     sec_signers = format!("{}\n\t\t\t{:#?},", sec_signers, sec_signer);
//                 }
//                 write!(
//                     f,
//                     "TransactionAuthenticator[\n\
//                         \tscheme: MultiAgent, \n\
//                         \tsender: {}\n\
//                         \tsecondary signer addresses: {}\n\
//                         \tsecondary signers: {}\n\n
//                         \tfee payer address: {}\n\n
//                         \tfee payer signer: {}]",
//                     sender, sec_addrs, sec_signers, fee_payer_address, fee_payer_signer,
//                 )
//             }
//             Self::MultiEd25519 { .. } => {
//                 write!(
//                     f,
//                     "TransactionAuthenticator[scheme: MultiEd25519, sender: {}]",
//                     self.sender()
//                 )
//             }
//             Self::MultiAgent {
//                 sender,
//                 secondary_signer_addresses,
//                 secondary_signers,
//             } => {
//                 let mut sec_addrs: String = "".to_string();
//                 for sec_addr in secondary_signer_addresses {
//                     sec_addrs = format!("{}\n\t\t\t{:#?},", sec_addrs, sec_addr);
//                 }
//                 let mut sec_signers: String = "".to_string();
//                 for sec_signer in secondary_signers {
//                     sec_signers = format!("{}\n\t\t\t{:#?},", sec_signers, sec_signer);
//                 }
//                 write!(
//                     f,
//                     "TransactionAuthenticator[\n\
//                         \tscheme: MultiAgent, \n\
//                         \tsender: {}\n\
//                         \tsecondary signer addresses: {}\n\
//                         \tsecondary signers: {}]",
//                     sender, sec_addrs, sec_signers,
//                 )
//             }
//             Self::SingleSender { sender } => {
//                 write!(
//                     f,
//                     "TransactionAuthenticator[scheme: SingleSender, sender: {}]",
//                     sender
//                 )
//             }
//         }
//     }
// }

#[derive(Debug)]
#[repr(u8)]
pub enum Scheme {
    Ed25519 = 0,
    MultiEd25519 = 1,
    SingleKey = 2,
    MultiKey = 3,
    NoScheme = 250,
    /// Scheme identifier used to derive addresses (not the authentication key) of objects and
    /// resources accounts. This application serves to domain separate hashes. Without such
    /// separation, an adversary could create (and get a signer for) a these accounts
    /// when a their address matches matches an existing address of a MultiEd25519 wallet.
    /// Add new derived schemes below.
    DeriveAuid = 251,
    DeriveObjectAddressFromObject = 252,
    DeriveObjectAddressFromGuid = 253,
    DeriveObjectAddressFromSeed = 254,
    DeriveResourceAccountAddress = 255,
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Scheme::Ed25519 => "Ed25519",
            Scheme::MultiEd25519 => "MultiEd25519",
            Scheme::SingleKey => "SingleKey",
            Scheme::MultiKey => "MultiKey",
            Scheme::NoScheme => "NoScheme",
            Scheme::DeriveAuid => "DeriveAuid",
            Scheme::DeriveObjectAddressFromObject => "DeriveObjectAddressFromObject",
            Scheme::DeriveObjectAddressFromGuid => "DeriveObjectAddressFromGuid",
            Scheme::DeriveObjectAddressFromSeed => "DeriveObjectAddressFromSeed",
            Scheme::DeriveResourceAccountAddress => "DeriveResourceAccountAddress",
        };
        write!(f, "Scheme::{}", display)
    }
}

/// An `AccountAuthenticator` is an an abstraction of a signature scheme. It must know:
/// (1) How to check its signature against a message and public key
/// (2) How to convert its public key into an `AuthenticationKeyPreimage` structured as
/// (public_key | signature_scheme_id).
/// Each on-chain `Account` must store an `AuthenticationKey` (computed via a sha3 hash of `(public
/// key bytes | scheme as u8)`).
#[derive(Debug, Eq, PartialEq, Hash, Serialize)]
pub enum AccountAuthenticator {
    /// Ed25519 Single signature
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },

    NoAccountAuthenticator,
    // ... add more schemes here
}

impl AccountAuthenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme(&self) -> Scheme {
        match self {
            Self::Ed25519 { .. } => Scheme::Ed25519,

            Self::NoAccountAuthenticator => Scheme::NoScheme,
        }
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }

    /// Return the raw bytes of `self.public_key`
    pub fn public_key_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public_key, .. } => public_key.to_bytes().to_vec(),
            Self::NoAccountAuthenticator => vec![],
        }
    }

    /// Return the raw bytes of `self.signature`
    pub fn signature_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { signature, .. } => signature.to_bytes().to_vec(),
            Self::NoAccountAuthenticator => vec![],
        }
    }

    /// Return an authentication key derived from `self`'s public key and scheme id
    pub fn authentication_key(&self) -> Option<AuthenticationKey> {
        if let Self::NoAccountAuthenticator = self {
            None
        } else {
            Some(AuthenticationKey::from_preimage(
                self.public_key_bytes(),
                self.scheme(),
            ))
        }
    }

    /// Return the number of signatures included in this account authenticator.
    pub fn number_of_signatures(&self) -> usize {
        match self {
            Self::Ed25519 { .. } => 1,
            Self::NoAccountAuthenticator => 0,
        }
    }
}

/// A struct that represents an account authentication key. An account's address is the last 32
/// bytes of authentication key used to create it
#[derive(
    Clone,
    Copy,
    CryptoHasher,
    Debug,
    DeserializeKey,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeKey,
    CandidType,
)]
pub struct AuthenticationKey([u8; AuthenticationKey::LENGTH]);

impl AuthenticationKey {
    /// The number of bytes in an authentication key.
    pub const LENGTH: usize = AccountAddress::LENGTH;

    /// Create an authentication key from `bytes`
    pub const fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Return an authentication key that is impossible (in expectation) to sign for--useful for
    /// intentionally relinquishing control of an account.
    pub const fn zero() -> Self {
        Self([0; 32])
    }

    /// Create an authentication key from a preimage by taking its sha3 hash
    /// AuthenticationKey = (preimage | scheme_id)
    pub fn from_preimage(mut public_key_bytes: Vec<u8>, scheme: Scheme) -> AuthenticationKey {
        public_key_bytes.push(scheme as u8);
        AuthenticationKey::new(*HashValue::sha3_256_of(&public_key_bytes).as_ref())
    }

    /// Construct a preimage from a transaction-derived AUID as (txn_hash || auid_scheme_id)
    pub fn auid(mut txn_hash: Vec<u8>, auid_counter: u64) -> Self {
        txn_hash.extend(auid_counter.to_le_bytes().to_vec());
        Self::from_preimage(txn_hash, Scheme::DeriveAuid)
    }

    pub fn object_address_from_object(
        source: &AccountAddress,
        derive_from: &AccountAddress,
    ) -> AuthenticationKey {
        let mut bytes = source.to_vec();
        bytes.append(&mut derive_from.to_vec());
        Self::from_preimage(bytes, Scheme::DeriveObjectAddressFromObject)
    }

    /// Create an authentication key from an Ed25519 public key
    pub fn ed25519(public_key: &Ed25519PublicKey) -> AuthenticationKey {
        Self::from_preimage(public_key.to_bytes().to_vec(), Scheme::Ed25519)
    }

    /// Return the authentication key as an account address
    pub fn account_address(&self) -> AccountAddress {
        AccountAddress::new(self.0)
    }

    /// Construct a vector from this authentication key
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl ValidCryptoMaterial for AuthenticationKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl fmt::Display for AccountAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountAuthenticator[scheme id: {:?}, public key: {}, signature: {}]",
            self.scheme(),
            hex::encode(self.public_key_bytes()),
            hex::encode(self.signature_bytes())
        )
    }
}

impl TryFrom<&[u8]> for AuthenticationKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<AuthenticationKey, CryptoMaterialError> {
        if bytes.len() != Self::LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AuthenticationKey(addr))
    }
}

impl TryFrom<Vec<u8>> for AuthenticationKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: Vec<u8>) -> std::result::Result<AuthenticationKey, CryptoMaterialError> {
        AuthenticationKey::try_from(&bytes[..])
    }
}

impl FromStr for AuthenticationKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(
            !s.is_empty(),
            "authentication key string should not be empty.",
        );
        let bytes_out = ::hex::decode(s)?;
        let key = AuthenticationKey::try_from(bytes_out.as_slice())?;
        Ok(key)
    }
}

impl AsRef<[u8]> for AuthenticationKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::LowerHex for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Display for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}
