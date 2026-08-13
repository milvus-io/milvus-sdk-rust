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
/// This intentionally matches the Milvus C++ SDK conversion: the mantissa is
/// truncated, values below the normal binary16 range become signed zero, and
/// overflow becomes infinity.
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
        return 0;
    }

    exponent += 15;
    if exponent <= 0 {
        sign
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
    fn converts_f16_with_cpp_sdk_semantics() {
        assert_eq!(f32_to_f16(0.0), 0x0000);
        assert_eq!(f32_to_f16(-0.0), 0x0000);
        assert_eq!(f32_to_f16(1.0), 0x3c00);
        assert_eq!(f32_to_f16(-1.0), 0xbc00);
        assert_eq!(f32_to_f16(65504.0), 0x7bff);
        assert_eq!(f32_to_f16(f32::INFINITY), 0x7c00);
        assert_eq!(f32_to_f16(f32::NEG_INFINITY), 0xfc00);
        assert_eq!(f32_to_f16(f32::NAN), 0x7e00);
        assert_eq!(f32_to_f16(2.0_f32.powi(-15)), 0x0000);
        assert_eq!(f32_to_f16(65536.0), 0x7c00);

        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0xbc00), -1.0);
        assert_eq!(f16_to_f32(0x7c00), f32::INFINITY);
        assert!(f16_to_f32(0x7e00).is_nan());
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
