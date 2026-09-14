// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Client-built Split-Block Bloom Filter blobs for `membership_match(field, {blob}, type=bloom)`.
//!
//! The blob is an **MBF1** envelope wrapping a Parquet Split-Block Bloom Filter (SBBF) body that is
//! bit-identical to Arrow C++'s `parquet::BlockSplitBloomFilter` and therefore to the parquet-format
//! `BloomFilter.md` spec. Building the filter client-side and shipping the compact blob lets large
//! membership sets pass the proxy gRPC receive limit that a raw `in [...]` list would exceed; the
//! server embeds the blob verbatim after validating the envelope and never rebuilds the filter.
//!
//! # MBF1 envelope layout (all integers little-endian)
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0 | 4 | magic `MBF1` |
//! | 4 | 2 | version = 1 |
//! | 6 | 2 | algo = 1 (parquet_sbbf_xxh64) |
//! | 8 | 8 | n_declared (informational) |
//! | 16 | 8 | fpr_declared (float64, informational) |
//! | 24 | 4 | num_blocks (body length must equal num_blocks × 32) |
//! | 28 | 1 | domains bitmask: `0x01` = int64, `0x02` = utf8 |
//! | 29 | 3 | reserved, must be 0 |
//! | 32 | … | body: SBBF blocks |
//!
//! The body is a power-of-two number of 32-byte blocks. Values hash with **XXH64 seed 0**: int64
//! values hash their 8-byte little-endian encoding, strings hash their raw UTF-8 bytes (Parquet
//! plain encoding for INT64 / BYTE_ARRAY). Block index is the multiply-shift reduction
//! `((hash >> 32) * num_blocks) >> 32`; within a block, word `i` in `0..7` sets bit
//! `1 << ((u32(hash) * SALT[i]) >> 27)` with the multiply wrapping mod 2³² and a logical shift.
//!
//! # Domains
//!
//! The two hash domains share one XXH64 output space, so the envelope records which domains were
//! inserted (`domains`) and a probe in an absent domain never matches. Build the blob from the same
//! value domain as the target field: integer fields hash int64, VARCHAR fields hash UTF-8.

use crate::v2::error::{Error, Result};

/// The 4-byte MBF1 envelope magic.
pub const BLOOM_MAGIC: &[u8; 4] = b"MBF1";
/// The MBF1 envelope version implemented here.
pub const BLOOM_VERSION: u16 = 1;
/// Identifies the parquet SBBF + XXH64 algorithm.
pub const ALGO_PARQUET_SBBF_XXH64: u16 = 1;
/// Size in bytes of the MBF1 envelope header.
pub const BLOOM_HEADER_SIZE: usize = 32;

/// Marks a filter that recorded int64 values (8-byte little-endian hash domain).
pub const DOMAIN_INT64: u8 = 1 << 0;
/// Marks a filter that recorded string values (raw UTF-8 hash domain).
pub const DOMAIN_UTF8: u8 = 1 << 1;

/// Size of one SBBF block (parquet-format spec).
pub const BYTES_PER_BLOCK: usize = 32;
const WORDS_PER_BLOCK: usize = 8;

/// Minimum filter body size, mirroring Arrow's `kMinimumBloomFilterBytes`.
pub const MIN_FILTER_BYTES: usize = 32;
/// Maximum filter body size, mirroring Arrow's `kMaximumBloomFilterBytes`.
pub const MAX_FILTER_BYTES: usize = 128 * 1024 * 1024;

/// Lower bound of the accepted false-positive rate.
pub const MIN_FPR: f64 = 0.0001;
/// Upper bound of the accepted false-positive rate.
pub const MAX_FPR: f64 = 0.05;
/// Recommended false-positive rate when a caller has no specific target. At this rate a body holds
/// roughly 0.72 members per byte, so a 64 MiB body holds ~48.6M members. Because bodies are powers
/// of two, a member count just past a tier boundary doubles the blob; raising `fpr` is usually the
/// cheaper fix.
pub const DEFAULT_FPR: f64 = 0.005;

