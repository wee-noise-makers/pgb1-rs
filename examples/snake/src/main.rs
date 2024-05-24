#![no_std]
#![no_main]

use pgb1;
use pgb1::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use embedded_graphics::{
    pixelcolor::*,
    prelude::DrawTarget,
};

mod snake;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut periph = pgb1::Peripherals::take().unwrap();
    let small_rng = SmallRng::seed_from_u64(42);

    let mut game = 
    snake::SnakeGame::<100, embedded_graphics::pixelcolor::BinaryColor, SmallRng>::new(
        128,
        64,
        3,
        3,
        small_rng,
        BinaryColor::On,
        BinaryColor::On,
        255,
    );

    loop {
        periph.keyboard.scan(&mut periph.delay);
        let mut direction = snake::Direction::None;
        
        if periph.keyboard.pressed(pgb1::Keys::UP) {
            direction = snake::Direction::Up;
        } else if periph.keyboard.pressed(pgb1::Keys::DOWN) {
            direction = snake::Direction::Down;
        } else if periph.keyboard.pressed(pgb1::Keys::LEFT) {
            direction = snake::Direction::Left;
        } else if periph.keyboard.pressed(pgb1::Keys::RIGHT) {
            direction = snake::Direction::Right;
        }

        game.set_direction(direction);
        periph.display.clear(BinaryColor::Off).unwrap();
        game.draw(&mut periph.display);
        periph.display.flush().unwrap();
        
        periph.delay.delay_ms(50);
    }
}
