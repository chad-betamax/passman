use anyhow::{Context, Result};
use qrcode::QrCode;
use qrcode::render::unicode;

pub fn print_qr(data: &str) -> Result<()> {
    let code = QrCode::new(data).context("Failed to generate QR code")?;
    let image = code.render::<unicode::Dense1x2>().quiet_zone(false).build();
    println!("{image}");
    Ok(())
}