/// The eight odd constants used to derive one bit position per word inside a block, fixed by the
/// parquet-format spec and mirrored from Arrow C++'s `BlockSplitBloomFilter::SALT`.
const SALT: [u32; WORDS_PER_BLOCK] = [
    0x47b6137b, 0x44974d91, 0x8824ad5b, 0xa2b7289d, 0x705495c7, 0x2df1424b, 0x9efc4947, 0x5c6bfb31,
];

/// Mirrors Arrow's `BlockSplitBloomFilter::OptimalNumOfBytes`: the classic blocked-bloom sizing
/// formula `m = -8n / ln(1 - fpp^(1/8))`, rounded up to the next power of two and clamped to
/// `[MIN_FILTER_BYTES, MAX_FILTER_BYTES]`. The result is always a power of two and a multiple of
/// [`BYTES_PER_BLOCK`].
fn optimal_num_of_bytes(ndv: u64, fpp: f64) -> usize {
    const MIN_BITS: f64 = (MIN_FILTER_BYTES as f64) * 8.0;
    const MAX_BITS: f64 = (MAX_FILTER_BYTES as f64) * 8.0;

    let m = -8.0 * ndv as f64 / (1.0 - fpp.powf(1.0 / 8.0)).ln();
    let num_bits = if m < 0.0 || m > MAX_BITS { MAX_BITS } else { m }.max(MIN_BITS) as u64;
    // Round up to the next power of two, then re-clamp.
    let num_bits = next_power_of_two(num_bits)
        .max(MIN_BITS as u64)
        .min(MAX_BITS as u64);
    (num_bits / 8) as usize
}

/// Returns the smallest power of two greater than or equal to `v` (`v >= 1`, `v <= 2^31`).
fn next_power_of_two(v: u64) -> u64 {
    let mut v = v;
    if v == 0 {
        return 1;
    }
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v + 1
}

/// Returns XXH64(seed 0) over `v`'s 8-byte little-endian encoding.
fn hash_int64(v: i64) -> u64 {
    xxhash_rust::xxh64::xxh64(&v.to_le_bytes(), 0)
}

/// Returns XXH64(seed 0) over the raw UTF-8 bytes of `s`.
fn hash_string(s: &str) -> u64 {
    xxhash_rust::xxh64::xxh64(s.as_bytes(), 0)
}

/// Reduces a hash to a block index via the multiply-shift scheme used by Arrow:
/// `((hash >> 32) * num_blocks) >> 32`. `num_blocks <= 2^22`, so the product cannot overflow.
fn block_index(hash: u64, num_blocks: u32) -> usize {
    (((hash >> 32) * num_blocks as u64) >> 32) as usize
}

///////////////////////////////////////////////////////////////////////////////
// BloomFilterBuilder
///////////////////////////////////////////////////////////////////////////////
/// Incrementally constructs a Split-Block Bloom Filter and serializes it into an MBF1 envelope.
///
/// The filter is sized from `n` and `fpr` alone — never from the values — so members can be added
/// in any order, duplicated, or streamed in one at a time. Not safe for concurrent use.
#[derive(Debug, Clone)]
pub struct BloomFilterBuilder {
    buf: Vec<u8>,
    num_blocks: u32,
    n_declared: u64,
    fpr: f64,
    domains: u8,
}

impl BloomFilterBuilder {
    /// Returns a builder sized for `n` distinct values at false-positive rate `fpr`. `fpr` must lie
    /// in `[MIN_FPR, MAX_FPR]` and be finite. The filter size follows Arrow's `OptimalNumOfBytes`
    /// (power-of-two bytes, clamped to `[MIN_FILTER_BYTES, MAX_FILTER_BYTES]`).
    pub fn new(n: u64, fpr: f64) -> Result<Self> {
        if fpr.is_nan() || fpr < MIN_FPR || fpr > MAX_FPR {
            return Err(Error::validation(
                "fpr".into(),
                format!("bloom filter fpr {fpr} out of range [{MIN_FPR}, {MAX_FPR}]"),
            ));
        }
        let num_bytes = optimal_num_of_bytes(n, fpr);
        let num_blocks = (num_bytes / BYTES_PER_BLOCK) as u32;
        Ok(Self {
            buf: vec![0u8; BLOOM_HEADER_SIZE + num_bytes],
            num_blocks,
            n_declared: n,
            fpr,
            domains: 0,
        })
    }

