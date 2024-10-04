use image::ImageEncoder;

fn main() -> anyhow::Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    let image = clipboard.get_image()?;
    image::codecs::png::PngEncoder::new(std::io::stdout()).write_image(
        &image.bytes,
        image.width as u32,
        image.height as u32,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(())
}
