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

//! Function-chain types for search-time reranking and post-processing.
//!
//! Function chains compose ordered [`FunctionChainOp`] operations (for example `map`, `sort`,
//! `limit`) into a [`FunctionChain`] executed by the server at a given [`FunctionChainStage`].
//! Expressions are built with [`FunctionChainExpr`] using [`col`] references and typed
//! [`FunctionParamValue`] parameters. The [`fn_`] module provides convenience constructors for
//! the common server-side functions, mirroring pymilvus's `fn` module.
//!
//! ```
//! use milvus::v2::prelude::*;
//!
//! let chain = FunctionChain::new()
//!     .stage(FunctionChainStage::L2Rerank)
//!     .name("fresh_popular_rerank")
//!     .map(
//!         "freshness",
//!         fn_::decay(
//!             col("published_at"),
//!             "exp",
//!             1700000000.0,
//!             86400.0,
//!             None,
//!             None,
//!         ),
//!     )
//!     .map(
//!         "$score",
//!         fn_::num_combine(
//!             vec![col("$score"), col("freshness"), col("popularity")],
//!             "weighted",
//!             Some(vec![0.7, 0.2, 0.1]),
//!         ),
//!     )
//!     .sort("$score", true, None)
//!     .limit(10, 0);
//! ```

use crate::proto::schema;
use crate::v2::error::{Error, Result};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// FunctionChainStage
///////////////////////////////////////////////////////////////////////////////
/// Execution stage where a function chain runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum FunctionChainStage {
    #[default]
    /// Represents the Unspecified case.
    Unspecified,
    /// Represents the Ingestion case.
    Ingestion,
    /// Represents the PreProcess case.
    PreProcess,
    /// Represents the L0Rerank case.
    L0Rerank,
    /// Represents the L1Rerank case.
    L1Rerank,
    /// Represents the L2Rerank case.
    L2Rerank,
    /// Represents the PostProcess case.
    PostProcess,
}

impl FunctionChainStage {
    pub(crate) fn into_proto(self) -> schema::FunctionChainStage {
        match self {
            Self::Unspecified => schema::FunctionChainStage::Unspecified,
            Self::Ingestion => schema::FunctionChainStage::Ingestion,
            Self::PreProcess => schema::FunctionChainStage::PreProcess,
            Self::L0Rerank => schema::FunctionChainStage::L0Rerank,
            Self::L1Rerank => schema::FunctionChainStage::L1Rerank,
            Self::L2Rerank => schema::FunctionChainStage::L2Rerank,
            Self::PostProcess => schema::FunctionChainStage::PostProcess,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionParamValue
///////////////////////////////////////////////////////////////////////////////
/// Typed parameter value accepted by a function chain expression or operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FunctionParamValue {
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer value.
    Int(i64),
    /// Represents a floating-point value.
    Double(f64),
    /// Represents a string value.
    String(String),
    /// Represents a bytes value.
    Bytes(Vec<u8>),
    /// Represents an array of values.
    Array(Vec<FunctionParamValue>),
    /// Represents a keyed object of values.
    Object(HashMap<String, FunctionParamValue>),
}

impl FunctionParamValue {
    /// Builds an array value from an iterator of values.
    pub fn array(values: impl IntoIterator<Item = impl Into<FunctionParamValue>>) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }

    /// Builds an object value from an iterator of key/value pairs.
    pub fn object(
        fields: impl IntoIterator<Item = (String, impl Into<FunctionParamValue>)>,
    ) -> Self {
        Self::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        )
    }

    pub(crate) fn into_proto(self) -> schema::FunctionParamValue {
        use schema::function_param_value::Value;
        schema::FunctionParamValue {
            value: Some(match self {
                Self::Bool(value) => Value::BoolValue(value),
                Self::Int(value) => Value::Int64Value(value),
                Self::Double(value) => Value::DoubleValue(value),
                Self::String(value) => Value::StringValue(value),
                Self::Bytes(value) => Value::BytesValue(value),
                Self::Array(values) => Value::ArrayValue(schema::FunctionParamArray {
                    values: values
                        .into_iter()
                        .map(FunctionParamValue::into_proto)
                        .collect(),
                }),
                Self::Object(fields) => Value::ObjectValue(schema::FunctionParamObject {
                    fields: fields
                        .into_iter()
                        .map(|(key, value)| (key, value.into_proto()))
                        .collect(),
                }),
            }),
        }
    }
}

impl From<bool> for FunctionParamValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for FunctionParamValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for FunctionParamValue {
    fn from(value: i32) -> Self {
        Self::Int(i64::from(value))
    }
}

impl From<f64> for FunctionParamValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<String> for FunctionParamValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for FunctionParamValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<u8>> for FunctionParamValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<Vec<FunctionParamValue>> for FunctionParamValue {
    fn from(value: Vec<FunctionParamValue>) -> Self {
        Self::Array(value)
    }
}