    /// Returns the number of 32-byte blocks in the filter body.
    pub fn num_blocks(&self) -> u32 {
        self.num_blocks
    }

    /// Inserts an int64 value (8-byte little-endian encoding) and returns the builder.
    pub fn add_int64(&mut self, v: i64) -> &mut Self {
        self.domains |= DOMAIN_INT64;
        self.add_hash(hash_int64(v));
        self
    }

    /// Inserts a string value (raw UTF-8 bytes) and returns the builder.
    pub fn add_string(&mut self, s: &str) -> &mut Self {
        self.domains |= DOMAIN_UTF8;
        self.add_hash(hash_string(s));
        self
    }

    /// Returns the value domains inserted so far (see [`DOMAIN_INT64`] / [`DOMAIN_UTF8`]). Zero
    /// means nothing was inserted.
    pub fn domains(&self) -> u8 {
        self.domains
    }

    fn add_hash(&mut self, h: u64) {
        let off = BLOOM_HEADER_SIZE + block_index(h, self.num_blocks) * BYTES_PER_BLOCK;
        let block = &mut self.buf[off..off + BYTES_PER_BLOCK];
        let key = h as u32;
        for (i, salt) in SALT.iter().enumerate() {
            let mask = 1u32 << ((key.wrapping_mul(*salt)) >> 27);
            let word = u32::from_le_bytes(block[i * 4..i * 4 + 4].try_into().unwrap());
            block[i * 4..i * 4 + 4].copy_from_slice(&(word | mask).to_le_bytes());
        }
    }

    /// Stamps the MBF1 header onto the filter and returns the envelope.
    ///
    /// This consumes the builder and returns its buffer directly, so building a filter never makes a
    /// second copy of the (possibly large) body. Call `build` once per filter; insert all members
    /// first.
    pub fn build(mut self) -> Vec<u8> {
        self.buf[0..4].copy_from_slice(BLOOM_MAGIC);
        self.buf[4..6].copy_from_slice(&BLOOM_VERSION.to_le_bytes());
        self.buf[6..8].copy_from_slice(&ALGO_PARQUET_SBBF_XXH64.to_le_bytes());
        self.buf[8..16].copy_from_slice(&self.n_declared.to_le_bytes());
        self.buf[16..24].copy_from_slice(&self.fpr.to_le_bytes());
        self.buf[24..28].copy_from_slice(&self.num_blocks.to_le_bytes());
        self.buf[28] = self.domains;
        self.buf
    }
}

/// Returns the exact number of bytes [`BloomFilterBuilder::build`] would produce for a filter
/// sized for `n` distinct values at false-positive rate `fpr`, without allocating the filter or
/// hashing any value. Callers can use it to reject an over-large filter before building it.
/// Returns an error if `fpr` is out of `[MIN_FPR, MAX_FPR]` or not finite.
pub fn estimate_size(n: u64, fpr: f64) -> Result<usize> {
    if fpr.is_nan() || fpr < MIN_FPR || fpr > MAX_FPR {
        return Err(Error::validation(
            "fpr".into(),
            format!("bloom filter fpr {fpr} out of range [{MIN_FPR}, {MAX_FPR}]"),
        ));
    }
    Ok(BLOOM_HEADER_SIZE + optimal_num_of_bytes(n, fpr))
}

/// Builds an MBF1 bloom blob from an int64 membership set at false-positive rate `fpr`. `fpr`
/// must lie in `[MIN_FPR, MAX_FPR]`; pass [`DEFAULT_FPR`] when you have no specific target.
///
/// The blob body is a power-of-two size clamped to `[MIN_FILTER_BYTES, MAX_FILTER_BYTES]`; it can
/// reach 128 MiB for very large sets, which may exceed the proxy's `maxMembershipFilterSize` (64 MiB
/// by default). Use [`estimate_size`] to check the exact blob size for the planned member count
/// before building large filters, and raise `fpr` if the count just misses a tier boundary.
pub fn bloom_filter_blob_int64(members: &[i64], fpr: f64) -> Result<Vec<u8>> {
    let mut builder = BloomFilterBuilder::new(members.len() as u64, fpr)?;
    for value in members {
        builder.add_int64(*value);
    }
    Ok(builder.build())
}

