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

//! Client-built roaring bitmap blobs for `membership_match(field, {blob}, type=roaring)`.
//!
//! The blob is an **MRB1** envelope wrapping the portable 64-bit Roaring format that segcore
//! decodes. A membership set rides the wire as a compact bitmap instead of a raw `in [...]` list
//! and, unlike a bloom filter, with no false positives — which is why the server also permits it in
//! delete expressions.
//!
//! # MRB1 envelope layout (all integers little-endian)
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0 | 4 | magic `MRB1` |
//! | 4 | 2 | version = 1 |
//! | 6 | 2 | format = 1 (portable_roaring64) |
//! | 8 | 8 | cardinality (checked against the decoded bitmap) |
//! | 16 | 8 | body_len (must equal len(blob) - 32) |
//! | 24 | 8 | reserved, must be 0 |
//! | 32 | … | body: portable Roaring64 |
//!
//! The body is the portable 64-bit Roaring serialization: a `u64` high-container count, then for
//! each high group (in ascending high-key order) the high key and a portable Roaring32 blob. Each
//! Roaring32 blob starts with a serial cookie (12347 when any container is a run container, else
//! 12346), the container metadata, an optional offset table, and the container bodies (array /
//! bitmap / run). Signed members are sign-extended into the uint64 key space before insertion, so
//! `-1` becomes `0xffffffffffffffff`, not `0xff`.

use crate::v2::error::{Error, Result};

/// The 4-byte MRB1 envelope magic.
pub const ROARING_MAGIC: &[u8; 4] = b"MRB1";
/// The MRB1 envelope version implemented here.
pub const ROARING_VERSION: u16 = 1;
/// Identifies the portable Roaring64 format.
pub const FORMAT_PORTABLE_ROARING64: u16 = 1;
/// Size in bytes of the MRB1 envelope header.
pub const ROARING_HEADER_SIZE: usize = 32;

/// Bounds an untrusted portable body.
pub const MAX_BODY_BYTES: u64 = 128 * 1024 * 1024;
/// Bounds the number of separately allocated high containers.
pub const MAX_HIGH_CONTAINER_COUNT: u64 = 1 << 18;
/// Bounds the estimated decoded size of one bitmap.
pub const MAX_ESTIMATED_DECODED_BYTES: u64 = 64 * 1024 * 1024;
/// Covers the inline high-container object and its vector entry overhead.
pub const ESTIMATED_HIGH_CONTAINER_OVERHEAD_BYTES: u64 = 128;
/// Covers decoded container pointers, keys/typecodes and container header overhead.
pub const ESTIMATED_LOW_CONTAINER_OVERHEAD_BYTES: u64 = 64;

const SERIAL_COOKIE: u16 = 12347;
const SERIAL_COOKIE_NO_RUN_CONTAINER: u32 = 12346;
const MAX_ARRAY_CARDINALITY: u32 = 4096;
const BITMAP_BODY_SIZE: usize = 8192;
/// The run-versus-bitmap tie-break compares against the reference implementation's in-memory
/// bitmap container size (32 bytes of container struct plus 8192 bytes of payload), not the 8192
/// bytes a bitmap container occupies on the wire. Using 8192 here would flip the choice for
/// containers holding 2048..2055 runs and stop matching the Go SDK's output.
const BITMAP_CONTAINER_SIZE_IN_MEMORY: u64 = 8224;

/// Container encoding kinds within a Roaring32 blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    Array,
    Bitmap,
    Run,
}

/// One 16-bit container: the `[begin, end)` slice of the normalized key vector, so planning
/// copies nothing.
#[derive(Debug, Clone)]
struct ContainerPlan {
    key: u16,
    begin: usize,
    end: usize,
    num_runs: u32,
    kind: ContainerKind,
    body_size: u32,
}

