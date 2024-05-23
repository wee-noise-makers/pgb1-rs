//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

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
    pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

use critical_section;

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
}

impl KeyboardMatrix {
    pub fn get_pressed_key(keys : &mut KeyboardMatrix, delay : &mut Delay) -> u32
    {
        let mut cols : [&mut dyn OutputPin<Error = Infallible>; 5] = [&mut keys.col1, 
                                                                      &mut keys.col2, 
                                                                      &mut keys.col3, 
                                                                      &mut keys.col4,
                                                                      &mut keys.col5];
        let mut rows : [&mut dyn InputPin<Error = Infallible>; 6] = [&mut keys.row1,
                                                                     &mut keys.row2,
                                                                     &mut keys.row3,
                                                                     &mut keys.row4,
                                                                     &mut keys.row5,
                                                                     &mut keys.row6];
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
        return new_state;
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
    
        // External high-speed crystal on the pico board is 12Mhz
        let external_xtal_freq_hz = 12_000_000u32;
        let clocks = init_clocks_and_plls(
            external_xtal_freq_hz,
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
        }
    }
}

pub const K_TRACK : u32 = 0x10000000;
pub const K_STEP  : u32 = 0x08000000;
pub const K_PLAY  : u32 = 0x02000000;
pub const K_REC   : u32 = 0x04000000;
pub const K_ALT   : u32 = 0x00040000;
pub const K_PATT  : u32 = 0x00001000;
pub const K_SONG  : u32 = 0x00000040;
pub const K_MENU  : u32 = 0x00000020;
pub const K_UP    : u32 = 0x00020000;
pub const K_DOWN  : u32 = 0x00800000;
pub const K_RIGHT : u32 = 0x00000800;
pub const K_LEFT  : u32 = 0x20000000;
pub const K_A     : u32 = 0x01000000;
pub const K_B     : u32 = 0x00000001;
pub const K_1     : u32 = 0x00400000;
pub const K_2     : u32 = 0x00010000;
pub const K_3     : u32 = 0x00000400;
pub const K_4     : u32 = 0x00000010;
pub const K_5     : u32 = 0x00000002;
pub const K_6     : u32 = 0x00000080;
pub const K_7     : u32 = 0x00002000;
pub const K_8     : u32 = 0x00080000;
pub const K_9     : u32 = 0x00200000;
pub const K_10    : u32 = 0x00008000;
pub const K_11    : u32 = 0x00000200;
pub const K_12    : u32 = 0x00000008;
pub const K_13    : u32 = 0x00000004;
pub const K_14    : u32 = 0x00000100;
pub const K_15    : u32 = 0x00004000;
pub const K_16    : u32 = 0x00100000;
