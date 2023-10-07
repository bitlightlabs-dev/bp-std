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

use std::ops::Range;
use std::{iter, vec};

use bc::ScriptPubkey;
use indexmap::IndexMap;

use crate::{
    CompressedPk, Derive, DeriveCompr, DeriveScripts, DeriveSet, DeriveXOnly, DerivedScript,
    KeyOrigin, NormalIndex, TapDerivation, TaprootPk, Terminal, WPubkeyHash, XpubDerivable,
    XpubSpec,
};

pub trait Descriptor<K = XpubDerivable, V = ()>: DeriveScripts {
    type KeyIter<'k>: Iterator<Item = &'k K>
    where
        Self: 'k,
        K: 'k;

    type VarIter<'v>: Iterator<Item = &'v V>
    where
        Self: 'v,
        V: 'v;

    type XpubIter<'x>: Iterator<Item = &'x XpubSpec>
    where Self: 'x;

    fn keys(&self) -> Self::KeyIter<'_>;
    fn vars(&self) -> Self::VarIter<'_>;
    fn xpubs(&self) -> Self::XpubIter<'_>;

    fn compr_keyset(&self, terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin>;
    fn xonly_keyset(&self, terminal: Terminal) -> IndexMap<TaprootPk, TapDerivation>;
}

/*
pub trait KeyTranslate<K, V = ()>: Descriptor<K, V> {
    type Dest<K2>: Descriptor<K2, V>;
    fn translate<K2>(&self, f: impl Fn(K) -> K2) -> Self::Dest<K2>;
}

pub trait VarResolve<K, V>: Descriptor<K, V> {
    type Dest<V2>: Descriptor<K, V2>;
    fn resolve<V2>(&self, f: impl Fn(V) -> V2) -> Self::Dest<V2>;
}
 */

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate",))]
#[derive(Clone, Eq, PartialEq, Hash, Debug, From)]
pub struct Wpkh<K: DeriveCompr = XpubDerivable>(K);

impl<K: DeriveCompr> Wpkh<K> {
    pub fn as_key(&self) -> &K { &self.0 }
    pub fn into_key(self) -> K { self.0 }
}

impl<K: DeriveCompr> Derive<DerivedScript> for Wpkh<K> {
    #[inline]
    fn keychains(&self) -> Range<u8> { self.0.keychains() }

    fn derive(&self, keychain: u8, index: impl Into<NormalIndex>) -> DerivedScript {
        let key = self.0.derive(keychain, index);
        DerivedScript::Bare(ScriptPubkey::p2wpkh(WPubkeyHash::with(key)))
    }
}

impl<K: DeriveCompr> Descriptor<K> for Wpkh<K> {
    type KeyIter<'k> = iter::Once<&'k K> where Self: 'k, K: 'k;
    type VarIter<'v> = iter::Empty<&'v ()> where Self: 'v, (): 'v;
    type XpubIter<'x> = iter::Once<&'x XpubSpec> where Self: 'x;

    fn keys(&self) -> Self::KeyIter<'_> { iter::once(&self.0) }
    fn vars(&self) -> Self::VarIter<'_> { iter::empty() }
    fn xpubs(&self) -> Self::XpubIter<'_> { iter::once(self.0.xpub_spec()) }

