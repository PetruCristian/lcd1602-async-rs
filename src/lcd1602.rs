use crate::error::Error;
use crate::LCD1602;

use crate::error::Error::{InvalidCursorPos, UnsupportedBusWidth};
use crate::lcd1602::BusWidth::FourBits;
use crate::lcd1602::Direction::RightToLeft;
use embassy_time::{Duration, Timer};
use embassy_rp::gpio::Output;

impl<EN, RS, D4, D5, D6, D7, E> LCD1602<EN, RS, D4, D5, D6, D7>
where
    EN: Output<Error = E>,
    RS: Output<Error = E>,
    D4: Output<Error = E>,
    D5: Output<Error = E>,
    D6: Output<Error = E>,
    D7: Output<Error = E>,
{
    pub async fn new(
        en: EN,
        rs: RS,
        d4: D4,
        d5: D5,
        d6: D6,
        d7: D7,
    ) -> Result<LCD1602<EN, RS, D4, D5, D6, D7>, Error<E>> {
        let mut lcd = LCD1602 {
            en,
            rs,
            d4,
            d5,
            d6,
            d7,
        };
        lcd.init();
        Ok(lcd)
    }

    async fn init(&mut self) {
        self.delay(50000);
        self.set_bus_width(FourBits);

        self.command(0x0C); // Display mode
        self.clear();
        self.set_entry_mode(RightToLeft, false);
    }

    pub async fn set_bus_width(&mut self, bus_width: BusWidth) {
        match bus_width {
            FourBits => {
                self.write_bus(0x02);
                self.delay(39);
            }
            _ => (),
        }
    }
    pub async fn set_entry_mode(
        &mut self,
        text_direction: Direction,
        screen_edge_tracking: bool,
    ) {
        let mut cmd = 0x04;
        if text_direction == Direction::RightToLeft {
            cmd |= 0x02;
        }
        if screen_edge_tracking {
            cmd |= 0x01;
        }
        self.command(cmd);
        self.delay(39);
    }

    pub async fn set_position(
        &mut self,
        x: u8,
        y: u8
    ) {
        match (x,y) {
            (0..=15, 0) => {
                self.command(0x80 | x);
                self.delay(1530);
            },
            (0..=15, 1) => {
                self.command(0x80 | (x + 0x40));
                self.delay(1530);
            },
            _ => ()
        }
    }

    pub async fn clear(&mut self) {
        self.command(0x01);
        self.delay(1530);
    }

    pub async fn home(&mut self) {
        self.command(0x02);
        self.delay(1530);
    }

    async fn command(&mut self, cmd: u8) {
        self.rs.set_low();
        self.write_bus((cmd & 0xF0) >> 4);
        self.write_bus(cmd & 0x0F); // 4bit writes send end pulses
    }

    async fn write_char(&mut self, ch: u8) {
        self.rs.set_high();
        self.write_bus((ch & 0xF0) >> 4);
        self.write_bus(ch & 0x0F); // 4bit writes send end pulses
    }

    pub async fn print(&mut self, s: &str) {
        for ch in s.chars() {
            self.delay(320); // per char delay
            self.write_char(ch as u8);
        }
        self.delay(1530);
    }

    async fn write_bus(&mut self, data: u8) {
        self.en.set_low();
        match (data & 0x1) > 0 {
            true => self.d4.set_high(),
            false => self.d4.set_low(),
        };
        match (data & 0x2) > 0 {
            true => self.d5.set_high(),
            false => self.d5.set_low(),
        };
        match (data & 0x4) > 0 {
            true => self.d6.set_high(),
            false => self.d6.set_low(),
        };
        match (data & 0x8) > 0 {
            true => self.d7.set_high(),
            false => self.d7.set_low(),
        };
        self.en.set_high();
        self.en.set_low();
    }

    pub async fn delay(&mut self, interval_us: u64) {
        Timer::after(Duration::from_micros(interval_us)).await;
    }
}

#[derive(PartialEq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

#[derive(PartialEq)]
pub enum BusWidth {
    FourBits,
    EightBits,
}
