// Protocol Buffers - Google's data interchange format
// Copyright 2023 Google LLC.  All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://developers.google.com/open-source/licenses/bsd

//! Kernel-agnostic logic for the Rust Protobuf runtime that should not be
//! exposed to through the `protobuf` path but must be public for use by
//! generated code.

// Used by the proto! macro
pub use paste::paste;

use crate::map;
pub use crate::r#enum::Enum;
use crate::repeated;
pub use crate::ProtoStr;
use crate::Proxied;
pub use std::fmt::Debug;

#[cfg(all(bzl, cpp_kernel))]
#[path = "cpp.rs"]
pub mod runtime;
#[cfg(any(not(bzl), upb_kernel))]
#[path = "upb.rs"]
pub mod runtime;

// TODO: Temporarily re-export these symbols which are now under
// runtime under __internal directly since some external callers using it
// through __internal.
pub use runtime::{PtrAndLen, RawMap, RawMessage, RawRepeatedField};

/// Used to protect internal-only items from being used accidentally.
#[derive(Debug)]
pub struct Private;

/// A trait that is used as a subtrait of traits that we intend to be used but
/// not be implemented by users.
///
/// This is slightly less 'sealed' than the typical sealed trait pattern would
/// permit in other crates; this trait is intended to be available to crates
/// which were generated by protoc, but not to application code.
///
/// We require Sized as a supertrait, because we generally do not want our
/// traits to support trait objects.
pub trait SealedInternal: Sized {}

/// A trait used by the proto_eq() gtest macro.
pub trait MatcherEq: SealedInternal + Debug {
    fn matches(&self, o: &Self) -> bool;
}

/// Used by the proto! macro to get a default value for a repeated field.
pub fn get_repeated_default_value<T: repeated::ProxiedInRepeated + Default>(
    _: Private,
    _: repeated::RepeatedView<'_, T>,
) -> T {
    Default::default()
}

/// Used by the proto! macro to get a default value for a map field.
pub fn get_map_default_value<K: Proxied, V: map::ProxiedInMapValue<K> + Default>(
    _: Private,
    _: map::MapView<'_, K, V>,
) -> V {
    Default::default()
}

/// A function that is used to assert that the generated code is compatible with
/// the current runtime version. Right now a perfect/exact match with zero skew
/// is require, except any -prerelease suffixes are ignored as long it is
/// present on both. This may be relaxed in the future.
///
/// As the generated code is permitted to use unstable internal APIs, the protoc
/// used to generate the code must correspond to the runtime dependency. This
/// const fn is used to check at compile time that the right gencode is used
/// with the right runtime; if you are seeing this fail it means your protoc
/// version mismatches the Rust runtime crate version.
#[cfg(not(bzl))]
pub const fn assert_compatible_gencode_version(gencode_version: &'static str) {
    // Helper since str eq is not allowed in const context. In a future rust release
    // &str PartialEq will be allowed in const contexts and we can drop this.
    const fn const_str_eq(lhs: &str, rhs: &str) -> bool {
        let lhs = lhs.as_bytes();
        let rhs = rhs.as_bytes();
        if lhs.len() != rhs.len() {
            return false;
        }
        let mut i = 0;
        while i < lhs.len() {
            if lhs[i] != rhs[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    let runtime_version = env!("CARGO_PKG_VERSION");
    assert!(
        const_str_eq(gencode_version, runtime_version),
        "Gencode version is not compatible with runtime version",
    )
}

/// There is no need for gencode/runtime poison pill when running in bzl; the
/// gencode using the __internal mod which is not available to checked in
/// gencode; gencode built from source should always match.
#[cfg(bzl)]
pub const fn assert_compatible_gencode_version(_gencode_version: &'static str) {}