    fn compr_keyset(&self, terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin> {
        let mut map = IndexMap::with_capacity(1);
        let key = self.0.derive(terminal.keychain, terminal.index);
        map.insert(key, KeyOrigin::with(self.0.xpub_spec().origin().clone(), terminal));
        map
    }

    fn xonly_keyset(&self, _terminal: Terminal) -> IndexMap<TaprootPk, TapDerivation> {
        IndexMap::new()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate",))]
#[derive(Clone, Eq, PartialEq, Hash, Debug, From)]
pub struct TrKey<K: DeriveXOnly = XpubDerivable>(K);

impl<K: DeriveXOnly> TrKey<K> {
    pub fn as_internal_key(&self) -> &K { &self.0 }
    pub fn into_internal_key(self) -> K { self.0 }
}

impl<K: DeriveXOnly> Derive<DerivedScript> for TrKey<K> {
    #[inline]
    fn keychains(&self) -> Range<u8> { self.0.keychains() }

    fn derive(&self, keychain: u8, index: impl Into<NormalIndex>) -> DerivedScript {
        let internal_key = self.0.derive(keychain, index);
        DerivedScript::TaprootKeyOnly(internal_key)
    }
}

impl<K: DeriveXOnly> Descriptor<K> for TrKey<K> {
    type KeyIter<'k> = iter::Once<&'k K> where Self: 'k, K: 'k;
    type VarIter<'v> = iter::Empty<&'v ()> where Self: 'v, (): 'v;
    type XpubIter<'x> = iter::Once<&'x XpubSpec> where Self: 'x;

    fn keys(&self) -> Self::KeyIter<'_> { iter::once(&self.0) }
    fn vars(&self) -> Self::VarIter<'_> { iter::empty() }
    fn xpubs(&self) -> Self::XpubIter<'_> { iter::once(self.0.xpub_spec()) }

    fn compr_keyset(&self, _terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin> {
        IndexMap::new()
    }

    fn xonly_keyset(&self, terminal: Terminal) -> IndexMap<TaprootPk, TapDerivation> {
        let mut map = IndexMap::with_capacity(1);
        let key = self.0.derive(terminal.keychain, terminal.index);
        map.insert(
            key.into(),
            TapDerivation::with_internal_pk(self.0.xpub_spec().origin().clone(), terminal),
        );
        map
    }
}

/*
pub struct TrScript<K: DeriveXOnly> {
    internal_key: K,
    tap_tree: TapTree<Policy<K>>,
}
*/

#[derive(Clone, Eq, PartialEq, Hash, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(
        crate = "serde_crate",
        rename_all = "camelCase",
        bound(
            serialize = "S::Compr: serde::Serialize, S::XOnly: serde::Serialize",
            deserialize = "S::Compr: serde::Deserialize<'de>, S::XOnly: serde::Deserialize<'de>"
        )
    )
)]
pub enum DescriptorStd<S: DeriveSet = XpubDerivable> {
    #[from]
    Wpkh(Wpkh<S::Compr>),

    #[from]
    TrKey(TrKey<S::XOnly>),
}

impl<S: DeriveSet> Derive<DerivedScript> for DescriptorStd<S> {
    fn keychains(&self) -> Range<u8> {
        match self {
            DescriptorStd::Wpkh(d) => d.keychains(),
            DescriptorStd::TrKey(d) => d.keychains(),
        }
    }

    fn derive(&self, keychain: u8, index: impl Into<NormalIndex>) -> DerivedScript {
        match self {
            DescriptorStd::Wpkh(d) => d.derive(keychain, index),
            DescriptorStd::TrKey(d) => d.derive(keychain, index),
        }
    }
}

impl<K: DeriveSet<Compr = K, XOnly = K> + DeriveCompr + DeriveXOnly> Descriptor<K>
    for DescriptorStd<K>
where Self: Derive<DerivedScript>
{
    type KeyIter<'k> = vec::IntoIter<&'k K> where Self: 'k, K: 'k;
    type VarIter<'v> = iter::Empty<&'v ()> where Self: 'v, (): 'v;
    type XpubIter<'x> = vec::IntoIter<&'x XpubSpec> where Self: 'x;

    fn keys(&self) -> Self::KeyIter<'_> {
        match self {
            DescriptorStd::Wpkh(d) => d.keys().collect::<Vec<_>>(),
            DescriptorStd::TrKey(d) => d.keys().collect::<Vec<_>>(),
        }
        .into_iter()
    }

    fn vars(&self) -> Self::VarIter<'_> { iter::empty() }

    fn xpubs(&self) -> Self::XpubIter<'_> {
        match self {
            DescriptorStd::Wpkh(d) => d.xpubs().collect::<Vec<_>>(),
            DescriptorStd::TrKey(d) => d.xpubs().collect::<Vec<_>>(),
        }
        .into_iter()
    }

    fn compr_keyset(&self, terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin> {
        match self {
            DescriptorStd::Wpkh(d) => d.compr_keyset(terminal),
            DescriptorStd::TrKey(d) => d.compr_keyset(terminal),
        }
    }

    fn xonly_keyset(&self, terminal: Terminal) -> IndexMap<TaprootPk, TapDerivation> {
        match self {
            DescriptorStd::Wpkh(d) => d.xonly_keyset(terminal),
            DescriptorStd::TrKey(d) => d.xonly_keyset(terminal),
        }
    }
}
