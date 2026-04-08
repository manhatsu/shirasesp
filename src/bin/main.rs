#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    Async,
    clock::CpuClock,
    dma::{DmaRxBuf, DmaTxBuf},
    gpio::{Level, Output, OutputConfig},
    rng::Rng,
    spi::{
        Mode,
        master::{Config as SpiConfig, Spi},
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder},
};
use static_cell::StaticCell;
use {esp_backtrace as _, esp_println as _};

use shirasesp::{
    display::{MyDisplay, display_task},
    mqtt::{self},
    wifi::{self, setup_wifi},
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.1.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let dma_channel = peripherals.DMA_SPI2;
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = esp_hal::dma_buffers!(512);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(2))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO13)
    .with_mosi(peripherals.GPIO15)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let _back_light = Output::new(peripherals.GPIO27, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO12, Level::High, OutputConfig::default());

    let spi_device: ExclusiveDevice<esp_hal::spi::master::SpiDmaBus<'_, Async>, Output<'_>, Delay> =
        ExclusiveDevice::new(spi_bus, cs, Delay).unwrap();

    static SPI_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    let spi_buffer = SPI_BUF.init([0u8; 512]);
    let di = SpiInterface::new(spi_device, dc, spi_buffer);

    static DISPLAY: StaticCell<MyDisplay> = StaticCell::new();
    let display = DISPLAY.init(
        Builder::new(ST7789, di)
            .reset_pin(rst)
            .display_size(135, 240)
            .display_offset(52, 40)
            .color_order(ColorOrder::Rgb)
            .invert_colors(ColorInversion::Inverted)
            .init(&mut Delay)
            .unwrap(),
    );

    spawner.spawn(display_task(display)).unwrap();

    let rng = Rng::new();
    let (controller, stack, runner) = setup_wifi(peripherals.WIFI, rng);

    wifi::spawn_tasks(&spawner, controller, runner).await;
    mqtt::spawn_tasks(&spawner, stack).await;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}
