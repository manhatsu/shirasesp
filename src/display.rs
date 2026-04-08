use crate::{
    contents::{DisplayText, *},
    mqtt::Notification,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Delay;
use embassy_time::Timer;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{Async, gpio::Output, spi::master::SpiDmaBus};
use mipidsi::{interface::SpiInterface, models::ST7789};

const DEFAULT_DISPLAY_COLOR: Rgb565 = Rgb565::BLACK;
const NOTIFICATION_HOLD_DURATION_MS: u64 = 3000;

pub enum DisplayEvent {
    Notification(Notification),
    Summary(NotificationSummary),
    NoNotification,
    Error,
}

pub static DISPLAY_CHANNEL: Channel<CriticalSectionRawMutex, DisplayEvent, 8> = Channel::new();

pub type MyDisplay = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<SpiDmaBus<'static, Async>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    Output<'static>,
>;

#[embassy_executor::task]
pub async fn display_task(display: &'static mut MyDisplay) -> ! {
    let mut notification_summary = NotificationSummary {
        slack_count: 0,
        gmail_count: 0,
        gcal_count: 0,
    };

    let mut summary_text_object = SummaryTextObject::default();

    display.clear(DEFAULT_DISPLAY_COLOR).unwrap();

    NONOTI_IMAGE.draw(&mut *display).unwrap();
    display_text(display, &NONOTI_TEXT).unwrap();

    let mut pending_event: Option<DisplayEvent> = None;

    loop {
        let event = match pending_event.take() {
            Some(e) => e,
            None => DISPLAY_CHANNEL.receive().await,
        };
        defmt::info!("Received display event");
        match event {
            DisplayEvent::Notification(notification) => {
                let image = match notification.service {
                    crate::mqtt::ServiceType::Slack => {
                        notification_summary.slack_count = notification.count;
                        SLACK_IMAGE
                    }
                    crate::mqtt::ServiceType::Gmail => {
                        notification_summary.gmail_count = notification.count;
                        GMAIL_IMAGE
                    }
                    crate::mqtt::ServiceType::GoogleCalendar => {
                        notification_summary.gcal_count = notification.count;
                        GCAL_IMAGE
                    }
                };
                display.clear(DEFAULT_DISPLAY_COLOR).unwrap();
                image.draw(&mut *display).unwrap();
                Timer::after_millis(NOTIFICATION_HOLD_DURATION_MS).await;

                match DISPLAY_CHANNEL.try_receive() {
                    Ok(next) => pending_event = Some(next),
                    Err(_) => {
                        if notification_summary.total_count() > 0 {
                            DISPLAY_CHANNEL
                                .send(DisplayEvent::Summary(notification_summary))
                                .await;
                        } else {
                            DISPLAY_CHANNEL.send(DisplayEvent::NoNotification).await;
                        }
                    }
                }
                continue;
            }
            DisplayEvent::Summary(summary) => {
                display.clear(DEFAULT_DISPLAY_COLOR).unwrap();
                SUMMARY_IMAGE.draw(&mut *display).unwrap();

                summary_text_object.update_counts(&summary);

                display_text(display, &summary_text_object).unwrap();
            }
            DisplayEvent::NoNotification => {
                display.clear(DEFAULT_DISPLAY_COLOR).unwrap();
                NONOTI_IMAGE.draw(&mut *display).unwrap();
                display_text(display, &NONOTI_TEXT).unwrap();
            }
            DisplayEvent::Error => {
                display.clear(DEFAULT_DISPLAY_COLOR).unwrap();
                ERROR_IMAGE.draw(&mut *display).unwrap();
            }
        }
        Timer::after_millis(NOTIFICATION_HOLD_DURATION_MS).await;
        display.clear(DEFAULT_DISPLAY_COLOR).unwrap();
    }
}

fn display_text<D>(display: &mut D, text_object: &impl DisplayText) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let text_style = MonoTextStyle::new(&FONT_10X20, text_object.color());
    Text::with_baseline(
        text_object.text(),
        text_object.position(),
        text_style,
        Baseline::Top,
    )
    .draw(display)?;
    Ok(())
}
