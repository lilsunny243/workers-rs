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
    pub fn get(this: &R2Bucket, key: String, options: JsValue) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = put)]
    pub fn put(this: &R2Bucket, key: String, value: JsValue, options: JsValue)
        -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = delete)]
    pub fn delete(this: &R2Bucket, key: String) -> ::js_sys::Promise;
    #[wasm_bindgen(structural, method, js_class=R2Bucket, js_name = list)]
    pub fn list(this: &R2Bucket, options: JsValue) -> ::js_sys::Promise;
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
