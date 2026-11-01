pub fn prepare_photo(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(crate::core::i18n::Language::system().text("workspace-choose-an-image-up-to-10-mb").to_string());
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| crate::core::i18n::Language::system().format("services-invalid-image", &[("e", &e.to_string())]))?
        .resize_to_fill(192, 192, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut result = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut result, 85)
        .encode_image(&image)
        .map_err(|e| e.to_string())?;
    Ok(result)
}
