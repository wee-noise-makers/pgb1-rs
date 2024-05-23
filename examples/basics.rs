#![no_std]
#![no_main]

use rp2040_hal::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use pgb1;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut periph = pgb1::Peripherals::take().unwrap();

    // let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    use smart_leds::{RGB8, SmartLedsWrite};
    let mut colors: [RGB8; 24];

    loop {
        colors = [(0, 0, 100).into(); 24];

        //let pressed = periph.keyboard.get_pressed_key(&mut delay);
        let pressed = 0;
        info!("Pressed {:#032b}", pressed);

        if (pressed & pgb1::K_MENU) != 0 {
            colors[0] = (100, 0, 0).into();
        }
        if (pressed & pgb1::K_SONG) != 0 {
            colors[1] = (100, 0, 0).into();
        }
        if (pressed & pgb1::K_PATT) != 0 {
            colors[2] = (100, 0, 0).into();
        }
        for _n in 1..25 {
            periph.leds.write(colors.iter().copied()).unwrap();
        }
        //  delay.delay_ms(100);
    };
}
