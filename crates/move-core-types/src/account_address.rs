// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use hex::FromHex;
use num::BigUint;
// use rand::{rngs::OsRng, Rng};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, fmt, str::FromStr};

/// A struct that represents an account address.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]

pub struct AccountAddress([u8; AccountAddress::LENGTH]);

impl AccountAddress {
    /// Hex address: 0x4
    pub const FOUR: Self = Self::get_hex_address_four();
    /// The number of bytes in an address.
    /// Default to 16 bytes, can be set to 20 bytes with address20 feature.
    pub const LENGTH: usize = 32;
    /// Max address: 0xff....
    pub const MAX_ADDRESS: Self = Self([0xFF; Self::LENGTH]);
    /// Hex address: 0x1
    pub const ONE: Self = Self::get_hex_address_one();
    /// Hex address: 0xA
    pub const TEN: Self = Self::get_hex_address_ten();
    /// Hex address: 0x3
    pub const THREE: Self = Self::get_hex_address_three();
    /// Hex address: 0x2
    pub const TWO: Self = Self::get_hex_address_two();
    /// Hex address: 0x0
    pub const ZERO: Self = Self([0u8; Self::LENGTH]);

    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        Self(address)
    }

    const fn get_hex_address_one() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 1u8;
        Self(addr)
    }

    const fn get_hex_address_two() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 2u8;
        Self(addr)
    }

    const fn get_hex_address_ten() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 10u8;
        Self(addr)
    }

    const fn get_hex_address_three() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 3u8;
        Self(addr)
    }

    const fn get_hex_address_four() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 4u8;
        Self(addr)
    }

    // pub fn random() -> Self {
    //     let mut rng = OsRng;
    //     let buf: [u8; Self::LENGTH] = rng.gen();
    //     Self(buf)
    // }

    /// Represent an account address in a way that is compliant with the v1 address
    /// standard. The standard is defined as part of AIP-40, read more here:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>
    ///
    /// In short, all special addresses MUST be represented in SHORT form, e.g.
    ///
    /// 0x1
    ///
    /// All other addresses MUST be represented in LONG form, e.g.
    ///
    /// 0x002098630cfad4734812fa37dc18d9b8d59242feabe49259e26318d468a99584
    ///
    /// For an explanation of what defines a "special" address, see `is_special`.
    ///
    /// All string representations of addresses MUST be prefixed with 0x.
    pub fn to_standard_string(&self) -> String {
        let suffix = if self.is_special() {
            self.short_str_lossless()
        } else {
            self.to_canonical_string()
        };
        format!("0x{}", suffix)
    }

    /// Returns whether the address is a "special" address. Addresses are considered
    /// special if the first 63 characters of the hex string are zero. In other words,
    /// an address is special if the first 31 bytes are zero and the last byte is
    /// smaller than than `0b10000` (16). In other words, special is defined as an address
    /// that matches the following regex: `^0x0{63}[0-9a-f]$`. In short form this means
    /// the addresses in the range from `0x0` to `0xf` (inclusive) are special.
    ///
    /// For more details see the v1 address standard defined as part of AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>
    pub fn is_special(&self) -> bool {
        self.0[..Self::LENGTH - 1].iter().all(|x| *x == 0) && self.0[Self::LENGTH - 1] < 0b10000
    }

    /// NOTE: For the purposes of displaying an address, using it in a response, or
    /// storing it at rest as a string, use `to_standard_string`.
    ///
    /// Return a canonical string representation of the address
    /// Addresses are hex-encoded lowercase values of length ADDRESS_LENGTH (16, 20, or 32 depending on the Move platform)
    /// e.g., 0000000000000000000000000000000a, *not* 0x0000000000000000000000000000000a, 0xa, or 0xA
    /// Note: this function is guaranteed to be stable, and this is suitable for use inside
    /// Move native functions or the VM.
    pub fn to_canonical_string(&self) -> String {
        hex::encode(self.0)
    }

    /// NOTE: For the purposes of displaying an address, using it in a response, or
    /// storing it at rest as a string, use `to_standard_string`.
    pub fn short_str_lossless(&self) -> String {
        let hex_str = hex::encode(self.0).trim_start_matches('0').to_string();
        if hex_str.is_empty() {
            "0".to_string()
        } else {
            hex_str
        }
    }

    pub fn to_big_uint(self) -> BigUint {
        BigUint::from_bytes_be(&self.into_bytes())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }

    /// NOTE: Where possible use from_str_strict or from_str instead.
    pub fn from_hex_literal(literal: &str) -> Result<Self, AccountAddressParseError> {
        if !literal.starts_with("0x") {
            return Err(AccountAddressParseError::LeadingZeroXRequired);
        }

        let hex_len = literal.len() - 2;

        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(&literal[2..]);
            AccountAddress::from_hex(hex_str)
        } else {
            AccountAddress::from_hex(&literal[2..])
        }
    }

    /// NOTE: For the purposes of displaying an address, using it in a response, or
    /// storing it at rest as a string, use `to_standard_string`.
    pub fn to_hex_literal(&self) -> String {
        format!("0x{}", self.short_str_lossless())
    }

    /// NOTE: Where possible use from_str_strict or from_str instead.
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|e| AccountAddressParseError::InvalidHexChars(format!("{:#}", e)))
            .map(Self)
    }

    /// NOTE: For the purposes of displaying an address, using it in a response, or
    /// storing it at rest as a string, use `to_standard_string`.
    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|e| AccountAddressParseError::InvalidHexChars(format!("{:#}", e)))
            .map(Self)
    }

    /// NOTE: This function has strict parsing behavior. For relaxed behavior, please use
    /// the `from_str` function. Where possible, prefer to use `from_str_strict`.
    ///
    /// Create an instance of AccountAddress by parsing a hex string representation.
    ///
    /// This function allows only the strictest formats defined by AIP-40. In short this
    /// means only the following formats are accepted:
    ///
    /// - LONG
    /// - SHORT for special addresses
    ///
    /// Where:
    ///
    /// - LONG is defined as 0x + 64 hex characters.
    /// - SHORT for special addresses is 0x0 to 0xf inclusive.
    ///
    /// This means the following are not accepted:
    ///
    /// - SHORT for non-special addresses.
    /// - Any address without a leading 0x.
    ///
    /// Learn more about the different address formats by reading AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>.
    pub fn from_str_strict(s: &str) -> Result<Self, AccountAddressParseError> {
        // Assert the string starts with 0x.
        if !s.starts_with("0x") {
            return Err(AccountAddressParseError::LeadingZeroXRequired);
        }

        let address = AccountAddress::from_str(s)?;

        // Check if the address is in LONG form. If it is not, this is only allowed for
        // special addresses, in which case we check it is in proper SHORT form.
        if s.len() != (AccountAddress::LENGTH * 2) + 2 {
            if !address.is_special() {
                return Err(AccountAddressParseError::LongFormRequiredUnlessSpecial);
            } else {
                // 0x + one hex char is the only valid SHORT form for special addresses.
                if s.len() != 3 {
                    return Err(AccountAddressParseError::InvalidPaddingZeroes);
                }
            }
        }

        Ok(address)
    }
}

