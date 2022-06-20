pub use builder::*;

use futures_util::{stream::BoxStream, Stream, TryStreamExt};
use js_sys::{JsString, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::r2::{
    R2Bucket as EdgeR2Bucket, R2Object as EdgeR2Object, R2ObjectBody as EdgeR2ObjectBody,
    R2Objects as EdgeR2Objects,
};

use crate::{ByteStream, Error, Result};

mod builder;

/// An instance of the R2 bucket binding.
pub struct Bucket {
    inner: EdgeR2Bucket,
}

impl Bucket {
    /// Retrieves the [Object] for the given key containing only object metadata, if the key exists.
    pub async fn head(&self, key: impl Into<String>) -> Result<Option<Object>> {
        let head_promise = self.inner.head(key.into());
        let value = JsFuture::from(head_promise).await?;

        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(Object {
            inner: ObjectInner::NoBody(value.into()),
        }))
    }

    /// Retrieves the [Object] for the given key containing object metadata and the object body if
    /// the key exists. In the event that a precondition specified in options fails, get() returns
    /// an [Object] with no body.
    pub fn get(&self, key: impl Into<String>) -> GetOptionsBuilder {
        GetOptionsBuilder {
            edge_bucket: &self.inner,
            key: key.into(),
            only_if: None,
            range: None,
        }
    }

    /// Stores the given `value` and metadata under the associated `key`. Once the write succeeds,
    /// returns an [Object] containing metadata about the stored Object.
    ///
    /// R2 writes are strongly consistent. Once the future resolves, all subsequent read operations
    /// will see this key value pair globally.
    pub fn put(&self, key: impl Into<String>, value: impl Into<R2Data>) -> PutOptionsBuilder {
        PutOptionsBuilder {
            edge_bucket: &self.inner,
            key: key.into(),
            value: value.into(),
            http_metadata: None,
            custom_metadata: None,
            md5: None,
        }
    }

    /// Deletes the given value and metadata under the associated key. Once the delete succeeds,
    /// returns void.
    ///
    /// R2 deletes are strongly consistent. Once the Promise resolves, all subsequent read
    /// operations will no longer see this key value pair globally.
    pub async fn delete(&self, key: impl Into<String>) -> Result<()> {
        let delete_promise = self.inner.delete(key.into());
        JsFuture::from(delete_promise).await?;
        Ok(())
    }

    /// Returns an [Objects] containing a list of [Objects]s contained within the bucket. By
    /// default, returns the first 1000 entries.
    pub fn list(&self) -> ListOptionsBuilder {
        ListOptionsBuilder {
            edge_bucket: &self.inner,
            limit: None,
            prefix: None,
            cursor: None,
            delimiter: None,
            include: None,
        }
    }
}

/// [Object] is created when you [put](Bucket::put) an object into a [Bucket]. [Object] represents
/// the metadata of an object based on the information provided by the uploader. Every object that
/// you [put](Bucket::put) into a [Bucket] will have an [Object] created.
pub struct Object {
    inner: ObjectInner,
}

impl Object {
    pub fn body(&self) -> Option<ObjectBody> {
        match &self.inner {
            ObjectInner::NoBody(_) => None,
            ObjectInner::Body(body) => Some(ObjectBody { inner: body }),
        }
    }
}

/// The data contained within an [Object].
pub struct ObjectBody<'body> {
    inner: &'body EdgeR2ObjectBody,
}

impl<'body> ObjectBody<'body> {
    /// Reads the data in the [Object] via a [ByteStream].
    pub fn stream(self) -> Result<ByteStream> {
        if self.inner.body_used() {
            return Err(Error::BodyUsed);
        }

        let stream = self.inner.body();
        let stream = wasm_streams::ReadableStream::from_raw(stream.unchecked_into());
        Ok(ByteStream {
            inner: stream.into_stream(),
        })
    }
}

/// A series of [Object]s returned by [list](Bucket::list).
pub struct Objects {
    inner: EdgeR2Objects,
}

impl Objects {
    /// An [Vec] of [Object] matching the [list](Bucket::list) request.
    pub fn objects(&self) -> Vec<Object> {
        self.inner
            .objects()
            .into_iter()
            .map(|raw| Object {
                inner: ObjectInner::NoBody(raw),
            })
            .collect()
    }

    /// If true, indicates there are more results to be retrieved for the current
    /// [list](Bucket::list) request.
    pub fn truncated(&self) -> bool {
        self.inner.truncated()
    }

    /// A token that can be passed to future [list](Bucket::list) calls to resume listing from that
    /// point. Only present if truncated is true.
    pub fn cursor(&self) -> Option<String> {
        self.inner.cursor()
    }

    /// If a delimiter has been specified, contains all prefixes between the specified prefix and
    /// the next occurence of the delimiter.
    ///
    /// For example, if no prefix is provided and the delimiter is '/', `foo/bar/baz` would return
    /// `foo` as a delimited prefix. If `foo/` was passed as a prefix with the same structure and
    /// delimiter, `foo/bar` would be returned as a delimited prefix.
    pub fn delimited_prefixes(&self) -> Vec<String> {
        self.inner
            .delimited_prefixes()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

#[derive(Clone)]
pub(crate) enum ObjectInner {
    NoBody(EdgeR2Object),
    Body(EdgeR2ObjectBody),
}

pub enum R2Data {
    Stream(BoxStream<'static, Result<Vec<u8>>>),
    Text(String),
    Bytes(Vec<u8>),
    None,
}

impl R2Data {
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Vec<u8>>>,
        S: Send + 'static,
    {
        let stream = Box::pin(stream);
        R2Data::Stream(stream)
    }
}

impl From<String> for R2Data {
    fn from(value: String) -> Self {
        R2Data::Text(value)
    }
}

impl From<Vec<u8>> for R2Data {
    fn from(value: Vec<u8>) -> Self {
        R2Data::Bytes(value)
    }
}

impl From<R2Data> for JsValue {
    fn from(data: R2Data) -> Self {
        match data {
            R2Data::Stream(stream) => {
                let js_stream = stream
                    .map_ok(|chunk| {
                        let array = Uint8Array::new_with_length(chunk.len() as _);
                        array.copy_from(&chunk);

                        array.into()
                    })
                    .map_err(|err| -> crate::Error { err })
                    .map_err(|e| JsValue::from(e.to_string()));

                let stream = wasm_streams::ReadableStream::from_stream(js_stream);
                stream.into_raw().into()
            }
            R2Data::Text(text) => JsString::from(text).into(),
            R2Data::Bytes(bytes) => {
                let arr = Uint8Array::new_with_length(bytes.len() as u32);
                arr.copy_from(&bytes);
                arr.into()
            }
            R2Data::None => JsValue::UNDEFINED,
        }
    }
}
