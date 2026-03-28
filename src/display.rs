use embassy_time::Delay;
use embassy_time::Timer;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{Blocking, gpio::Output, spi::master::Spi};
use mipidsi::{interface::SpiInterface, models::ST7789};

pub type MyDisplay = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    Output<'static>,
>;

#[embassy_executor::task]
pub async fn display_task(display: &'static mut MyDisplay) -> ! {
    defmt::info!("display_task started");

    loop {
        display.clear(Rgb565::BLUE).unwrap();
        Timer::after_millis(100).await;
        display_text(display, "Hello,", Rgb565::WHITE).unwrap();
        Timer::after_millis(2000).await;
        display.clear(Rgb565::BLACK).unwrap();
        Timer::after_millis(100).await;
        display_text(display, "from Rust\nno-std!", Rgb565::RED).unwrap();
        Timer::after_millis(2000).await;
    }
}

#[allow(dead_code)]
fn display_text<D>(display: &mut D, message: &str, color: Rgb565) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let text_style = MonoTextStyle::new(&FONT_10X20, color);
    Text::with_baseline(message, Point::new(0, 0), text_style, Baseline::Top).draw(display)?;
    Ok(())
}
