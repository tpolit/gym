use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadError {
    NoDocument,
    BrowserRefused,
}

// Saves bytes as a file through a temporary link; on iOS this opens the share sheet.
pub fn download_file(file_name: &str, bytes: &[u8], mime_type: &str) -> Result<(), DownloadError> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or(DownloadError::NoDocument)?;
    let parts = Array::of1(&Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime_type);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|_| DownloadError::BrowserRefused)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| DownloadError::BrowserRefused)?;
    let link: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| DownloadError::BrowserRefused)?
        .dyn_into()
        .map_err(|_| DownloadError::BrowserRefused)?;
    link.set_href(&url);
    link.set_download(file_name);
    link.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}
