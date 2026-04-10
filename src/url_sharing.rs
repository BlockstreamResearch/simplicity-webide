use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use leptos::wasm_bindgen::JsValue;
use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec_with_limit;
use web_sys::window;

const URL_PREFIX: &str = "#code=";
const MAX_DECOMPRESSED_SIZE: usize = 65_536;

fn encode_program(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let compressed = compress_to_vec(text.as_bytes(), 6);
    Some(URL_SAFE_NO_PAD.encode(&compressed))
}

fn decode_program(encoded: &str) -> Option<String> {
    let compressed = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let decompressed = decompress_to_vec_with_limit(&compressed, MAX_DECOMPRESSED_SIZE).ok()?;
    String::from_utf8(decompressed).ok()
}

pub fn build_share_url(text: &str) -> Option<String> {
    let encoded = encode_program(text)?;
    let window = window()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let pathname = location.pathname().ok()?;
    Some(format!("{origin}{pathname}{URL_PREFIX}{encoded}"))
}

pub fn read_shared_program() -> Option<Result<String, ()>> {
    let hash = window()?.location().hash().ok()?;
    let encoded = hash.strip_prefix(URL_PREFIX)?;
    Some(decode_program(encoded).ok_or(()))
}

pub fn set_url_hash(text: &str) {
    let Some(encoded) = encode_program(text) else {
        return;
    };
    let hash = format!("{URL_PREFIX}{encoded}");
    let _ = window().and_then(|w| w.history().ok()).and_then(|h| {
        h.replace_state_with_url(&JsValue::NULL, "", Some(&hash))
            .ok()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn roundtrip_simple() {
        let text = "fn main() {\n    jet::bip_0340_verify((pk, msg), sig)\n}";
        let encoded = encode_program(text).unwrap();
        let decoded = decode_program(&encoded).unwrap();
        assert_eq!(text, decoded);
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn roundtrip_empty() {
        assert!(encode_program("").is_none());
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn roundtrip_large_program() {
        let text = "fn main() {\n".to_string()
            + &"    let x: u256 = jet::sig_all_hash();\n".repeat(100)
            + "}";
        let encoded = encode_program(&text).unwrap();
        let decoded = decode_program(&encoded).unwrap();
        assert_eq!(text, decoded);
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn decode_invalid_base64() {
        assert!(decode_program("!!!not-valid-base64!!!").is_none());
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn decode_invalid_compression() {
        let not_deflate = URL_SAFE_NO_PAD.encode(b"this is not deflate data");
        assert!(decode_program(&not_deflate).is_none());
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn encoded_is_url_safe() {
        let text = "fn main() { let x: u256 = 0xff; }";
        let encoded = encode_program(text).unwrap();
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn roundtrip_all_examples() {
        for name in crate::examples::keys() {
            let example = crate::examples::get(name).unwrap();
            let text = example.template_text();
            let encoded = encode_program(text).unwrap();
            let decoded = decode_program(&encoded).unwrap();
            assert_eq!(text, decoded);
        }
    }
}