/// Builds an MBF1 bloom blob from a string membership set at false-positive rate `fpr`. `fpr`
/// must lie in `[MIN_FPR, MAX_FPR]`; pass [`DEFAULT_FPR`] when you have no specific target.
///
/// The blob body is a power-of-two size clamped to `[MIN_FILTER_BYTES, MAX_FILTER_BYTES]`; it can
/// reach 128 MiB for very large sets, which may exceed the proxy's `maxMembershipFilterSize` (64 MiB
/// by default). Use [`estimate_size`] to check the exact blob size for the planned member count
/// before building large filters, and raise `fpr` if the count just misses a tier boundary.
pub fn bloom_filter_blob_string(members: &[&str], fpr: f64) -> Result<Vec<u8>> {
    let mut builder = BloomFilterBuilder::new(members.len() as u64, fpr)?;
    for value in members {
        builder.add_string(value);
    }
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fpr_validation_rejects_out_of_range_nan_and_infinity() {
        for fpr in [
            MIN_FPR - 1e-5,
            MAX_FPR + 1e-5,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert!(
                BloomFilterBuilder::new(100, fpr).is_err(),
                "fpr {fpr} accepted"
            );
            assert!(
                estimate_size(100, fpr).is_err(),
                "estimate fpr {fpr} accepted"
            );
        }
        assert!(BloomFilterBuilder::new(100, MIN_FPR).is_ok());
        assert!(BloomFilterBuilder::new(100, MAX_FPR).is_ok());
    }

    #[test]
    fn empty_member_set_is_legal_and_matches_nothing() {
        let builder = BloomFilterBuilder::new(0, DEFAULT_FPR).unwrap();
        assert_eq!(builder.domains(), 0);
        assert_eq!(builder.build()[28], 0);
    }

    #[test]
    fn estimate_matches_built_length() {
        for (n, fpr) in [(0u64, DEFAULT_FPR), (100, 0.001), (10_000_000, DEFAULT_FPR)] {
            let builder = BloomFilterBuilder::new(n, fpr).unwrap();
            assert_eq!(estimate_size(n, fpr).unwrap(), builder.build().len());
        }
    }

    fn golden_cases() -> serde_json::Value {
        let json = include_str!("../../tests/v2/golden_vectors/bloom_golden_vectors.json");
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn blob_is_byte_identical_to_shared_golden_vectors() {
        let root = golden_cases();
        for case in root["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let n = case["n"].as_u64().unwrap();
            let fpr = case["fpr"].as_f64().unwrap();
            let mut builder = BloomFilterBuilder::new(n, fpr).unwrap();
            for value in case["int_values"].as_array().unwrap() {
                let value: i64 = value.as_str().unwrap().parse().unwrap();
                builder.add_int64(value);
            }
            for value in case["string_values"].as_array().unwrap() {
                builder.add_string(value.as_str().unwrap());
            }
            let blob = builder.build();
            let expected = hex::decode(case["blob_hex"].as_str().unwrap()).unwrap();
            assert_eq!(blob, expected, "case {name} mismatch");
        }
    }

    #[test]
    fn blob_matches_cpp_generated_100_int64_fixture() {
        let fixture = include_str!("../../tests/v2/golden_vectors/cpp_generated_100_int64.json");
        let fixture: serde_json::Value = serde_json::from_str(fixture).unwrap();
        let fpr = fixture["fpr"].as_f64().unwrap();
        let values: Vec<i64> = fixture["int_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        let blob = bloom_filter_blob_int64(&values, fpr).unwrap();
        let expected = hex::decode(fixture["blob_hex"].as_str().unwrap()).unwrap();
        assert_eq!(blob, expected);
    }
}