impl From<HashMap<String, FunctionParamValue>> for FunctionParamValue {
    fn from(value: HashMap<String, FunctionParamValue>) -> Self {
        Self::Object(value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionChainArg
///////////////////////////////////////////////////////////////////////////////
/// Expression argument: either a collection-field/temporary reference or a literal value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FunctionChainArg {
    /// A collection field, temporary variable, or system name (for example `"$score"`).
    Column(String),
    /// A literal argument value.
    Literal(FunctionParamValue),
}

/// Creates a column-reference argument for a function chain expression.
pub fn col(name: impl Into<String>) -> FunctionChainArg {
    FunctionChainArg::Column(name.into())
}

impl FunctionChainArg {
    pub(crate) fn into_proto(self) -> schema::FunctionChainExprArg {
        use schema::function_chain_expr_arg::Arg;
        schema::FunctionChainExprArg {
            arg: Some(match self {
                Self::Column(name) => Arg::Column(schema::FunctionChainColumnArg { name }),
                Self::Literal(value) => Arg::Literal(value.into_proto()),
            }),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionChainExpr
///////////////////////////////////////////////////////////////////////////////
/// A function invocation expression used by a function chain operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FunctionChainExpr {
    pub(crate) name: String,
    pub(crate) args: Vec<FunctionChainArg>,
    pub(crate) params: HashMap<String, FunctionParamValue>,
}

impl FunctionChainExpr {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            args: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Sets the expression name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the expression name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Sets the expression arguments and returns the updated value.
    pub fn args(mut self, values: impl IntoIterator<Item = FunctionChainArg>) -> Self {
        self.args = values.into_iter().collect();
        self
    }

    /// Sets the expression arguments and returns this value for further mutation.
    pub fn set_args(&mut self, values: impl IntoIterator<Item = FunctionChainArg>) -> &mut Self {
        self.args = values.into_iter().collect();
        self
    }

    /// Appends a positional argument and returns the updated value.
    pub fn add_arg(mut self, value: FunctionChainArg) -> Self {
        self.args.push(value);
        self
    }

    /// Sets the expression parameters and returns the updated value.
    pub fn params(
        mut self,
        values: impl IntoIterator<Item = (String, FunctionParamValue)>,
    ) -> Self {
        self.params = values.into_iter().collect();
        self
    }

    /// Sets the expression parameters and returns this value for further mutation.
    pub fn set_params(
        &mut self,
        values: impl IntoIterator<Item = (String, FunctionParamValue)>,
    ) -> &mut Self {
        self.params = values.into_iter().collect();
        self
    }

    /// Adds a keyword-style parameter and returns the updated value.
    pub fn add_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<FunctionParamValue>,
    ) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Returns the expression name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the expression arguments.
    pub fn get_args(&self) -> &[FunctionChainArg] {
        &self.args
    }

    /// Returns the expression parameters.
    pub fn get_params(&self) -> &HashMap<String, FunctionParamValue> {
        &self.params
    }

    pub(crate) fn into_proto(self) -> schema::FunctionChainExpr {
        schema::FunctionChainExpr {
            name: self.name,
            args: self
                .args
                .into_iter()
                .map(FunctionChainArg::into_proto)
                .collect(),
            params: self
                .params
                .into_iter()
                .map(|(key, value)| (key, value.into_proto()))
                .collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionChainOp
///////////////////////////////////////////////////////////////////////////////
/// A single operation in a function chain pipeline.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FunctionChainOp {
    pub(crate) op: String,
    pub(crate) expr: Option<FunctionChainExpr>,
    pub(crate) inputs: Vec<String>,
    pub(crate) outputs: Vec<String>,
    pub(crate) params: HashMap<String, FunctionParamValue>,
}

impl FunctionChainOp {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            op: String::new(),
            expr: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Sets the operation name and returns the updated value.
    pub fn op(mut self, value: impl Into<String>) -> Self {
        self.op = value.into();
        self
    }

    /// Sets the operation name and returns this value for further mutation.
    pub fn set_op(&mut self, value: impl Into<String>) -> &mut Self {
        self.op = value.into();
        self
    }

    /// Sets the attached expression and returns the updated value.
    pub fn expr(mut self, value: FunctionChainExpr) -> Self {
        self.expr = Some(value);
        self
    }

    /// Sets the attached expression and returns this value for further mutation.
    pub fn set_expr(&mut self, value: FunctionChainExpr) -> &mut Self {
        self.expr = Some(value);
        self
    }

    /// Sets the input names and returns the updated value.
    pub fn inputs(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.inputs = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the input names and returns this value for further mutation.
    pub fn set_inputs(&mut self, values: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.inputs = values.into_iter().map(Into::into).collect();
        self
    }

    /// Appends an input name and returns the updated value.
    pub fn add_input(mut self, value: impl Into<String>) -> Self {
        self.inputs.push(value.into());
        self
    }

    /// Sets the output names and returns the updated value.
    pub fn outputs(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.outputs = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the output names and returns this value for further mutation.
    pub fn set_outputs(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.outputs = values.into_iter().map(Into::into).collect();
        self
    }

    /// Appends an output name and returns the updated value.
    pub fn add_output(mut self, value: impl Into<String>) -> Self {
        self.outputs.push(value.into());
        self
    }

    /// Sets the operation parameters and returns the updated value.
    pub fn params(
        mut self,
        values: impl IntoIterator<Item = (String, FunctionParamValue)>,
    ) -> Self {
        self.params = values.into_iter().collect();
        self
    }

    /// Sets the operation parameters and returns this value for further mutation.
    pub fn set_params(
        &mut self,
        values: impl IntoIterator<Item = (String, FunctionParamValue)>,
    ) -> &mut Self {
        self.params = values.into_iter().collect();
        self
    }

    /// Adds a parameter and returns the updated value.
    pub fn add_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<FunctionParamValue>,
    ) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Returns the operation name.
    pub fn get_op(&self) -> &str {
        &self.op
    }

    /// Returns the attached expression.
    pub fn get_expr(&self) -> &Option<FunctionChainExpr> {
        &self.expr
    }

    /// Returns the input names.
    pub fn get_inputs(&self) -> &[String] {
        &self.inputs
    }

    /// Returns the output names.
    pub fn get_outputs(&self) -> &[String] {
        &self.outputs
    }

    /// Returns the operation parameters.
    pub fn get_params(&self) -> &HashMap<String, FunctionParamValue> {
        &self.params
    }

    pub(crate) fn into_proto(self) -> schema::FunctionChainOp {
        schema::FunctionChainOp {
            op: self.op,
            expr: self.expr.map(FunctionChainExpr::into_proto),
            inputs: self.inputs,
            outputs: self.outputs,
            params: self
                .params
                .into_iter()
                .map(|(key, value)| (key, value.into_proto()))
                .collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionChain
///////////////////////////////////////////////////////////////////////////////
/// An ordered rerank/refine plan executed by the server at a function chain stage.
///
/// Compose operations fluently with [`Self::map`], [`Self::sort`], [`Self::limit`], or attach any
/// raw [`FunctionChainOp`] with [`Self::add_op`], then attach the chain to a search request. The
/// stage must not remain [`FunctionChainStage::Unspecified`] and the chain cannot be combined with
/// a search `ranker`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FunctionChain {
    pub(crate) name: String,
    pub(crate) stage: FunctionChainStage,
    pub(crate) ops: Vec<FunctionChainOp>,
}

impl FunctionChain {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            stage: FunctionChainStage::Unspecified,
            ops: Vec::new(),
        }
    }

    /// Sets the chain name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the chain name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Sets the execution stage and returns the updated value.
    pub fn stage(mut self, value: FunctionChainStage) -> Self {
        self.stage = value;
        self
    }

    /// Sets the execution stage and returns this value for further mutation.
    pub fn set_stage(&mut self, value: FunctionChainStage) -> &mut Self {
        self.stage = value;
        self
    }

    /// Sets the ordered operations and returns the updated value.
    pub fn ops(mut self, values: impl IntoIterator<Item = FunctionChainOp>) -> Self {
        self.ops = values.into_iter().collect();
        self
    }

    /// Sets the ordered operations and returns this value for further mutation.
    pub fn set_ops(&mut self, values: impl IntoIterator<Item = FunctionChainOp>) -> &mut Self {
        self.ops = values.into_iter().collect();
        self
    }

    /// Appends a raw [`FunctionChainOp`] and returns the updated value.
    ///
    /// Use this to attach operations that the fluent [`Self::map`], [`Self::sort`], and
    /// [`Self::limit`] conveniences do not expose, such as `filter`, `merge`, or a
    /// hand-built future operation. The operation is validated by [`Self::validate`]
    /// together with every other operation in the chain.
    pub fn add_op(mut self, op: FunctionChainOp) -> Self {
        self.ops.push(op);
        self
    }

    /// Appends a `map` operation that writes an expression result to an output column.
    pub fn map(mut self, output: impl Into<String>, expr: FunctionChainExpr) -> Self {
        self.ops.push(
            FunctionChainOp::new()
                .op("map")
                .expr(expr)
                .outputs([output.into()]),
        );
        self
    }

    /// Appends a `sort` operation by a column, optionally with a tie-break column.
    pub fn sort(
        mut self,
        by: impl Into<String>,
        desc: bool,
        tie_break_col: Option<String>,
    ) -> Self {
        let column = by.into();
        let mut inputs = vec![column.clone()];
        let mut op = FunctionChainOp::new()
            .op("sort")
            .add_param("column", column)
            .add_param("desc", desc);
        if let Some(tie_break_col) = tie_break_col {
            inputs.push(tie_break_col.clone());
            op = op.add_param("tie_break_col", tie_break_col);
        }
        self.ops.push(op.inputs(inputs));
        self
    }

    /// Appends a `limit` operation with an optional offset.
    pub fn limit(mut self, limit: i64, offset: i64) -> Self {
        self.ops.push(
            FunctionChainOp::new()
                .op("limit")
                .add_param("limit", limit)
                .add_param("offset", offset),
        );
        self
    }

    /// Returns the chain name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the execution stage.
    pub fn get_stage(&self) -> FunctionChainStage {
        self.stage
    }

    /// Returns the ordered operations.
    pub fn get_ops(&self) -> &[FunctionChainOp] {
        &self.ops
    }

    /// Validates that the chain can be attached to a search request.
    pub fn validate(&self) -> Result<()> {
        if self.stage == FunctionChainStage::Unspecified {
            return Err(Error::validation(
                "function_chains".into(),
                "UNSPECIFIED function chain stage is not supported for search".into(),
            ));
        }
        if self.ops.is_empty() {
            return Err(Error::validation(
                "function_chains".into(),
                "function chain must contain at least one operation".into(),
            ));
        }
        for op in &self.ops {
            let op_name = op.get_op();
            validate_param_keys(&op.params, "function chain operation")?;
            if op_name.is_empty() {
                return Err(Error::validation(
                    "function_chains".into(),
                    "operation name must not be empty".into(),
                ));
            }
            if let Some(expr) = &op.expr {
                Self::validate_expr(expr)?;
            }
            match op_name {
                "map" => {
                    if op.expr.is_none() {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "map operation must attach an expression".into(),
                        ));
                    }
                    if op.outputs.is_empty() || op.outputs.iter().any(String::is_empty) {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "map operation must write to at least one non-empty output name".into(),
                        ));
                    }
                }
                "sort" => {
                    if !op
                        .inputs
                        .first()
                        .map(|column| !column.is_empty())
                        .unwrap_or(false)
                    {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "sort operation must specify a non-empty sort column".into(),
                        ));
                    }
                }
                "limit" => {
                    match op.params.get("limit") {
                        Some(FunctionParamValue::Int(limit)) if *limit > 0 => {}
                        _ => {
                            return Err(Error::validation(
                                "function_chains".into(),
                                "limit operation must specify a positive limit".into(),
                            ));
                        }
                    }
                    if let Some(FunctionParamValue::Int(offset)) = op.params.get("offset") {
                        if *offset < 0 {
                            return Err(Error::validation(
                                "function_chains".into(),
                                "limit operation must not use a negative offset".into(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_expr(expr: &FunctionChainExpr) -> Result<()> {
        if expr.get_name().is_empty() {
            return Err(Error::validation(
                "function_chains".into(),
                "expression name must not be empty".into(),
            ));
        }
        validate_param_keys(expr.get_params(), "function chain expression")?;
        for arg in expr.get_args() {
            if let FunctionChainArg::Column(name) = arg {
                if name.is_empty() {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "column reference must not be empty".into(),
                    ));
                }
            }
        }
        match expr.get_name() {
            "num_combine" => {
                let columns = expr.get_args();
                if columns.len() < 2 {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "num_combine requires at least two columns".into(),
                    ));
                }
                if columns
                    .iter()
                    .any(|arg| !matches!(arg, FunctionChainArg::Column(_)))
                {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "num_combine arguments must be column references".into(),
                    ));
                }
                let mode = match expr.get_params().get("mode") {
                    Some(FunctionParamValue::String(mode)) => mode.as_str(),
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "num_combine must specify a mode".into(),
                        ));
                    }
                };
                if !matches!(
                    mode,
                    "multiply" | "sum" | "max" | "min" | "avg" | "weighted"
                ) {
                    return Err(Error::validation(
                        "function_chains".into(),
                        format!("unsupported num_combine mode {mode}"),
                    ));
                }
                match expr.get_params().get("weights") {
                    Some(FunctionParamValue::Array(weights))
                        if mode == "weighted"
                            && weights.len() == columns.len()
                            && weights.iter().all(|weight| {
                                matches!(weight, FunctionParamValue::Double(value) if value.is_finite())
                            }) => {}
                    None if mode != "weighted" => {}
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "num_combine weighted mode requires finite numeric weights matching the column count"
                                .into(),
                        ));
                    }
                }
            }
            "decay" => {
                let args = expr.get_args();
                if args.len() != 1 || !matches!(args[0], FunctionChainArg::Column(_)) {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "decay requires a single column value".into(),
                    ));
                }
                let function = match expr.get_params().get("function") {
                    Some(FunctionParamValue::String(function)) => function.as_str(),
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "decay must specify a function".into(),
                        ));
                    }
                };
                if !matches!(function, "gauss" | "exp" | "linear") {
                    return Err(Error::validation(
                        "function_chains".into(),
                        format!("unsupported decay function {function}"),
                    ));
                }
                for name in ["origin", "scale", "offset", "decay"] {
                    match expr.get_params().get(name) {
                        Some(FunctionParamValue::Double(value)) if value.is_finite() => {}
                        Some(FunctionParamValue::Double(_)) => {
                            return Err(Error::validation(
                                "function_chains".into(),
                                format!("decay {name} must be a finite number"),
                            ));
                        }
                        Some(_) => {
                            return Err(Error::validation(
                                "function_chains".into(),
                                format!("decay {name} must be numeric"),
                            ));
                        }
                        None if name == "origin" || name == "scale" => {
                            return Err(Error::validation(
                                "function_chains".into(),
                                format!("decay must specify {name}"),
                            ));
                        }
                        None => {}
                    }
                }
            }
            "round_decimal" => {
                let args = expr.get_args();
                if args.len() != 1 || !matches!(args[0], FunctionChainArg::Column(_)) {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "round_decimal requires a single column value".into(),
                    ));
                }
                match expr.get_params().get("decimal") {
                    Some(FunctionParamValue::Int(decimal)) if (0..=6).contains(decimal) => {}
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "round_decimal decimal must be an integer in [0, 6]".into(),
                        ));
                    }
                }
            }
            "xgboost" => {
                let features = expr.get_args();
                if features.is_empty()
                    || features
                        .iter()
                        .any(|arg| !matches!(arg, FunctionChainArg::Column(_)))
                {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "xgboost requires at least one feature column".into(),
                    ));
                }
                match expr.get_params().get("model_resource") {
                    Some(FunctionParamValue::String(resource)) if !resource.is_empty() => {}
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "xgboost model_resource must be a non-empty string".into(),
                        ));
                    }
                }
                match expr.get_params().get("output") {
                    Some(FunctionParamValue::String(output))
                        if matches!(output.as_str(), "default" | "raw") => {}
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "xgboost output must be either \"default\" or \"raw\"".into(),
                        ));
                    }
                }
            }
            "rerank_model" => {
                let args = expr.get_args();
                if args.len() != 1 || !matches!(args[0], FunctionChainArg::Column(_)) {
                    return Err(Error::validation(
                        "function_chains".into(),
                        "rerank_model requires a single column value".into(),
                    ));
                }
                match expr.get_params().get("queries") {
                    Some(FunctionParamValue::Array(queries))
                        if !queries.is_empty()
                            && queries.iter().all(|query| {
                                matches!(
                                    query,
                                    FunctionParamValue::String(value) if !value.is_empty()
                                )
                            }) => {}
                    _ => {
                        return Err(Error::validation(
                            "function_chains".into(),
                            "rerank_model queries must be a non-empty list of non-empty strings"
                                .into(),
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn into_proto(self) -> schema::FunctionChain {
        schema::FunctionChain {
            name: self.name,
            stage: self.stage.into_proto() as i32,
            ops: self
                .ops
                .into_iter()
                .map(FunctionChainOp::into_proto)
                .collect(),
            ..Default::default()
        }
    }
}

/// Rejects empty keys in a parameter map, recursing into array and object values.
///
/// Empty parameter names would serialize into the proto parameter maps and are rejected by
/// pymilvus (`_copy_param_map`) and the Milvus server alike.
fn validate_param_keys(params: &HashMap<String, FunctionParamValue>, context: &str) -> Result<()> {
    for (key, value) in params {
        if key.is_empty() {
            return Err(Error::validation(
                "function_chains".into(),
                format!("{context} parameter names must not be empty"),
            ));
        }
        validate_param_value_keys(value, context)?;
    }
    Ok(())
}

fn validate_param_value_keys(value: &FunctionParamValue, context: &str) -> Result<()> {
    match value {
        FunctionParamValue::Array(values) => {
            for value in values {
                validate_param_value_keys(value, context)?;
            }
        }
        FunctionParamValue::Object(fields) => validate_param_keys(fields, context)?,
        _ => {}
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
// fn_
///////////////////////////////////////////////////////////////////////////////
/// Convenience constructors for common server-side function chain expressions.
pub mod fn_ {
    use super::{FunctionChainArg, FunctionChainExpr};
    use std::collections::HashMap;

    /// Builds a numeric combination expression over two or more columns.
    pub fn num_combine(
        cols: Vec<FunctionChainArg>,
        mode: &str,
        weights: Option<Vec<f64>>,
    ) -> FunctionChainExpr {
        let mut expr = FunctionChainExpr::new().name("num_combine");
        for column in cols {
            expr = expr.add_arg(column);
        }
        expr = expr.add_param("mode", mode);
        if let Some(weights) = weights {
            expr = expr.add_param("weights", super::FunctionParamValue::array(weights));
        }
        expr
    }

    /// Builds a decay scoring expression for a numeric column.
    pub fn decay(
        value: FunctionChainArg,
        function: &str,
        origin: f64,
        scale: f64,
        offset: Option<f64>,
        decay: Option<f64>,
    ) -> FunctionChainExpr {
        let mut expr = FunctionChainExpr::new()
            .name("decay")
            .add_arg(value)
            .add_param("function", function)
            .add_param("origin", origin)
            .add_param("scale", scale);
        if let Some(offset) = offset {
            expr = expr.add_param("offset", offset);
        }
        if let Some(decay) = decay {
            expr = expr.add_param("decay", decay);
        }
        expr
    }

    /// Builds an expression that rounds a numeric column to a fixed number of decimals.
    pub fn round_decimal(value: FunctionChainArg, decimal: i64) -> FunctionChainExpr {
        FunctionChainExpr::new()
            .name("round_decimal")
            .add_arg(value)
            .add_param("decimal", decimal)
    }

    /// Builds an expression that scores feature columns with a server-side XGBoost model.
    ///
    /// `output` selects the prediction mode: `"default"` applies the model objective transform
    /// when supported, and `"raw"` returns the raw margin. When `output` is `None`, `"default"`
    /// is used, mirroring pymilvus.
    pub fn xgboost(
        features: Vec<FunctionChainArg>,
        model_resource: &str,
        output: Option<&str>,
    ) -> FunctionChainExpr {
        let mut expr = FunctionChainExpr::new()
            .name("xgboost")
            .add_param("model_resource", model_resource)
            .add_param("output", output.unwrap_or("default"));
        for feature in features {
            expr = expr.add_arg(feature);
        }
        expr
    }

    /// Builds an expression that reranks a column with an external rerank model provider.
    pub fn rerank_model(
        value: FunctionChainArg,
        queries: Vec<String>,
        provider_params: HashMap<String, super::FunctionParamValue>,
    ) -> FunctionChainExpr {
        let mut expr = FunctionChainExpr::new()
            .name("rerank_model")
            .add_arg(value)
            .add_param("queries", super::FunctionParamValue::array(queries));
        for (key, value) in provider_params {
            expr = expr.add_param(key, value);
        }
        expr
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{col, fn_, FunctionChain, FunctionChainStage};
    use crate::proto::schema;

    #[test]
    fn function_chain_encodes_stage_and_ops() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .name("fresh_popular_rerank")
            .map(
                "freshness",
                fn_::decay(
                    col("published_at"),
                    "exp",
                    1700000000.0,
                    86400.0,
                    None,
                    None,
                ),
            )
            .map(
                "$score",
                fn_::num_combine(
                    vec![col("$score"), col("freshness"), col("popularity")],
                    "weighted",
                    Some(vec![0.7, 0.2, 0.1]),
                ),
            )
            .sort("$score", true, None)
            .limit(10, 0);
        chain.validate().expect("valid chain");

        let proto = chain.into_proto();
        assert_eq!(proto.name, "fresh_popular_rerank");
        assert_eq!(proto.stage, schema::FunctionChainStage::L2Rerank as i32);
        assert_eq!(proto.ops.len(), 4);

        assert_eq!(proto.ops[0].op, "map");
        assert_eq!(proto.ops[0].outputs, ["freshness"]);
        let decay = proto.ops[0].expr.as_ref().expect("map expr");
        assert_eq!(decay.name, "decay");
        assert_eq!(decay.args.len(), 1);
        assert!(matches!(
            decay.args[0].arg,
            Some(schema::function_chain_expr_arg::Arg::Column(_))
        ));
        assert_eq!(
            decay.params.get("function").and_then(|v| v.value.as_ref()),
            Some(&schema::function_param_value::Value::StringValue(
                "exp".to_owned()
            ))
        );

        assert_eq!(proto.ops[1].op, "map");
        let combine = proto.ops[1].expr.as_ref().expect("combine expr");
        assert_eq!(combine.name, "num_combine");
        assert_eq!(combine.args.len(), 3);

        assert_eq!(proto.ops[2].op, "sort");
        assert_eq!(proto.ops[2].inputs, ["$score"]);
        assert_eq!(
            proto.ops[2]
                .params
                .get("desc")
                .and_then(|v| v.value.as_ref()),
            Some(&schema::function_param_value::Value::BoolValue(true))
        );

        assert_eq!(proto.ops[3].op, "limit");
        assert_eq!(
            proto.ops[3]
                .params
                .get("limit")
                .and_then(|v| v.value.as_ref()),
            Some(&schema::function_param_value::Value::Int64Value(10))
        );
    }

    #[test]
    fn function_chain_rejects_unspecified_stage() {
        let chain = FunctionChain::new().map("out", fn_::round_decimal(col("$score"), 3));
        let error = chain
            .validate()
            .expect_err("unspecified stage must be rejected");
        assert!(error.to_string().contains("UNSPECIFIED"));
    }

    #[test]
    fn function_chain_rejects_empty_ops() {
        let chain = FunctionChain::new().stage(FunctionChainStage::L2Rerank);
        let error = chain
            .validate()
            .expect_err("chain without operations must be rejected");
        assert!(error.to_string().contains("at least one operation"));
    }

    #[test]
    fn function_chain_rejects_non_positive_limit() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .limit(0, 0);
        let error = chain
            .validate()
            .expect_err("non-positive limit must be rejected");
        assert!(error.to_string().contains("positive limit"));

        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .limit(-1, 0);
        assert!(chain.validate().is_err());
    }

    #[test]
    fn function_chain_rejects_negative_offset() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .limit(10, -1);
        let error = chain
            .validate()
            .expect_err("negative offset must be rejected");
        assert!(error.to_string().contains("negative offset"));
    }

    #[test]
    fn function_chain_rejects_empty_sort_column() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .sort("", true, None);
        let error = chain
            .validate()
            .expect_err("empty sort column must be rejected");
        assert!(error.to_string().contains("sort"));
    }

    #[test]
    fn function_chain_accepts_raw_sort_op_with_inputs_only() {
        let op = super::FunctionChainOp::new()
            .op("sort")
            .inputs(["$score"])
            .add_param("desc", true);
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .add_op(op);
        chain
            .validate()
            .expect("sort column from inputs[0] is valid");
    }

    #[test]
    fn function_chain_rejects_raw_sort_op_without_inputs() {
        let op = super::FunctionChainOp::new()
            .op("sort")
            .add_param("desc", true);
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .add_op(op);
        let error = chain
            .validate()
            .expect_err("sort without a column input must be rejected");
        assert!(error.to_string().contains("sort"));
    }

    #[test]
    fn function_chain_rejects_empty_map_output() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map("", fn_::round_decimal(col("$score"), 3));
        let error = chain
            .validate()
            .expect_err("empty map output must be rejected");
        assert!(error.to_string().contains("map"));
    }

    #[test]
    fn function_chain_rejects_empty_operation_name() {
        let mut chain = FunctionChain::new().stage(FunctionChainStage::L2Rerank);
        chain.ops.push(super::FunctionChainOp::new().op(""));
        let error = chain
            .validate()
            .expect_err("empty operation name must be rejected");
        assert!(error.to_string().contains("operation name"));
    }

    #[test]
    fn function_chain_rejects_empty_expression_name() {
        let op = super::FunctionChainOp::new()
            .op("map")
            .expr(super::FunctionChainExpr::new().name(""))
            .outputs(["out"]);
        let mut chain = FunctionChain::new().stage(FunctionChainStage::L2Rerank);
        chain.ops.push(op);
        let error = chain
            .validate()
            .expect_err("empty expression name must be rejected");
        assert!(error.to_string().contains("expression name"));
    }

    #[test]
    fn function_chain_rejects_map_without_expression() {
        let op = super::FunctionChainOp::new().op("map").outputs(["out"]);
        let mut chain = FunctionChain::new().stage(FunctionChainStage::L2Rerank);
        chain.ops.push(op);
        let error = chain
            .validate()
            .expect_err("map without expression must be rejected");
        assert!(error.to_string().contains("expression"));
    }

    #[test]
    fn function_chain_rejects_empty_column_reference() {
        let op = super::FunctionChainOp::new()
            .op("map")
            .expr(
                super::FunctionChainExpr::new()
                    .name("round_decimal")
                    .add_arg(col("")),
            )
            .outputs(["out"]);
        let mut chain = FunctionChain::new().stage(FunctionChainStage::L2Rerank);
        chain.ops.push(op);
        let error = chain
            .validate()
            .expect_err("empty column reference must be rejected");
        assert!(error.to_string().contains("column reference"));
    }

    #[test]
    fn function_chain_adds_raw_operation() {
        let op = super::FunctionChainOp::new()
            .op("map")
            .expr(fn_::num_combine(
                vec![col("$score"), col("freshness")],
                "sum",
                None,
            ))
            .outputs(["out"]);
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .name("manual")
            .add_op(op);
        chain.validate().expect("valid chain with a raw operation");

        let proto = chain.into_proto();
        assert_eq!(proto.ops.len(), 1);
        assert_eq!(proto.ops[0].op, "map");
        assert_eq!(proto.ops[0].outputs, ["out"]);
    }

    #[test]
    fn function_chain_add_op_validates_expression() {
        let op = super::FunctionChainOp::new()
            .op("map")
            .expr(fn_::num_combine(vec![col("$score")], "sum", None))
            .outputs(["out"]);
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .add_op(op);
        let error = chain
            .validate()
            .expect_err("single-column num_combine must be rejected");
        assert!(error.to_string().contains("at least two columns"));
    }

    #[test]
    fn function_chain_rejects_num_combine_with_one_column() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map("out", fn_::num_combine(vec![col("$score")], "sum", None));
        let error = chain
            .validate()
            .expect_err("single-column num_combine must be rejected");
        assert!(error.to_string().contains("at least two columns"));
    }

    #[test]
    fn function_chain_rejects_num_combine_literal_arg() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::num_combine(
                    vec![
                        col("$score"),
                        super::FunctionChainArg::Literal(super::FunctionParamValue::Double(1.0)),
                    ],
                    "sum",
                    None,
                ),
            );
        let error = chain
            .validate()
            .expect_err("literal num_combine argument must be rejected");
        assert!(error.to_string().contains("column references"));
    }

    #[test]
    fn function_chain_rejects_weighted_num_combine_without_weights() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::num_combine(vec![col("$score"), col("freshness")], "weighted", None),
            );
        let error = chain
            .validate()
            .expect_err("weighted num_combine without weights must be rejected");
        assert!(error.to_string().contains("weights"));
    }

    #[test]
    fn function_chain_rejects_mismatched_num_combine_weights() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::num_combine(
                    vec![col("$score"), col("freshness")],
                    "weighted",
                    Some(vec![0.5]),
                ),
            );
        let error = chain
            .validate()
            .expect_err("mismatched num_combine weights must be rejected");
        assert!(error.to_string().contains("weights"));
    }

    #[test]
    fn function_chain_rejects_non_finite_num_combine_weights() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .add_op(
                super::FunctionChainOp::new()
                    .op("map")
                    .expr(
                        super::FunctionChainExpr::new()
                            .name("num_combine")
                            .add_arg(col("$score"))
                            .add_arg(col("popularity"))
                            .add_param("mode", "weighted")
                            .add_param(
                                "weights",
                                super::FunctionParamValue::array([f64::NAN, f64::INFINITY]),
                            ),
                    )
                    .outputs(["out"]),
            );
        let error = chain
            .validate()
            .expect_err("non-finite num_combine weights must be rejected");
        assert!(error.to_string().contains("weights"));
    }

    #[test]
    fn function_chain_rejects_non_numeric_num_combine_weights() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .add_op(
                super::FunctionChainOp::new()
                    .op("map")
                    .expr(
                        super::FunctionChainExpr::new()
                            .name("num_combine")
                            .add_arg(col("$score"))
                            .add_arg(col("popularity"))
                            .add_param("mode", "weighted")
                            .add_param("weights", super::FunctionParamValue::array(["0.5", "0.5"])),
                    )
                    .outputs(["out"]),
            );
        let error = chain
            .validate()
            .expect_err("non-numeric num_combine weights must be rejected");
        assert!(error.to_string().contains("weights"));
    }

    #[test]
    fn function_chain_rejects_empty_param_key() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                super::FunctionChainExpr::new()
                    .name("round_decimal")
                    .add_arg(col("$score"))
                    .add_param("", 3),
            );
        let error = chain
            .validate()
            .expect_err("empty parameter keys must be rejected");
        assert!(error.to_string().contains("parameter names"));
    }

    #[test]
    fn function_chain_rejects_empty_nested_object_key() {
        let mut nested = std::collections::HashMap::new();
        nested.insert(String::new(), super::FunctionParamValue::Double(1.0));
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                super::FunctionChainExpr::new()
                    .name("round_decimal")
                    .add_arg(col("$score"))
                    .add_param("provider", super::FunctionParamValue::Object(nested)),
            );
        let error = chain
            .validate()
            .expect_err("empty nested object keys must be rejected");
        assert!(error.to_string().contains("parameter names"));
    }

    #[test]
    fn function_chain_rejects_unsupported_num_combine_mode() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::num_combine(vec![col("$score"), col("freshness")], "product", None),
            );
        let error = chain
            .validate()
            .expect_err("unsupported num_combine mode must be rejected");
        assert!(error.to_string().contains("mode"));
    }

    #[test]
    fn function_chain_rejects_unsupported_decay_function() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::decay(col("published_at"), "foo", 1.0, 1.0, None, None),
            );
        let error = chain
            .validate()
            .expect_err("unsupported decay function must be rejected");
        assert!(error.to_string().contains("decay function"));
    }

    #[test]
    fn function_chain_rejects_round_decimal_out_of_range() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map("out", fn_::round_decimal(col("$score"), 7));
        let error = chain
            .validate()
            .expect_err("out-of-range round_decimal must be rejected");
        assert!(error.to_string().contains("decimal"));
    }

    #[test]
    fn function_chain_rejects_xgboost_without_features() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map("out", fn_::xgboost(vec![], "xgb_model", None));
        let error = chain
            .validate()
            .expect_err("xgboost without features must be rejected");
        assert!(error.to_string().contains("feature column"));
    }

    #[test]
    fn function_chain_xgboost_defaults_output_to_default() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::xgboost(vec![col("price"), col("popularity")], "xgb_model", None),
            );
        chain.validate().expect("valid xgboost chain");

        let proto = chain.into_proto();
        let expr = proto.ops[0].expr.as_ref().expect("xgboost expr");
        assert_eq!(
            expr.params.get("output").and_then(|v| v.value.as_ref()),
            Some(&schema::function_param_value::Value::StringValue(
                "default".to_owned()
            ))
        );
    }

    #[test]
    fn function_chain_rejects_rerank_model_without_queries() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "out",
                fn_::rerank_model(col("text"), vec![], std::collections::HashMap::new()),
            );
        let error = chain
            .validate()
            .expect_err("rerank_model without queries must be rejected");
        assert!(error.to_string().contains("queries"));
    }

    #[test]
    fn function_param_value_encodes_nested_arrays_and_objects() {
        let value = super::FunctionParamValue::object([
            (
                "mode".to_owned(),
                super::FunctionParamValue::String("weighted".to_owned()),
            ),
            (
                "weights".to_owned(),
                super::FunctionParamValue::array([0.7_f64, 0.3_f64]),
            ),
        ]);
        let proto = value.into_proto();
        match proto.value {
            Some(schema::function_param_value::Value::ObjectValue(object)) => {
                assert_eq!(object.fields.len(), 2);
                assert!(object.fields.contains_key("mode"));
                assert!(object.fields.contains_key("weights"));
            }
            other => panic!("unexpected param value encoding: {other:?}"),
        }
    }
}
