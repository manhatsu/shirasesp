use core::fmt::Write;

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::Rgb565,
    prelude::*,
};

const IMAGE_WIDTH: u32 = 100;
const IMAGE_START_X: i32 = 10;
const IMAGE_START_Y: i32 = 70;
const SUMMARY_TEXT_X: i32 = 70;
const SUMMARY_TEXT_Y: i32 = 70;
const NONOTI_TEXT_X: i32 = 30;
const NONOTI_TEXT_Y: i32 = 30;

#[derive(Clone, Copy, Debug)]
pub struct NotificationSummary {
    pub slack_count: u32,
    pub gmail_count: u32,
    pub gcal_count: u32,
}

impl NotificationSummary {
    pub fn total_count(&self) -> u32 {
        self.slack_count + self.gmail_count + self.gcal_count
    }
}

pub trait DisplayText {
    fn text(&self) -> &str;
    fn color(&self) -> Rgb565;
    fn position(&self) -> Point;
}

pub struct SummaryTextObject {
    buf: FmtBuf,
    pub color: Rgb565,
    pub position: Point,
}

impl DisplayText for SummaryTextObject {
    fn text(&self) -> &str {
        self.buf.as_str()
    }
    fn color(&self) -> Rgb565 {
        self.color
    }
    fn position(&self) -> Point {
        self.position
    }
}

impl Default for SummaryTextObject {
    fn default() -> Self {
        Self::new()
    }
}

impl SummaryTextObject {
    pub fn new() -> Self {
        let mut buf = FmtBuf::new();
        let _ = write!(buf, "Slk:0\nGml:0\nGcl:0");
        Self {
            buf,
            color: Rgb565::WHITE,
            position: Point::new(SUMMARY_TEXT_X, SUMMARY_TEXT_Y),
        }
    }

    pub fn update_counts(&mut self, summary: &NotificationSummary) {
        self.buf.reset();
        let _ = write!(
            self.buf,
            "Slk:{}\nGml:{}\nGcl:{}",
            summary.slack_count, summary.gmail_count, summary.gcal_count
        );
    }
}

pub struct TextObject<'a> {
    pub text: &'a str,
    pub color: Rgb565,
    pub position: Point,
}

impl DisplayText for TextObject<'_> {
    fn text(&self) -> &str {
        self.text
    }
    fn color(&self) -> Rgb565 {
        self.color
    }
    fn position(&self) -> Point {
        self.position
    }
}

pub const NONOTI_TEXT: TextObject<'static> = TextObject {
    text: "No Noti",
    color: Rgb565::WHITE,
    position: Point::new(NONOTI_TEXT_X, NONOTI_TEXT_Y),
};

pub const SUMMARY_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/summary.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);
pub const SLACK_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/slack.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);
pub const GMAIL_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/gmail.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);
pub const GCAL_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/gcal.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);

pub const NONOTI_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/nonoti.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);

pub const ERROR_IMAGE: Image<ImageRaw<Rgb565>> = Image::new(
    &ImageRaw::<Rgb565>::new(include_bytes!("../images/error.raw"), IMAGE_WIDTH),
    Point::new(IMAGE_START_X, IMAGE_START_Y),
);

struct FmtBuf {
    buf: [u8; 64],
    pos: usize,
}

impl FmtBuf {
    fn new() -> Self {
        Self {
            buf: [0u8; 64],
            pos: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }

    fn reset(&mut self) {
        self.pos = 0;
    }
}

impl Write for FmtBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let len = bytes.len().min(remaining);
        self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
        self.pos += len;
        Ok(())
    }
}
