use std::collections::HashMap;

use js_sys::JsString;
use serde::Serialize;
use serde_bytes::ByteBuf;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::r2::{R2Bucket as EdgeR2Bucket, R2Object as EdgeR2Object};

use crate::{Date, Error, ObjectInner, Objects, Result};

use super::{Object, R2Data};

/// Options for configuring the [get](crate::r2::Bucket::get) operation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOptionsBuilder<'bucket> {
    #[serde(skip)]
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    #[serde(skip)]
    pub(crate) key: String,
    pub(crate) only_if: Option<R2Conditional>,
    pub(crate) range: Option<R2Range>,
}

impl<'bucket> GetOptionsBuilder<'bucket> {
    /// Specifies that the object should only be returned given satisfaction of certain conditions
    /// in the [R2Conditional]. Refer to [Conditional operations](https://developers.cloudflare.com/r2/runtime-apis/#conditional-operations).
    pub fn only_if(mut self, only_if: R2Conditional) -> Self {
        self.only_if = Some(only_if);
        self
    }

    /// Specifies that only a specific length (from an optional offset) or suffix of bytes from the
    /// object should be returned. Refer to [Ranged reads](https://developers.cloudflare.com/r2/runtime-apis/#ranged-reads).
    pub fn range(mut self, range: R2Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Executes the GET operation on the R2 bucket.
    pub async fn execute(self) -> Result<Option<Object>> {
        let options = serde_wasm_bindgen::to_value(&self).map_err(|e| Error::Internal(e.into()))?;
        let name: String = self.key;

        let get_promise = self.edge_bucket.get(name, options);
        let value = JsFuture::from(get_promise).await?;

        if value.is_null() {
            return Ok(None);
        }

        let res: EdgeR2Object = value.into();
        let inner = if JsString::from("bodyUsed").js_in(&res) {
            ObjectInner::Body(res.unchecked_into())
        } else {
            ObjectInner::NoBody(res)
        };

        Ok(Some(Object { inner }))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R2Conditional {
    pub(crate) etag_atches: Option<String>,
    pub(crate) etag_does_not_match: Option<String>,
    // TODO(zeb): implement serializer for date.
    #[serde(skip)]
    pub(crate) uploaded_before: Option<Date>,
    // TODO(zeb): implement serializer for date.
    #[serde(skip)]
    pub(crate) uploaded_after: Option<Date>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum R2Range {
    OffsetWithOptionalLength { offset: i64, length: Option<i64> },
    OptionalOffsetWithLength { offset: Option<i64>, length: i64 },
    Suffix { suffix: i64 },
}

/// Options for configuring the [put](crate::r2::Bucket::put) operation.
#[derive(Serialize)]
pub struct PutOptionsBuilder<'bucket> {
    #[serde(skip)]
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    #[serde(skip)]
    pub(crate) key: String,
    #[serde(skip)]
    pub(crate) value: R2Data,
    pub(crate) http_metadata: Option<HttpMetadata>,
    pub(crate) custom_metadata: Option<HashMap<String, String>>,
    pub(crate) md5: Option<ByteBuf>,
}

impl<'bucket> PutOptionsBuilder<'bucket> {
    /// Various HTTP headers associated with the object. Refer to [HttpMetadata].
    pub fn http_metadata(mut self, metadata: HttpMetadata) -> Self {
        self.http_metadata = Some(metadata);
        self
    }

    /// A map of custom, user-defined metadata that will be stored with the object.
    pub fn custom_metdata(mut self, metadata: impl Into<HashMap<String, String>>) -> Self {
        self.custom_metadata = Some(metadata.into());
        self
    }

    /// A md5 hash to use to check the recieved object’s integrity.
    pub fn md5(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.md5 = Some(ByteBuf::from(bytes));
        todo!()
    }

    /// Executes the PUT operation on the R2 bucket.
    pub async fn execute(self) -> Result<Object> {
        let options = serde_wasm_bindgen::to_value(&self).map_err(|e| Error::Internal(e.into()))?;
        let value: JsValue = self.value.into();
        let name: String = self.key;

        let put_promise = self.edge_bucket.put(name, value, options);
        let res: EdgeR2Object = JsFuture::from(put_promise).await?.into();
        let inner = if JsString::from("bodyUsed").js_in(&res) {
            ObjectInner::Body(res.unchecked_into())
        } else {
            ObjectInner::NoBody(res)
        };

        Ok(Object { inner })
    }
}

/// Metadata that's automatically rendered into R2 HTTP API endpoints.
/// ```
/// * contentType -> content-type
/// * contentLanguage -> content-language
/// etc...
/// ```
/// This data is echoed back on GET responses based on what was originally
/// assigned to the object (and can typically also be overriden when issuing
/// the GET request).
#[derive(Debug, Serialize)]
pub struct HttpMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    // TODO(zeb): implement serializer for date.
    #[serde(skip)]
    pub cache_expiry: Option<Date>,
}

/// Options for configuring the [list](crate::r2::Bucket::list) operation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOptionsBuilder<'bucket> {
    #[serde(skip)]
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) limit: Option<i64>,
    pub(crate) prefix: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) include: Option<Vec<Include>>,
}

impl<'bucket> ListOptionsBuilder<'bucket> {
    /// The number of results to return. Defaults to 1000, with a maximum of 1000.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// The prefix to match keys against. Keys will only be returned if they start with given prefix.
    pub fn prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// An opaque token that indicates where to continue listing objects from. A cursor can be
    /// retrieved from a previous list operation.
    pub fn cursor(mut self, cursor: String) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// The character to use when grouping keys.
    pub fn delimiter(mut self, delimiter: String) -> Self {
        self.delimiter = Some(delimiter);
        self
    }

    /// If you populate this array, then items returned will include this metadata.
    /// A tradeoff is that fewer results may be returned depending on how big this
    /// data is. For now the caps are TBD but expect the total memory usage for a list
    /// operation may need to be <1MB or even <128kb depending on how many list operations
    /// you are sending into one bucket. Make sure to look at `truncated` for the result
    /// rather than having logic like
    ///
    /// ```no_run
    /// while listed.len() < limit {
    ///     listed = bucket.list()
    ///         .limit(limit),
    ///         .include(vec![Include::CustomMetadata])
    ///         .execute()
    ///         .await?;
    /// }
    /// ```
    pub fn include(mut self, include: Vec<Include>) -> Self {
        self.include = Some(include);
        self
    }

    /// Executes the LIST operation on the R2 bucket.
    pub async fn execute(self) -> Result<Objects> {
        let options = serde_wasm_bindgen::to_value(&self).map_err(|e| Error::Internal(e.into()))?;
        let list_promise = self.edge_bucket.list(options);
        let inner = JsFuture::from(list_promise).await?.into();
        Ok(Objects { inner })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Include {
    HttpMetadata,
    CustomMetadata,
}
