// Modern, minimalistic & standard-compliant cold wallet library.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2020-2023 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2023 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2020-2023 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bp::LockTime;

/// Error constructing timelock from the provided value.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display("invalid timelock value")]
pub struct InvalidTimelock;

/// Value for a transaction `nTimeLock` field which is guaranteed to represent a
/// UNIX timestamp which is always either 0 or a greater than or equal to
/// 500000000.
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct LockTimestamp(u32);

impl From<LockTimestamp> for u32 {
    fn from(lock_timestamp: LockTimestamp) -> Self { lock_timestamp.into_consensus() }
}

impl TryFrom<u32> for LockTimestamp {
    type Error = InvalidTimelock;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        LockTime::from_consensus(value).try_into()
    }
}

impl TryFrom<LockTime> for LockTimestamp {
    type Error = InvalidTimelock;

    fn try_from(lock_time: LockTime) -> Result<Self, Self::Error> {
        if !lock_time.is_time_based() {
            return Err(InvalidTimelock);
        }
        Ok(Self(lock_time.into_consensus()))
    }
}

impl LockTimestamp {
    /// Create zero time lock
    #[inline]
    pub fn anytime() -> Self { Self(0) }

    /// Creates absolute time lock valid since the current timestamp.
    pub fn since_now() -> Self {
        let now = Utc::now();
        LockTimestamp::from_unix_timestamp(now.timestamp() as u32)
            .expect("we are too far in the future")
    }

    /// Creates absolute time lock with the given UNIX timestamp value.
    ///
    /// Timestamp value must be greater or equal to `0x1DCD6500`, otherwise
    /// `None` is returned.
    #[inline]
    pub fn from_unix_timestamp(timestamp: u32) -> Option<Self> {
        if timestamp < LOCKTIME_THRESHOLD {
            None
        } else {
            Some(Self(timestamp))
        }
    }

    /// Converts into full u32 representation of `nSeq` value as it is
    /// serialized in bitcoin transaction.
    #[inline]
    pub fn into_consensus(self) -> u32 { self.0 }

    /// Converts into [`LockTime`] representation.
    #[inline]
    pub fn into_locktime(self) -> LockTime { self.into() }
}

impl Display for LockTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("time(")?;
        Display::fmt(&self.0, f)?;
        f.write_str(")")
    }
}

impl FromStr for LockTimestamp {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        if s == "0" || s == "none" {
            Ok(LockTimestamp::anytime())
        } else if s.starts_with("time(") && s.ends_with(')') {
            let no = s[5..].trim_end_matches(')').parse()?;
            LockTimestamp::try_from(no).map_err(|_| ParseError::InvalidTimestamp(no))
        } else {
            Err(ParseError::InvalidDescriptor(s))
        }
    }
}

/// Value for a transaction `nTimeLock` field which is guaranteed to represent a
/// block height number which is always less than 500000000.
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct LockHeight(u32);

impl From<LockHeight> for u32 {
    fn from(lock_height: LockHeight) -> Self { lock_height.into_consensus() }
}

impl TryFrom<u32> for LockHeight {
    type Error = InvalidTimelock;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        LockTime::from_consensus(value).try_into()
    }
}

impl TryFrom<LockTime> for LockHeight {
    type Error = InvalidTimelock;

    fn try_from(lock_time: LockTime) -> Result<Self, Self::Error> {
        if !lock_time.is_height_based() {
            return Err(InvalidTimelock);
        }
        Ok(Self(lock_time.into_consensus()))
    }
}

impl LockHeight {
    /// Create zero time lock
    #[inline]
    pub fn anytime() -> Self { Self(0) }

    /// Creates absolute time lock with the given block height.
    ///
    /// Block height must be strictly less than `0x1DCD6500`, otherwise
    /// `None` is returned.
    #[inline]
    pub fn from_height(height: u32) -> Option<Self> {
        if height < LOCKTIME_THRESHOLD {
            Some(Self(height))
        } else {
            None
        }
    }

    /// Converts into full u32 representation of `nSeq` value as it is
    /// serialized in bitcoin transaction.
    #[inline]
    pub fn into_consensus(self) -> u32 { self.0 }

    /// Converts into [`LockTime`] representation.
    #[inline]
    pub fn into_locktime(self) -> LockTime { self.into() }
}

impl Display for LockHeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("height(")?;
        Display::fmt(&self.0, f)?;
        f.write_str(")")
    }
}

impl FromStr for LockHeight {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        if s == "0" || s == "none" {
            Ok(LockHeight::anytime())
        } else if s.starts_with("height(") && s.ends_with(')') {
            let no = s[7..].trim_end_matches(')').parse()?;
            LockHeight::try_from(no).map_err(|_| ParseError::InvalidHeight(no))
        } else {
            Err(ParseError::InvalidDescriptor(s))
        }
    }
}