/// One high-32 group and its container list.
#[derive(Debug, Clone)]
struct GroupPlan {
    key: u32,
    first: usize,
    count: usize,
    has_run: bool,
    has_offsets: bool,
    cookie_size: u32,
    run_bitmap_size: u32,
    size: u64,
}

/// The fully-planned layout of an MRB1 body.
#[derive(Debug)]
struct Layout {
    groups: Vec<GroupPlan>,
    containers: Vec<ContainerPlan>,
    /// An empty member set still writes the 8-byte high-container count, so the body is never 0.
    body_length: u64,
}

///////////////////////////////////////////////////////////////////////////////
// RoaringBitmapStats
///////////////////////////////////////////////////////////////////////////////
/// Statistics about a built (or planned) roaring bitmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoaringBitmapStats {
    cardinality: u64,
    high_container_count: u64,
    low_container_count: u64,
    array_containers: u64,
    bitmap_containers: u64,
    run_containers: u64,
    body_length: u64,
    estimated_decoded_size: u64,
}

impl RoaringBitmapStats {
    /// Returns the number of distinct members.
    pub fn cardinality(&self) -> u64 {
        self.cardinality
    }

    /// Returns the number of distinct high-32 groups.
    pub fn high_container_count(&self) -> u64 {
        self.high_container_count
    }

    /// Returns the number of distinct 16-bit low containers.
    pub fn low_container_count(&self) -> u64 {
        self.low_container_count
    }

    /// Returns the number of array containers.
    pub fn array_containers(&self) -> u64 {
        self.array_containers
    }

    /// Returns the number of bitmap containers.
    pub fn bitmap_containers(&self) -> u64 {
        self.bitmap_containers
    }

    /// Returns the number of run containers.
    pub fn run_containers(&self) -> u64 {
        self.run_containers
    }

    /// Returns the length in bytes of the MRB1 body.
    pub fn body_length(&self) -> u64 {
        self.body_length
    }

    /// Returns the estimated decoded size: body length plus per-container overhead.
    pub fn estimated_decoded_size(&self) -> u64 {
        self.estimated_decoded_size
    }
}

///////////////////////////////////////////////////////////////////////////////
// RoaringBitmapBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builds an MRB1 roaring bitmap blob from a membership set.
///
/// Members are inserted in any order and deduplicated at [`RoaringBitmapBuilder::build`] time;
/// signed members are sign-extended into the uint64 key space. Like the Go and C++ references,
/// members are buffered in memory until `build()` — the portable Roaring64 format requires sorting
/// the full set, so this builder does not reduce peak RAM versus materializing a value list. Not
/// safe for concurrent use.
#[derive(Debug, Clone, Default)]
pub struct RoaringBitmapBuilder {
    keys: Vec<u64>,
    normalized: bool,
}

impl RoaringBitmapBuilder {
    /// Returns an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an int64 member. Signed values preserve their two's-complement bits when mapped
    /// into the uint64 key space: `INT8(-1)` becomes `0xffffffffffffffff`, not `0xff`.
    pub fn add_int64(&mut self, value: i64) -> &mut Self {
        self.keys.push(value as u64);
        self.normalized = false;
        self
    }

    /// Inserts many int64 members at once.
    pub fn add_int64s(&mut self, values: &[i64]) -> &mut Self {
        self.keys.extend(values.iter().map(|&v| v as u64));
        self.normalized = false;
        self
    }

    fn normalize(&mut self) {
        if !self.normalized {
            self.keys.sort_unstable();
            self.keys.dedup();
            self.normalized = true;
        }
    }

    /// Returns the number of distinct members.
    pub fn cardinality(&mut self) -> u64 {
        self.normalize();
        self.keys.len() as u64
    }

