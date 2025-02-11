#![allow(unused_imports)]
#![allow(clippy::all)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = MediaMetadata , typescript_type = "MediaMetadata")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `MediaMetadata` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type MediaMetadata;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "MediaMetadata" , js_name = title)]
    #[doc = "Getter for the `title` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/title)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn title(this: &MediaMetadata) -> ::alloc::string::String;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , setter , js_class = "MediaMetadata" , js_name = title)]
    #[doc = "Setter for the `title` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/title)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn set_title(this: &MediaMetadata, value: &str);
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "MediaMetadata" , js_name = artist)]
    #[doc = "Getter for the `artist` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/artist)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn artist(this: &MediaMetadata) -> ::alloc::string::String;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , setter , js_class = "MediaMetadata" , js_name = artist)]
    #[doc = "Setter for the `artist` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/artist)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn set_artist(this: &MediaMetadata, value: &str);
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "MediaMetadata" , js_name = album)]
    #[doc = "Getter for the `album` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/album)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn album(this: &MediaMetadata) -> ::alloc::string::String;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , setter , js_class = "MediaMetadata" , js_name = album)]
    #[doc = "Setter for the `album` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/album)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn set_album(this: &MediaMetadata, value: &str);
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "MediaMetadata" , js_name = artwork)]
    #[doc = "Getter for the `artwork` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/artwork)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn artwork(this: &MediaMetadata) -> ::js_sys::Array;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , setter , js_class = "MediaMetadata" , js_name = artwork)]
    #[doc = "Setter for the `artwork` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/artwork)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn set_artwork(this: &MediaMetadata, value: &::wasm_bindgen::JsValue);
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "MediaMetadata")]
    #[doc = "The `new MediaMetadata(..)` constructor, creating a new instance of `MediaMetadata`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/MediaMetadata)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new() -> Result<MediaMetadata, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "MediaMetadataInit")]
    #[wasm_bindgen(catch, constructor, js_class = "MediaMetadata")]
    #[doc = "The `new MediaMetadata(..)` constructor, creating a new instance of `MediaMetadata`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaMetadata/MediaMetadata)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaMetadata`, `MediaMetadataInit`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_init(init: &MediaMetadataInit) -> Result<MediaMetadata, JsValue>;
}
