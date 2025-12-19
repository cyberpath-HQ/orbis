//! Database access for plugins.
//!
//! Provides type-safe database querying and mutation.
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_plugin_api::sdk::db;
//!
//! // Query with parameters
//! let users = db::query::<User>(
//!     "SELECT * FROM users WHERE active = ? LIMIT ?",
//!     &[&true, &10]
//! )?;
//!
//! // Execute a mutation
//! let rows_affected = db::execute(
//!     "UPDATE users SET last_login = ? WHERE id = ?",
//!     &[&now, &user_id]
//! )?;
//! ```

use super::error::{Error, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A value that can be used as a database parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DbValue {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// JSON value
    Json(serde_json::Value),
}

impl From<()> for DbValue {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl From<bool> for DbValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i32> for DbValue {
    fn from(v: i32) -> Self {
        Self::Int(i64::from(v))
    }
}

impl From<i64> for DbValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for DbValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<&str> for DbValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for DbValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<Vec<u8>> for DbValue {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}

impl From<serde_json::Value> for DbValue {
    fn from(v: serde_json::Value) -> Self {
        Self::Json(v)
    }
}

/// Trait for types that can be converted to database parameters
pub trait ToDbParams {
    fn to_db_params(&self) -> Vec<DbValue>;
}

impl ToDbParams for () {
    fn to_db_params(&self) -> Vec<DbValue> {
        vec![]
    }
}

impl<T: Into<DbValue> + Clone> ToDbParams for &[T] {
    fn to_db_params(&self) -> Vec<DbValue> {
        self.iter().map(|v| v.clone().into()).collect()
    }
}

impl<T: Into<DbValue> + Clone, const N: usize> ToDbParams for [T; N] {
    fn to_db_params(&self) -> Vec<DbValue> {
        self.iter().map(|v| v.clone().into()).collect()
    }
}

impl ToDbParams for Vec<DbValue> {
    fn to_db_params(&self) -> Vec<DbValue> {
        self.clone()
    }
}

/// A row from a database query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRow {
    /// Column values by name
    #[serde(flatten)]
    pub columns: std::collections::HashMap<String, serde_json::Value>,
}

impl DbRow {
    /// Get a column value by name
    #[inline]
    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.columns.get(name)
    }

    /// Get a column value as a specific type
    pub fn get_as<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        match self.columns.get(name) {
            Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(Error::from),
            None => Ok(None),
        }
    }

    /// Get a required column value
    pub fn get_required<T: DeserializeOwned>(&self, name: &str) -> Result<T> {
        self.get_as(name)?
            .ok_or_else(|| Error::database(format!("Missing column: {}", name)))
    }

    /// Try to convert the row to a typed struct
    pub fn into_typed<T: DeserializeOwned>(self) -> Result<T> {
        let value = serde_json::to_value(self.columns)?;
        serde_json::from_value(value).map_err(Error::from)
    }
}

/// Database query request
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct QueryRequest<'a> {
    sql: &'a str,
    params: Vec<DbValue>,
}

/// Database query response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryResponse {
    rows: Vec<DbRow>,
    #[serde(default)]
    error: Option<String>,
}

/// Database execute response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExecuteResponse {
    rows_affected: i64,
    #[serde(default)]
    last_insert_id: Option<i64>,
    #[serde(default)]
    error: Option<String>,
}

/// Execute a database query and return typed results.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct User {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// let users = db::query::<User>(
///     "SELECT id, name, email FROM users WHERE active = ?",
///     &[true]
/// )?;
/// ```
#[cfg(target_arch = "wasm32")]
pub fn query<T: DeserializeOwned>(sql: &str, params: impl ToDbParams) -> Result<Vec<T>> {
    let request = QueryRequest {
        sql,
        params: params.to_db_params(),
    };

    let request_json = serde_json::to_vec(&request)?;

    let result_ptr = unsafe {
        super::ffi::db_query(
            sql.as_ptr(),
            sql.len() as i32,
            request_json.as_ptr(),
            request_json.len() as i32,
        )
    };

    if result_ptr.is_null() {
        return Err(Error::database("Database query failed"));
    }

    let result_bytes = unsafe { super::ffi::read_length_prefixed(result_ptr) };
    let response: QueryResponse = serde_json::from_slice(&result_bytes)?;

    if let Some(err) = response.error {
        return Err(Error::database(err));
    }

    response
        .rows
        .into_iter()
        .map(|row| row.into_typed())
        .collect()
}