    /// Returns statistics about the bitmap that would be built. Runs the same guards as
    /// [`RoaringBitmapBuilder::build`] (bucket pre-checks and limit checks) first, so an oversized
    /// member set is rejected without allocating a container plan per value.
    pub fn stats(&mut self) -> Result<RoaringBitmapStats> {
        self.normalize();
        check_bucket_limits(count_buckets(&self.keys))?;
        let stats = stats_of(self.keys.len() as u64, &plan_layout(&self.keys));
        check_limits(&stats)?;
        Ok(stats)
    }

    /// Validates that this member set is within the server's limits.
    pub fn validate(&mut self) -> Result<()> {
        self.normalize();
        check_bucket_limits(count_buckets(&self.keys))?;
        check_limits(&stats_of(self.keys.len() as u64, &plan_layout(&self.keys)))
    }

    /// Builds the MRB1 blob. Duplicate members are deduplicated; ordering does not matter.
    pub fn build(&mut self) -> Result<Vec<u8>> {
        self.normalize();
        check_bucket_limits(count_buckets(&self.keys))?;
        let layout = plan_layout(&self.keys);
        let stats = stats_of(self.keys.len() as u64, &layout);
        check_limits(&stats)?;

        let mut blob = vec![0u8; ROARING_HEADER_SIZE + layout.body_length as usize];
        blob[0..4].copy_from_slice(ROARING_MAGIC);
        blob[4..6].copy_from_slice(&ROARING_VERSION.to_le_bytes());
        blob[6..8].copy_from_slice(&FORMAT_PORTABLE_ROARING64.to_le_bytes());
        blob[8..16].copy_from_slice(&stats.cardinality.to_le_bytes());
        blob[16..24].copy_from_slice(&layout.body_length.to_le_bytes());
        write_body(&mut blob[ROARING_HEADER_SIZE..], &self.keys, &layout);
        Ok(blob)
    }
}

/// Builds an MRB1 blob from an int64 membership set, deduplicating members.
pub fn roaring_bitmap_blob(members: &[i64]) -> Result<Vec<u8>> {
    let mut builder = RoaringBitmapBuilder::new();
    builder.add_int64s(members);
    builder.build()
}

/// Groups the normalized keys into high groups and 16-bit containers, picks each container's
/// encoding, and sizes the whole body without allocating it.
fn plan_layout(keys: &[u64]) -> Layout {
    let mut layout = Layout {
        groups: Vec::new(),
        containers: Vec::new(),
        body_length: 8,
    };
    let mut index = 0;
    while index < keys.len() {
        let high = (keys[index] >> 32) as u32;
        let mut group = GroupPlan {
            key: high,
            first: layout.containers.len(),
            count: 0,
            has_run: false,
            has_offsets: false,
            cookie_size: 0,
            run_bitmap_size: 0,
            size: 0,
        };
        let mut bodies: u64 = 0;

        while index < keys.len() && (keys[index] >> 32) as u32 == high {
            let container_key = (keys[index] >> 16) as u16;
            let mut container = ContainerPlan {
                key: container_key,
                begin: index,
                end: 0,
                num_runs: 0,
                kind: ContainerKind::Array,
                body_size: 0,
            };

            // Count maximal consecutive runs while walking the container's values.
            let mut previous: u32 = 0;
            let mut first = true;
            while index < keys.len()
                && (keys[index] >> 32) as u32 == high
                && (keys[index] >> 16) as u16 == container_key
            {
                let value = (keys[index] & 0xFFFF) as u32;
                if first || value != previous + 1 {
                    container.num_runs += 1;
                }
                previous = value;
                first = false;
                index += 1;
            }
            container.end = index;

            let cardinality = (container.end - container.begin) as u32;
            let as_run = 2 + 4 * container.num_runs as u64;
            let as_array = 2 * cardinality as u64;
            if as_run < BITMAP_CONTAINER_SIZE_IN_MEMORY.min(as_array) {
                container.kind = ContainerKind::Run;
                container.body_size = as_run as u32;
            } else if cardinality <= MAX_ARRAY_CARDINALITY {
                container.kind = ContainerKind::Array;
                container.body_size = as_array as u32;
            } else {
                container.kind = ContainerKind::Bitmap;
                container.body_size = BITMAP_BODY_SIZE as u32;
            }

            group.has_run = group.has_run || container.kind == ContainerKind::Run;
            bodies += container.body_size as u64;
            layout.containers.push(container);
        }

        group.count = layout.containers.len() - group.first;
        group.cookie_size = if group.has_run { 4 } else { 8 };
        group.run_bitmap_size = if group.has_run {
            (group.count as u32 + 7) / 8
        } else {
            0
        };
        // A run-bearing blob with one, two or three containers omits the offset table; a blob with
        // no run container always writes it.
        group.has_offsets = !group.has_run || group.count >= 4;
        group.size = group.cookie_size as u64
            + group.run_bitmap_size as u64
            + 4 * group.count as u64
            + if group.has_offsets {
                4 * group.count as u64
            } else {
                0
            }
            + bodies;
        layout.body_length += 4 + group.size;
        layout.groups.push(group);
    }
    layout
}

