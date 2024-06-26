//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

pub extern crate rp2040_hal as hal;

#[cfg(feature = "rt")]
extern crate cortex_m_rt;

#[cfg(feature = "rt")]
pub use hal::entry;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

pub use hal::pac;

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

use display_interface_spi::SPIInterfaceNoCS;
use fugit::*;
use ws2812_pio::Ws2812Direct;
use rp2040_hal::pio::PIOExt;
use embedded_hal::digital::{OutputPin, InputPin};
use core::convert::Infallible;
use cortex_m::delay::Delay;

use ssd1306::{prelude::*, Ssd1306};

use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionSio, FunctionSpi, Pin, PullDown, SioInput, SioOutput},
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

use critical_section;

#[derive(Copy, Clone)]
pub enum Keys {
    TRACK,
    STEP,
    PLAY,
    REC,
    ALT,
    PATT,
    SONG,
    MENU,
    UP,
    DOWN,
    RIGHT,
    LEFT,
    A,
    B,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    K10,
    K11,
    K12,
    K13,
    K14,
    K15,
    K16,
    }

impl Keys {
    pub const LIST: [Self; 30] = [Keys::TRACK,
                                  Keys::STEP,
                                  Keys::PLAY,
                                  Keys::REC,
                                  Keys::ALT,
                                  Keys::PATT,
                                  Keys::SONG,
                                  Keys::MENU,
                                  Keys::UP,
                                  Keys::DOWN,
                                  Keys::RIGHT,
                                  Keys::LEFT,
                                  Keys::A,
                                  Keys::B,
                                  Keys::K1,
                                  Keys::K2,
                                  Keys::K3,
                                  Keys::K4,
                                  Keys::K5,
                                  Keys::K6,
                                  Keys::K7,
                                  Keys::K8,
                                  Keys::K9,
                                  Keys::K10,
                                  Keys::K11,
                                  Keys::K12,
                                  Keys::K13,
                                  Keys::K14,
                                  Keys::K15,
                                  Keys::K16,
                                ];

    pub fn mask(&self) -> u32 {
        match *self {
            Keys::TRACK => 0x10000000,
            Keys::STEP  => 0x08000000,
            Keys::PLAY  => 0x02000000,
            Keys::REC   => 0x04000000,
            Keys::ALT   => 0x00040000,
            Keys::PATT  => 0x00001000,
            Keys::SONG  => 0x00000040,
            Keys::MENU  => 0x00000020,
            Keys::UP    => 0x00020000,
            Keys::DOWN  => 0x00800000,
            Keys::RIGHT => 0x00000800,
            Keys::LEFT  => 0x20000000,
            Keys::A     => 0x01000000,
            Keys::B     => 0x00000001,
            Keys::K1    => 0x00400000,
            Keys::K2    => 0x00010000,
            Keys::K3    => 0x00000400,
            Keys::K4    => 0x00000010,
            Keys::K5    => 0x00000002,
            Keys::K6    => 0x00000080,
            Keys::K7    => 0x00002000,
            Keys::K8    => 0x00080000,
            Keys::K9    => 0x00200000,
            Keys::K10   => 0x00008000,
            Keys::K11   => 0x00000200,
            Keys::K12   => 0x00000008,
            Keys::K13   => 0x00000004,
            Keys::K14   => 0x00000100,
            Keys::K15   => 0x00004000,
            Keys::K16   => 0x00100000,
        }
    }

    pub fn led_index(&self) -> usize {
        match *self {
            Keys::TRACK => 4,
            Keys::STEP  => 14,
            Keys::PLAY  => 13,
            Keys::REC   => 23,
            Keys::ALT   => 3,
            Keys::PATT  => 2,
            Keys::SONG  => 1,
            Keys::MENU  => 0,
            Keys::UP    => 0,
            Keys::DOWN  => 0,
            Keys::RIGHT => 0,
            Keys::LEFT  => 0,
            Keys::A     => 0,
            Keys::B     => 0,
            Keys::K1    => 5,
            Keys::K2    => 6,
            Keys::K3    => 7,
            Keys::K4    => 8,
            Keys::K5    => 9,
            Keys::K6    => 10,
            Keys::K7    => 11,
            Keys::K8    => 12,
            Keys::K9    => 15,
            Keys::K10   => 16,
            Keys::K11   => 17,
            Keys::K12   => 18,
            Keys::K13   => 19,
            Keys::K14   => 20,
            Keys::K15   => 21,
            Keys::K16   => 22,
        }
    }
}
pub struct KeyboardMatrix {
    col1 : Pin<rp2040_hal::gpio::bank0::Gpio18, FunctionSio<SioOutput>, PullDown>,
    col2 : Pin<rp2040_hal::gpio::bank0::Gpio19, FunctionSio<SioOutput>, PullDown>,
    col3 : Pin<rp2040_hal::gpio::bank0::Gpio26, FunctionSio<SioOutput>, PullDown>,
    col4 : Pin<rp2040_hal::gpio::bank0::Gpio23, FunctionSio<SioOutput>, PullDown>,
    col5 : Pin<rp2040_hal::gpio::bank0::Gpio29, FunctionSio<SioOutput>, PullDown>,

    row1 : Pin<rp2040_hal::gpio::bank0::Gpio20, FunctionSio<SioInput>, PullDown>,
    row2 : Pin<rp2040_hal::gpio::bank0::Gpio21, FunctionSio<SioInput>, PullDown>,
    row3 : Pin<rp2040_hal::gpio::bank0::Gpio22, FunctionSio<SioInput>, PullDown>,
    row4 : Pin<rp2040_hal::gpio::bank0::Gpio24, FunctionSio<SioInput>, PullDown>,
    row5 : Pin<rp2040_hal::gpio::bank0::Gpio25, FunctionSio<SioInput>, PullDown>,
    row6 : Pin<rp2040_hal::gpio::bank0::Gpio27, FunctionSio<SioInput>, PullDown>,

