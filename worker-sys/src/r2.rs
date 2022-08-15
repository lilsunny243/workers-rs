use js_sys::JsString;
use wasm_bindgen::prelude::*;
use web_sys::ReadableStream;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Bucket)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Bucket;

    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = head)]
    pub fn head(this: &R2Bucket, key: String) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = get)]
    pub fn get(this: &R2Bucket, key: String, options: R2GetOptions) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = put)]
    pub fn put(
        this: &R2Bucket,
        key: String,
        value: JsValue,
        options: R2PutOptions,
    ) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = delete)]
    pub fn delete(this: &R2Bucket, key: String) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = list)]
    pub fn list(this: &R2Bucket, options: R2ListOptions) -> ::js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Object;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=R2Object, js_name=R2ObjectBody)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2ObjectBody;

    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name = body)]
    pub fn body(this: &R2ObjectBody) -> ReadableStream;
    #[wasm_bindgen(structural, method, getter, js_class=R2ObjectBody, js_name = bodyUsed)]
    pub fn body_used(this: &R2ObjectBody) -> bool;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=R2Objects)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type R2Objects;

    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = objects)]
    pub fn objects(this: &R2Objects) -> Vec<R2Object>;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = truncated)]
    pub fn truncated(this: &R2Objects) -> bool;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = cursor)]
    pub fn cursor(this: &R2Objects) -> Option<String>;
    #[wasm_bindgen(structural, method, getter, js_class=R2Objects, js_name = delimitedPrefixes)]
    pub fn delimited_prefixes(this: &R2Objects) -> Vec<JsString>;
}

#[wasm_bindgen(getter_with_clone)]
pub struct R2GetOptions {
    #[wasm_bindgen(js_name = "onlyIf")]
    pub only_f: Option<R2Conditional>,
    pub range: Option<R2Range>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct R2Conditional {
    #[wasm_bindgen(js_name = "etagMatches")]
    pub etag_matches: Option<String>,
    #[wasm_bindgen(js_name = "etagDoesNotMatch")]
    pub etag_does_not_match: Option<String>,
    #[wasm_bindgen(js_name = "uploadedBefore")]
    pub uploaded_before: Option<::js_sys::Date>,
    #[wasm_bindgen(js_name = "uploadedAfter")]
    pub uploaded_after: Option<::js_sys::Date>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct R2Range {
    pub offset: Option<i64>,
    pub length: Option<i64>,
    pub suffix: Option<i64>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct R2PutOptions {
    #[wasm_bindgen(js_name = "httpMetadata")]
    pub http_metadata: Option<R2HttpMetadata>,
    #[wasm_bindgen(js_name = "customMetadata")]
    pub custom_metadata: JsValue,
    pub md5: Option<::js_sys::ArrayBuffer>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct R2HttpMetadata {
    #[wasm_bindgen(js_name = "contentType")]
    pub content_type: Option<String>,
    #[wasm_bindgen(js_name = "contentLanguage")]
    pub content_language: Option<String>,
    #[wasm_bindgen(js_name = "contentDisposition")]
    pub content_disposition: Option<String>,
    #[wasm_bindgen(js_name = "contentEncoding")]
    pub content_encoding: Option<String>,
    #[wasm_bindgen(js_name = "cacheControl")]
    pub cache_control: Option<String>,
    #[wasm_bindgen(js_name = "cacheExpiry")]
    pub cache_expiry: Option<::js_sys::Date>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct R2ListOptions {
    pub limit: Option<i64>,
    pub prefix: Option<String>,
    pub cursor: Option<String>,
    pub delimiter: Option<String>,
    pub include: JsValue,
}