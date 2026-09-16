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

//! Float16 and bfloat16 conversion utilities.
//!
//! Milvus represents both formats as their 16-bit bit patterns in the SDK API
//! and as little-endian bytes on the protobuf wire.

/// Converts an `f32` to an IEEE 754 binary16 bit pattern.
///
/// The mantissa is truncated and overflow becomes infinity. Representable
/// subnormals and signed zero are preserved, including on decode/encode round trips.
pub fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 31) as u16) << 15;
    let mut exponent = ((bits >> 23) & 0xff) as i32 - 127;
    let mantissa = bits & 0x7f_ffff;

    if value.is_nan() {
        return 0x7e00;
    }
    if value.is_infinite() {
        return sign | 0x7c00;
    }
    if value == 0.0 {
        return sign;
    }

    exponent += 15;
    if exponent <= 0 {
        // With unbiased f32 exponent e = exponent - 15, e <= -25 is below
        // the smallest binary16 subnormal (2^-24), so truncation yields zero.
        // The guard handles e < -25; e = -25 shifts the 24-bit significand
        // by 24 below, also yielding zero.
        if exponent < -10 {
            sign
        } else {
            // |v| = (2^23 + mantissa) * 2^(e - 23); in units of 2^-24,
            // the fraction is (2^23 + mantissa) * 2^(e + 1). Thus shift
            // right by -(e + 1) = 14 - exponent, truncating toward zero.
            sign | (((mantissa | 0x80_0000) >> (14 - exponent)) as u16)
        }
    } else if exponent >= 31 {
        sign | 0x7c00
    } else {
        sign | ((exponent as u16 & 0x1f) << 10) | (mantissa >> 13) as u16
    }
}

/// Converts an IEEE 754 binary16 bit pattern to `f32`.
pub fn f16_to_f32(value: u16) -> f32 {
    let sign = (value & 0x8000) != 0;
    let exponent = (value & 0x7c00) >> 10;
    let fraction = value & 0x03ff;

    let result = if exponent == 0x1f {
        if fraction == 0 {
            f32::INFINITY
        } else {
            f32::NAN
        }
    } else if exponent == 0 {
        if fraction == 0 {
            0.0
        } else {
            fraction as f32 / 1024.0 * 2.0_f32.powi(-14)
        }
    } else {
        (1.0 + fraction as f32 / 1024.0) * 2.0_f32.powi(exponent as i32 - 15)
    };

    if sign {
        -result
    } else {
        result
    }
}

/// Converts an `f32` to a bfloat16 bit pattern by truncating its low 16 bits.
pub fn f32_to_bf16(value: f32) -> u16 {
    (value.to_bits() >> 16) as u16
}

/// Converts a bfloat16 bit pattern to `f32`.
pub fn bf16_to_f32(value: u16) -> f32 {
    f32::from_bits((value as u32) << 16)
}

/// Performs the array f32 to f16 operation.
pub fn array_f32_to_f16(values: &[f32]) -> Vec<u16> {
    values.iter().copied().map(f32_to_f16).collect()
}

/// Performs the array f16 to f32 operation.
pub fn array_f16_to_f32(values: &[u16]) -> Vec<f32> {
    values.iter().copied().map(f16_to_f32).collect()
}

/// Performs the array f32 to bf16 operation.
pub fn array_f32_to_bf16(values: &[f32]) -> Vec<u16> {
    values.iter().copied().map(f32_to_bf16).collect()
}

/// Performs the array bf16 to f32 operation.
pub fn array_bf16_to_f32(values: &[u16]) -> Vec<f32> {
    values.iter().copied().map(bf16_to_f32).collect()
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_f16_preserving_subnormals_and_signed_zero() {
        assert_eq!(f32_to_f16(0.0), 0x0000);
        assert_eq!(f32_to_f16(-0.0), 0x8000);
        assert_eq!(f32_to_f16(1.0), 0x3c00);
        assert_eq!(f32_to_f16(-1.0), 0xbc00);
        assert_eq!(f32_to_f16(65504.0), 0x7bff);
        assert_eq!(f32_to_f16(f32::INFINITY), 0x7c00);
        assert_eq!(f32_to_f16(f32::NEG_INFINITY), 0xfc00);
        assert_eq!(f32_to_f16(f32::NAN), 0x7e00);
        assert_eq!(f32_to_f16(2.0_f32.powi(-15)), 0x0200);
        assert_eq!(f32_to_f16(65536.0), 0x7c00);

        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0xbc00), -1.0);
        assert_eq!(f16_to_f32(0x7c00), f32::INFINITY);
        assert!(f16_to_f32(0x7e00).is_nan());
    }

    #[test]
    fn every_finite_half_pattern_survives_a_round_trip() {
        for bits in 0..=u16::MAX {
            let value = f16_to_f32(bits);
            if value.is_finite() {
                assert_eq!(f32_to_f16(value), bits, "binary16 pattern {bits:#06x}");
            }
        }
    }

    #[test]
    fn converts_f16_subnormal_boundaries_by_truncating() {
        let unit = 2.0_f32.powi(-24);
        for (value, expected) in [
            (f32::from_bits(1), 0x0000),
            (unit / 2.0, 0x0000),
            (unit, 0x0001),
            (unit * 1.5, 0x0001),
            (unit * 1023.0, 0x03ff),
            (unit * 1023.5, 0x03ff),
            (unit * 1024.0, 0x0400),
        ] {
            assert_eq!(f32_to_f16(value), expected);
            assert_eq!(f32_to_f16(-value), expected | 0x8000);
        }
        let bits = [0x0000, 0x8000, 0x0001, 0x8001, 0x03ff, 0x83ff, 0x0400];
        assert_eq!(array_f32_to_f16(&array_f16_to_f32(&bits)), bits);
    }

    #[test]
    fn converts_bf16_by_truncating_low_bits() {
        let value = 1.234567_f32;
        assert_eq!(f32_to_bf16(value), (value.to_bits() >> 16) as u16);
        assert_eq!(
            bf16_to_f32(f32_to_bf16(value)).to_bits(),
            value.to_bits() & 0xffff_0000
        );
    }

    #[test]
    fn converts_arrays() {
        let values = [0.0, 1.0, -1.0];
        assert_eq!(array_f32_to_f16(&values), vec![0x0000, 0x3c00, 0xbc00]);
        assert_eq!(array_f16_to_f32(&[0x0000, 0x3c00, 0xbc00]), values);

        let bf16 = array_f32_to_bf16(&values);
        assert_eq!(array_bf16_to_f32(&bf16), values);
    }
}

/// Parses an optional extra-info string into `T`, falling back to `default`.
///
/// Shared by the response decoders that read server extra-info values such as
/// `report_value`, `scanned_remote_bytes`, `scanned_total_bytes`, and
/// `cache_hit_ratio`.
pub(crate) fn parse_extra<T>(
    values: &std::collections::HashMap<String, String>,
    key: &str,
    default: T,
) -> T
where
    T: std::str::FromStr,
{
    values
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