fn stats_of(cardinality: u64, layout: &Layout) -> RoaringBitmapStats {
    let mut stats = RoaringBitmapStats {
        cardinality,
        high_container_count: layout.groups.len() as u64,
        low_container_count: layout.containers.len() as u64,
        array_containers: 0,
        bitmap_containers: 0,
        run_containers: 0,
        body_length: layout.body_length,
        estimated_decoded_size: 0,
    };
    for container in &layout.containers {
        match container.kind {
            ContainerKind::Array => stats.array_containers += 1,
            ContainerKind::Bitmap => stats.bitmap_containers += 1,
            ContainerKind::Run => stats.run_containers += 1,
        }
    }
    stats.estimated_decoded_size =
        layout.body_length + 128 * stats.high_container_count + 64 * stats.low_container_count;
    stats
}

/// Counts distinct high and low containers in one pass over the sorted keys.
fn count_buckets(keys: &[u64]) -> (u64, u64) {
    if keys.is_empty() {
        return (0, 0);
    }
    let mut high = 1u64;
    let mut low = 1u64;
    for pair in keys.windows(2) {
        if pair[1] >> 32 != pair[0] >> 32 {
            high += 1;
        }
        if pair[1] >> 16 != pair[0] >> 16 {
            low += 1;
        }
    }
    (high, low)
}

fn check_high_container_limit(count: u64) -> Result<()> {
    if count > MAX_HIGH_CONTAINER_COUNT {
        return Err(Error::validation(
            "members".into(),
            format!(
                "roaring bitmap high-container count {count} exceeds maximum {MAX_HIGH_CONTAINER_COUNT}"
            ),
        ));
    }
    Ok(())
}

/// Both limits decided from the counts alone, before `plan_layout` allocates per container.
fn check_bucket_limits(counts: (u64, u64)) -> Result<()> {
    check_high_container_limit(counts.0)?;
    let overhead = counts.0 * ESTIMATED_HIGH_CONTAINER_OVERHEAD_BYTES
        + counts.1 * ESTIMATED_LOW_CONTAINER_OVERHEAD_BYTES;
    if overhead > MAX_ESTIMATED_DECODED_BYTES {
        return Err(Error::validation(
            "members".into(),
            format!(
                "roaring bitmap estimated decoded size is at least {overhead}, exceeding maximum {MAX_ESTIMATED_DECODED_BYTES}"
            ),
        ));
    }
    Ok(())
}

fn check_limits(stats: &RoaringBitmapStats) -> Result<()> {
    check_high_container_limit(stats.high_container_count)?;
    if stats.estimated_decoded_size > MAX_ESTIMATED_DECODED_BYTES {
        return Err(Error::validation(
            "members".into(),
            format!(
                "roaring bitmap estimated decoded size {} exceeds maximum {MAX_ESTIMATED_DECODED_BYTES}",
                stats.estimated_decoded_size
            ),
        ));
    }
    if stats.body_length > MAX_BODY_BYTES {
        return Err(Error::validation(
            "members".into(),
            format!(
                "roaring bitmap body too large: body length {} exceeds maximum {MAX_BODY_BYTES}",
                stats.body_length
            ),
        ));
    }
    Ok(())
}

