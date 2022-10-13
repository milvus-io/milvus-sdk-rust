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

use milvus::value::Value;

#[test]
fn test_try_from_for_value_column(){
    // bool
    let b = Value::Bool(false);
    let b:Result<bool,milvus::error::Error > = b.try_into();
    assert!(b.is_ok());
    assert_eq!(false,b.unwrap());
    //i8
    let int8 = Value::Int8(12);
    let r:Result<i8,milvus::error::Error > = int8.try_into();
    assert!(r.is_ok());
    assert_eq!(12,r.unwrap());
    //i16
    let int16 = Value::Int16(1225);
    let r:Result<i16,milvus::error::Error > = int16.try_into();
    assert!(r.is_ok());
    assert_eq!(1225,r.unwrap());
    //i32
    let int32 = Value::Int32(37360798);
    let r:Result<i32,milvus::error::Error > = int32.try_into();
    assert!(r.is_ok());
    assert_eq!(37360798,r.unwrap());
    //i64
    let long = Value::Long(123 as i64);
    let r:Result<i64,milvus::error::Error > = long.try_into();
    assert!(r.is_ok());
    assert_eq!(123,r.unwrap());

    let float = Value::Float(22104f32);
    let r: Result<f32,milvus::error::Error> = float.try_into();
    assert!(r.is_ok());
    assert_eq!(22104f32,r.unwrap());

    let double = Value::Double(22104f64);
    let r: Result<f64,milvus::error::Error> = double.try_into();
    assert!(r.is_ok());
    assert_eq!(22104f64,r.unwrap());

}