impl AsRef<[u8]> for AccountAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for AccountAddress {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_standard_string())
    }
}

impl fmt::Debug for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::LowerHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::UpperHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl From<[u8; AccountAddress::LENGTH]> for AccountAddress {
    fn from(bytes: [u8; AccountAddress::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8]) -> Result<AccountAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte buffer into Address.
    fn try_from(bytes: Vec<u8>) -> Result<AccountAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

impl From<AccountAddress> for Vec<u8> {
    fn from(addr: AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<&AccountAddress> for Vec<u8> {
    fn from(addr: &AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<AccountAddress> for [u8; AccountAddress::LENGTH] {
    fn from(addr: AccountAddress) -> Self {
        addr.0
    }
}

impl From<&AccountAddress> for [u8; AccountAddress::LENGTH] {
    fn from(addr: &AccountAddress) -> Self {
        addr.0
    }
}

impl From<&AccountAddress> for String {
    fn from(addr: &AccountAddress) -> String {
        ::hex::encode(addr.as_ref())
    }
}

impl TryFrom<String> for AccountAddress {
    type Error = AccountAddressParseError;

    fn try_from(s: String) -> Result<AccountAddress, AccountAddressParseError> {
        Self::from_str(&s)
    }
}

impl FromStr for AccountAddress {
    type Err = AccountAddressParseError;

    /// NOTE: This function has relaxed parsing behavior. For strict behavior, please use
    /// the `from_str_strict` function. Where possible use `from_str_strict` rather than
    /// this function.
    ///
    /// Create an instance of AccountAddress by parsing a hex string representation.
    ///
    /// This function allows all formats defined by AIP-40. In short this means the
    /// following formats are accepted:
    ///
    /// - LONG, with or without leading 0x
    /// - SHORT, with or without leading 0x
    ///
    /// Where:
    ///
    /// - LONG is 64 hex characters.
    /// - SHORT is 1 to 63 hex characters inclusive.
    ///
    /// Learn more about the different address formats by reading AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>.
    fn from_str(s: &str) -> Result<Self, AccountAddressParseError> {
        if !s.starts_with("0x") {
            if s.is_empty() {
                return Err(AccountAddressParseError::TooShort);
            }
            AccountAddress::from_hex_literal(&format!("0x{}", s))
        } else {
            if s.len() == 2 {
                return Err(AccountAddressParseError::TooShort);
            }
            AccountAddress::from_hex_literal(s)
        }
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            AccountAddress::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "AccountAddress")]
            struct Value([u8; AccountAddress::LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(AccountAddress::new(value.0))
        }
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_hex().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("AccountAddress", &self.0)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AccountAddressParseError {
    #[error("AccountAddress data should be exactly 32 bytes long")]
    IncorrectNumberOfBytes,

    #[error("Hex characters are invalid: {0}")]
    InvalidHexChars(String),

    #[error("Hex string is too short, must be 1 to 64 chars long, excluding the leading 0x")]
    TooShort,

    #[error("Hex string is too long, must be 1 to 64 chars long, excluding the leading 0x")]
    TooLong,

    #[error("Hex string must start with a leading 0x")]
    LeadingZeroXRequired,

    #[error(
        "The given hex string is not a special address, it must be represented as 0x + 64 chars"
    )]
    LongFormRequiredUnlessSpecial,

    #[error("The given hex string is a special address not in LONG form, it must be 0x0 to 0xf without padding zeroes")]
    InvalidPaddingZeroes,
}