fn write_body(out: &mut [u8], keys: &[u64], layout: &Layout) {
    let mut pos = 0usize;
    write_u64_le(out, &mut pos, layout.groups.len() as u64);

    for group in &layout.groups {
        write_u32_le(out, &mut pos, group.key);

        // Everything from here to the end of this group is one portable Roaring32 blob, and the
        // offsets below are relative to this point.
        if group.has_run {
            write_u16_le(out, &mut pos, SERIAL_COOKIE);
            write_u16_le(out, &mut pos, (group.count - 1) as u16);
            for i in 0..group.count {
                if layout.containers[group.first + i].kind == ContainerKind::Run {
                    out[pos + i / 8] |= 1u8 << (i % 8);
                }
            }
            pos += group.run_bitmap_size as usize;
        } else {
            write_u32_le(out, &mut pos, SERIAL_COOKIE_NO_RUN_CONTAINER);
            write_u32_le(out, &mut pos, group.count as u32);
        }

        for i in 0..group.count {
            let container = &layout.containers[group.first + i];
            write_u16_le(out, &mut pos, container.key);
            // Minus one: a container holding all 65536 values still fits a uint16.
            write_u16_le(out, &mut pos, (container.end - container.begin - 1) as u16);
        }

        if group.has_offsets {
            let mut offset = group.cookie_size + group.run_bitmap_size + 8 * group.count as u32;
            for i in 0..group.count {
                write_u32_le(out, &mut pos, offset);
                offset += layout.containers[group.first + i].body_size;
            }
        }

        for i in 0..group.count {
            let container = &layout.containers[group.first + i];
            match container.kind {
                ContainerKind::Array => {
                    for key in &keys[container.begin..container.end] {
                        write_u16_le(out, &mut pos, *key as u16);
                    }
                }
                ContainerKind::Bitmap => {
                    // Word v >> 6, bit v & 63, stored little-endian — which is byte v >> 3,
                    // bit v & 7. The 8192 bytes start out zero.
                    for key in &keys[container.begin..container.end] {
                        let value = *key as u16;
                        out[pos + (value >> 3) as usize] |= 1u8 << (value & 7);
                    }
                    pos += BITMAP_BODY_SIZE;
                }
                ContainerKind::Run => {
                    write_u16_le(out, &mut pos, container.num_runs as u16);
                    let mut j = container.begin;
                    while j < container.end {
                        let start = (keys[j] & 0xFFFF) as u32;
                        let mut last = start;
                        j += 1;
                        while j < container.end && (keys[j] & 0xFFFF) as u32 == last + 1 {
                            last += 1;
                            j += 1;
                        }
                        write_u16_le(out, &mut pos, start as u16);
                        // Minus one, so a run covering the whole container still fits a uint16.
                        write_u16_le(out, &mut pos, (last - start) as u16);
                    }
                }
            }
        }
    }
}

fn write_u16_le(out: &mut [u8], pos: &mut usize, value: u16) {
    out[*pos..*pos + 2].copy_from_slice(&value.to_le_bytes());
    *pos += 2;
}

fn write_u32_le(out: &mut [u8], pos: &mut usize, value: u32) {
    out[*pos..*pos + 4].copy_from_slice(&value.to_le_bytes());
    *pos += 4;
}