/// Execute a database query (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn query<T: DeserializeOwned>(_sql: &str, _params: impl ToDbParams) -> Result<Vec<T>> {
    Ok(vec![])
}

/// Execute a query and return raw rows (for dynamic queries)
#[cfg(target_arch = "wasm32")]
pub fn query_raw(sql: &str, params: impl ToDbParams) -> Result<Vec<DbRow>> {
    let request = QueryRequest {
        sql,
        params: params.to_db_params(),
    };

    let request_json = serde_json::to_vec(&request)?;

    let result_ptr = unsafe {
        super::ffi::db_query(
            sql.as_ptr(),
            sql.len() as i32,
            request_json.as_ptr(),
            request_json.len() as i32,
        )
    };

    if result_ptr.is_null() {
        return Err(Error::database("Database query failed"));
    }

    let result_bytes = unsafe { super::ffi::read_length_prefixed(result_ptr) };
    let response: QueryResponse = serde_json::from_slice(&result_bytes)?;

    if let Some(err) = response.error {
        return Err(Error::database(err));
    }

    Ok(response.rows)
}

/// Execute a query and return raw rows (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn query_raw(_sql: &str, _params: impl ToDbParams) -> Result<Vec<DbRow>> {
    Ok(vec![])
}

/// Query for a single row
pub fn query_one<T: DeserializeOwned>(sql: &str, params: impl ToDbParams) -> Result<Option<T>> {
    let results = query::<T>(sql, params)?;
    Ok(results.into_iter().next())
}

/// Query for a single required row
pub fn query_one_required<T: DeserializeOwned>(sql: &str, params: impl ToDbParams) -> Result<T> {
    query_one::<T>(sql, params)?.ok_or_else(|| Error::not_found("No rows found"))
}

/// Query for a single scalar value
pub fn query_scalar<T: DeserializeOwned>(sql: &str, params: impl ToDbParams) -> Result<Option<T>> {
    let rows = query_raw(sql, params)?;
    if let Some(row) = rows.into_iter().next() {
        if let Some((_, value)) = row.columns.into_iter().next() {
            return Ok(Some(serde_json::from_value(value)?));
        }
    }
    Ok(None)
}

/// Execute a database mutation (INSERT, UPDATE, DELETE).
///
/// Returns the number of affected rows.
///
/// # Example
///
/// ```rust,ignore
/// let rows = db::execute(
///     "UPDATE users SET last_login = NOW() WHERE id = ?",
///     &[&user_id]
/// )?;
/// ```
#[cfg(target_arch = "wasm32")]
pub fn execute(sql: &str, params: impl ToDbParams) -> Result<i64> {
    let params_json = serde_json::to_vec(&params.to_db_params())?;

    let result = unsafe {
        super::ffi::db_execute(
            sql.as_ptr(),
            sql.len() as i32,
            params_json.as_ptr(),
            params_json.len() as i32,
        )
    };

    if result < 0 {
        return Err(Error::database("Database execute failed"));
    }

    Ok(i64::from(result))
}

/// Execute a database mutation (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn execute(_sql: &str, _params: impl ToDbParams) -> Result<i64> {
    Ok(0)
}

/// Insert a row and return the last insert ID
#[cfg(target_arch = "wasm32")]
pub fn insert_returning_id(sql: &str, params: impl ToDbParams) -> Result<i64> {
    // For PostgreSQL, append RETURNING id
    let returning_sql = if sql.to_uppercase().contains("RETURNING") {
        sql.to_string()
    } else {
        format!("{} RETURNING id", sql)
    };

    query_scalar::<i64>(&returning_sql, params)?
        .ok_or_else(|| Error::database("Insert did not return an ID"))
}

/// Insert a row and return the last insert ID (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn insert_returning_id(_sql: &str, _params: impl ToDbParams) -> Result<i64> {
    Ok(0)
}

/// Transaction builder for multiple operations
pub struct Transaction {
    operations: Vec<(String, Vec<DbValue>)>,
}

impl Transaction {
    /// Create a new transaction builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add an operation to the transaction
    pub fn add(&mut self, sql: impl Into<String>, params: impl ToDbParams) -> &mut Self {
        self.operations.push((sql.into(), params.to_db_params()));
        self
    }

    /// Execute all operations in a transaction
    ///
    /// Note: Actual transaction support depends on host implementation
    pub fn commit(self) -> Result<()> {
        for (sql, params) in self.operations {
            execute(&sql, params.as_slice())?;
        }
        Ok(())
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Start building a transaction
#[inline]
#[must_use]
pub const fn transaction() -> Transaction {
    Transaction::new()
}
