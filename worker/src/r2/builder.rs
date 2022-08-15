use std::collections::HashMap;

use js_sys::{Array, JsString, Map, Uint8Array};
use serde_bytes::ByteBuf;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::r2::{
    R2Bucket as EdgeR2Bucket, R2Conditional as R2ConditionalSys, R2GetOptions as R2GetOptionsSys,
    R2HttpMetadata as R2HttpMetadataSys, R2ListOptions as R2ListOptionsSys,
    R2Object as EdgeR2Object, R2PutOptions as R2PutOptionsSys, R2Range as R2RangeSys,
};

use crate::{Date, ObjectInner, Objects, Result};

use super::{Object, R2Data};

/// Options for configuring the [get](crate::r2::Bucket::get) operation.
pub struct GetOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
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
        let name: String = self.key;
        let get_promise = self.edge_bucket.get(
            name,
            R2GetOptionsSys {
                only_f: self.only_if.map(Into::into),
                range: self.range.map(Into::into),
            },
        );

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

/// You can pass an [R2Conditional] object to [GetOptionsBuilder]. If the condition check fails,
/// the body will not be returned. This will make [get](crate::r2::Bucket::get) have lower latency.
/// 
/// For more information about conditional requests, refer to [RFC 7232](https://datatracker.ietf.org/doc/html/rfc7232).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R2Conditional {
    /// Performs the operation if the object’s etag matches the given string.
    pub etag_matches: Option<String>,
    /// Performs the operation if the object’s etag does not match the given string.
    pub etag_does_not_match: Option<String>,
    /// Performs the operation if the object was uploaded before the given date.
    pub uploaded_before: Option<Date>,
    /// Performs the operation if the object was uploaded after the given date.
    pub uploaded_after: Option<Date>,
}

impl From<R2Conditional> for R2ConditionalSys {
    fn from(val: R2Conditional) -> Self {
        R2ConditionalSys {
            etag_matches: val.etag_matches,
            etag_does_not_match: val.etag_does_not_match,
            uploaded_before: val.uploaded_before.map(Into::into),
            uploaded_after: val.uploaded_after.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R2Range {
    OffsetWithOptionalLength { offset: i64, length: Option<i64> },
    OptionalOffsetWithLength { offset: Option<i64>, length: i64 },
    Suffix { suffix: i64 },
}

impl From<R2Range> for R2RangeSys {
    fn from(val: R2Range) -> Self {
        match val {
            R2Range::OffsetWithOptionalLength { offset, length } => R2RangeSys {
                offset: Some(offset),
                length,
                suffix: None,
            },
            R2Range::OptionalOffsetWithLength { offset, length } => R2RangeSys {
                offset,
                length: Some(length),
                suffix: None,
            },
            R2Range::Suffix { suffix } => R2RangeSys {
                offset: None,
                length: None,
                suffix: Some(suffix),
            },
        }
    }
}

/// Options for configuring the [put](crate::r2::Bucket::put) operation.
pub struct PutOptionsBuilder<'bucket> {
    pub(crate) edge_bucket: &'bucket EdgeR2Bucket,
    pub(crate) key: String,
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
        let value: JsValue = self.value.into();
        let name: String = self.key;

        let put_promise = self.edge_bucket.put(
            name,
            value,
            R2PutOptionsSys {
                http_metadata: self.http_metadata.map(Into::into),
                custom_metadata: match self.custom_metadata {
                    Some(metadata) => {
                        let map = Map::new();
                        for (k, v) in metadata.into_iter() {
                            map.set(&JsString::from(k), &JsString::from(v));
                        }
                        map.into()
                    }
                    None => JsValue::undefined(),
                },
                md5: self.md5.map(|bytes| {
                    let arr = Uint8Array::new_with_length(bytes.len() as _);
                    arr.copy_from(&bytes);
                    arr.buffer()
                }),
            },
        );
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub cache_expiry: Option<Date>,
}

impl From<HttpMetadata> for R2HttpMetadataSys {
    fn from(val: HttpMetadata) -> Self {
        R2HttpMetadataSys {
            content_type: val.content_type,
            content_language: val.content_language,
            content_disposition: val.content_disposition,
            content_encoding: val.content_encoding,
            cache_control: val.cache_control,
            cache_expiry: val.cache_expiry.map(Into::into),
        }
    }
}

/// Options for configuring the [list](crate::r2::Bucket::list) operation.
pub struct ListOptionsBuilder<'bucket> {
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
        let list_promise = self.edge_bucket.list(R2ListOptionsSys {
            limit: self.limit,
            prefix: self.prefix,
            cursor: self.cursor,
            delimiter: self.delimiter,
            include: self
                .include
                .map(|include| {
                    let arr = Array::new_with_length(include.len() as _);
                    for include in include {
                        arr.push(&JsString::from(match include {
                            Include::HttpMetadata => "httpMetadata",
                            Include::CustomMetadata => "customMetadata",
                        }));
                    }
                    arr.into()
                })
                .unwrap_or(JsValue::UNDEFINED),
        });
        let inner = JsFuture::from(list_promise).await?.into();
        Ok(Objects { inner })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Include {
    HttpMetadata,
    CustomMetadata,
}
