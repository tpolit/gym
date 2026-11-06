use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileReadError {
    Unreadable,
}

pub async fn read_file_bytes(file: &web_sys::File) -> Result<Vec<u8>, FileReadError> {
    let buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| FileReadError::Unreadable)?;
    Ok(Uint8Array::new(&buffer).to_vec())
}