    state : u32,
    prev_state : u32,
}

impl KeyboardMatrix {

    pub fn pressed(&self, k : Keys) -> bool
    {
        return (self.state & k.mask()) != 0;
    }

    pub fn falling(&self, k : Keys) -> bool
    {
        let all_falling = self.state & !self.prev_state;
        return (all_falling & k.mask()) != 0;
    }

    pub fn raising(&self, k : Keys) -> bool
    {
        let all_raising = !self.state & self.prev_state;
        return (all_raising & k.mask()) != 0;
    }

    pub fn scan(&mut self, delay : &mut Delay)
    {
        let mut cols : [&mut dyn OutputPin<Error = Infallible>; 5] = [&mut self.col1,
                                                                      &mut self.col2,
                                                                      &mut self.col3,
                                                                      &mut self.col4,
                                                                      &mut self.col5];
        let mut rows : [&mut dyn InputPin<Error = Infallible>; 6] = [&mut self.row1,
                                                                     &mut self.row2,
                                                                     &mut self.row3,
                                                                     &mut self.row4,
                                                                     &mut self.row5,
                                                                     &mut self.row6];
        let mut new_state : u32 = 0;

        for col in cols.iter_mut() {
            col.set_low().unwrap();
        }

        for col in cols.iter_mut() {
            col.set_high().unwrap();

            delay.delay_ms(1);

            for row in rows.iter_mut() {
                new_state <<= 1;
                if row.is_high().unwrap() {
                    new_state |= 1;
                }
            }
            col.set_low().unwrap();
        }

        self.prev_state = self.state;
        self.state = new_state;
    }

}

pub struct Peripherals {
    pub keyboard : KeyboardMatrix,
    pub display : Ssd1306<SPIInterfaceNoCS<rp2040_hal::spi::Spi<rp2040_hal::spi::Enabled, crate::pac::SPI1,
         (Pin<rp2040_hal::gpio::bank0::Gpio11, FunctionSpi, PullDown>, 
            Pin<rp2040_hal::gpio::bank0::Gpio10, FunctionSpi, PullDown>)>, 
            Pin<rp2040_hal::gpio::bank0::Gpio12, FunctionSio<SioOutput>, PullDown>>, 
            DisplaySize128x64,
            ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>>,
    pub leds : Ws2812Direct<crate::pac::PIO0,
                            rp2040_hal::pio::SM0,
                            Pin<rp2040_hal::gpio::bank0::Gpio5, rp2040_hal::gpio::FunctionPio0, PullDown>>,
    pub delay : Delay,
}

static mut DEVICE_PERIPHERALS: bool = false;

impl Peripherals {
    #[cfg(feature = "critical-section-impl")]
    #[inline]
    pub fn take() -> Option<Self> {
        critical_section::with(|_| {
            if unsafe { DEVICE_PERIPHERALS } {
                return None;
            }
            Some(unsafe { Peripherals::steal() })
        })
    }

    pub unsafe fn steal() -> Self {
        DEVICE_PERIPHERALS = true;
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);
    
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();
    
        let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    
        let pins = rp2040_hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
    
        // Keyboard
        let keys = KeyboardMatrix {        
            col1 : pins.gpio18.into_push_pull_output(),
            col2 : pins.gpio19.into_push_pull_output(),
            col3 : pins.gpio26.into_push_pull_output(),
            col4 : pins.gpio23.into_push_pull_output(),
            col5 : pins.gpio29.into_push_pull_output(),
            row1 : pins.gpio20.into_pull_down_input(),
            row2 : pins.gpio21.into_pull_down_input(),
            row3 : pins.gpio22.into_pull_down_input(),
            row4 : pins.gpio24.into_pull_down_input(),
            row5 : pins.gpio25.into_pull_down_input(),
            row6 : pins.gpio27.into_pull_down_input(),
            state : 0,
            prev_state : 0,
        };
   
        // These are implicitly used by the spi driver if they are in the correct mode
        let spi_sclk = pins.gpio10.into_function::<FunctionSpi>(); // scl
        let spi_mosi = pins.gpio11.into_function::<FunctionSpi>(); // sda
        // let _spi_miso = pins.gpio4.into_mode::<gpio::FunctionSpi>();
        let spi_dc = pins.gpio12.into_push_pull_output();
        let mut reset = pins.gpio13.into_push_pull_output();
    
        // Create an SPI driver instance for the SPI0 device
        let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI1, (spi_mosi, spi_sclk));
    
        // Exchange the uninitialised SPI driver for an initialised one
        let spi = spi.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            1_u32.MHz(),
            embedded_hal::spi::MODE_0,
        );
        let spi_interface = SPIInterfaceNoCS::new(spi, spi_dc);
        let mut display = Ssd1306::new(spi_interface, DisplaySize128x64, DisplayRotation::Rotate0)
                       .into_buffered_graphics_mode();
    
        display.reset(&mut reset, &mut delay).unwrap();
        display.init().unwrap();

        // LEDS
        let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
        let ws = Ws2812Direct::new(
            pins.gpio5.into_function(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
        );
    
        Peripherals {
            keyboard: keys,
            display: display,
            leds : ws,
            delay: delay,
        }
    }
}