fn write_u64_le(out: &mut [u8], pos: &mut usize, value: u64) {
    out[*pos..*pos + 8].copy_from_slice(&value.to_le_bytes());
    *pos += 8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn golden_cases() -> serde_json::Value {
        let json = include_str!("../../tests/v2/golden_vectors/roaring_golden_vectors.json");
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn blob_is_byte_identical_to_shared_golden_vectors() {
        let root = golden_cases();
        for case in root["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let mut builder = RoaringBitmapBuilder::new();
            for member in case["members"].as_array().unwrap() {
                // Members are `{start, count, step}` specs expanding to
                // start, start+step, ... repeated count times. `start` is a decimal string
                // (int64 does not survive a JSON number), `count`/`step` may be numbers.
                let value = |field: &str| -> serde_json::Value { member[field].clone() };
                let as_i64 = |field: &str| -> i64 {
                    let v = value(field);
                    v.as_str()
                        .and_then(|s| s.parse().ok())
                        .or_else(|| v.as_i64())
                        .unwrap()
                };
                let start: i64 = as_i64("start");
                let count: u64 = as_i64("count") as u64;
                let step: i64 = as_i64("step");
                for i in 0..count {
                    builder.add_int64(start + step * i as i64);
                }
            }
            let blob = builder.build().unwrap();
            let expected = base64::engine::general_purpose::STANDARD
                .decode(case["blob_base64"].as_str().unwrap())
                .unwrap();
            assert_eq!(blob, expected, "case {name} mismatch");

            let stats = builder.stats().unwrap();
            assert_eq!(
                stats.cardinality(),
                case["cardinality"].as_u64().unwrap(),
                "{name}"
            );
            assert_eq!(
                stats.body_length(),
                case["body_length"].as_u64().unwrap(),
                "{name}"
            );
            assert_eq!(
                stats.high_container_count(),
                case["high_container_count"].as_u64().unwrap(),
                "{name}"
            );
            assert_eq!(
                stats.low_container_count(),
                case["low_container_count"].as_u64().unwrap(),
                "{name}"
            );
        }
    }

    #[test]
    fn duplicate_members_are_deduplicated() {
        let mut builder = RoaringBitmapBuilder::new();
        for _ in 0..3 {
            builder.add_int64s(&[1, 2, 2, 3]);
        }
        assert_eq!(builder.cardinality(), 3);
        assert_eq!(builder.stats().unwrap().array_containers(), 1);
    }

    #[test]
    fn signed_values_land_in_the_top_key_space() {
        let mut builder = RoaringBitmapBuilder::new();
        builder.add_int64s(&[-1, 0, 5]);
        assert_eq!(builder.cardinality(), 3);
        // -1 (0xffffffffffffffff) is the largest key and belongs in the top high container.
        assert_eq!(builder.stats().unwrap().high_container_count(), 2);
    }

    #[test]
    fn build_rejects_excessive_high_container_count_before_allocating() {
        // MAX_HIGH_CONTAINER_COUNT + 1 members spread over distinct high groups (keys 2^32 apart)
        // must be rejected by check_bucket_limits before any per-container plan is allocated.
        let mut builder = RoaringBitmapBuilder::new();
        for i in 0..(MAX_HIGH_CONTAINER_COUNT + 1) {
            builder.add_int64((i << 32) as i64);
        }
        assert!(builder.validate().is_err());
        assert!(builder.build().is_err());
        assert!(builder.stats().is_err());
    }

    #[test]
    fn build_rejects_excessive_estimated_decoded_size_before_allocating() {
        // One member per 16-bit low container (all in the same high group): the per-container
        // overhead alone exceeds MAX_ESTIMATED_DECODED_BYTES, so the set is rejected up front.
        let low_containers_needed =
            MAX_ESTIMATED_DECODED_BYTES / ESTIMATED_LOW_CONTAINER_OVERHEAD_BYTES + 1;
        let mut builder = RoaringBitmapBuilder::new();
        for i in 0..low_containers_needed {
            builder.add_int64((i as i64) << 16);
        }
        assert!(builder.validate().is_err());
        assert!(builder.build().is_err());
        assert!(builder.stats().is_err());
    }
}
