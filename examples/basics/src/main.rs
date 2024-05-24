#![no_std]
#![no_main]

use pgb1::*;
use pgb1::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut periph = pgb1::Peripherals::take().unwrap();
    let mut small_rng = SmallRng::seed_from_u64(42);
    let mut brightness : u8 = 20;

    let yoffset = 20;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    // screen outline
    Rectangle::new(Point::new(0, 0), Size::new(127, 63))
        .into_styled(style)
        .draw(&mut periph.display)
        .unwrap();

    // triangle
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(style)
    .draw(&mut periph.display)
    .unwrap();

    // square
    Rectangle::new(Point::new(52, yoffset), Size::new_equal(16))
        .into_styled(style)
        .draw(&mut periph.display)
        .unwrap();

    // circle
    Circle::new(Point::new(88, yoffset), 16)
        .into_styled(style)
        .draw(&mut periph.display)
        .unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Create a text at position and draw it using the previously defined style
    Text::new("Hello PGB-1!", Point::new(5, 10), style).draw(&mut periph.display).unwrap();

    // Update display
    periph.display.flush().unwrap();

    use smart_leds::{SmartLedsWrite, RGB8, hsv};
    let mut colors: [RGB8; 24];

    colors = [(0, 0, 0).into(); 24];

    loop {

        // Scan keyboard state
        periph.keyboard.scan(&mut periph.delay);

        //  Set a random color to the LEDs of falling keys
        for k in pgb1::Keys::LIST {
            match k {
                Keys::UP   => {
                        if brightness < 255 && periph.keyboard.pressed(k) {
                            brightness += 1;
                            info!("brightness: {}", brightness);
                        }
                    }
                Keys::DOWN => {
                        if brightness > 0 && periph.keyboard.pressed(k) {
                            brightness -= 1;
                            info!("brightness: {}", brightness);   
                        }
                    }
                
                Keys::LEFT | Keys::RIGHT | Keys::A | Keys::B => {},

                _ => {
                    if periph.keyboard.falling(k) {
                        let hsv = hsv::Hsv{hue: small_rng.gen::<u8>(),
                                                sat: 255_u8,
                                                val: 255_u8};
                        colors[k.led_index()] = hsv::hsv2rgb(hsv).into();
                    }
                }
            }
        }

        // Update LEDs
        periph.leds.write(smart_leds::brightness(colors.iter().copied(), brightness)).unwrap();
        
        periph.delay.delay_ms(30);
    };
}
