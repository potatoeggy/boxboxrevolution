#![no_main]
#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
use bbr::{
    self as _,
    game::rhythm::{RhythmGame, Song},
    lcd_helper::LCD,
    songs::get_tune,
};
use cortex_m::peripheral::SYST;
use embedded_hal::digital::v2::OutputPin;
// use bbr::ultrasonic;
use core::ops::{Div, Mul};
// global logger + panicking-behavior + memory layout
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use keypad2::Keypad;
use stm32f4xx_hal::{
    gpio::{Edge, Input, Output, Pin, Pull, PushPull},
    pac::{self, TIM1, TIM2},
    prelude::*,
    timer::{Channel, Delay, SysDelay, Timer},
};

const ULTRASONIC_VALID_THRESHOLD: f64 = 30.0;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

fn init_allocator() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}

// Connections:
// GND: GND
// VDD: 5V
// V0:  10k poti between 5V and GND
// RS:  D9 / PC7
// RW:  GND
// E:   D10 / PB6
// D4:  D11 / PA7
// D5:  D12 / PA6
// D6:  A2 / PA4
// D7:  D7 / PA8
// BLA:   5V
// BLK:   GND

// Keypad connections:
// from left to right:
// D0 / PA3 (C2)
// D1 / PA2 (R1)
// D2 / PA10 (C1)
// D3 / PB3 (R4)
// D4 / PB5 (C3)
// D5 / PB4 (R3)
// D6 / PB10 (R2)

// Buzzer connection:
// D8 / PA9

// Ultrasonic sensor:
// trigger: D14 / PB8
// echo: D15 / PB9

struct UltrasonicSensor<Trigger>
where
    Trigger: OutputPin,
{
    trigger: Trigger,
    echo: Pin<'B', 8, Input>,
}

fn read_ultrasonic<T: OutputPin>(
    sensor: &mut UltrasonicSensor<T>,
    delay: &mut SysDelay,
) -> Option<f64> {
    sensor.trigger.set_high();
    delay.delay_us(10u16);
    sensor.trigger.set_low();

    let mut counter = 0;

    // defmt::info!("Distance: {}", counter);
    while !sensor.echo.is_high() {
        counter += 1;
        delay.delay_us(1u32);
        if counter > 100000 {
            // it means that we are not getting a response
            return Some(-1.0);
        }
    }

    let mut counter = 0;
    while !sensor.echo.is_low() {
        delay.delay_us(1u16);
        counter += 1;
        delay.delay_us(1u32);
        if counter > 100000 {
            return Some(-2.0);
        }
    }
    let result: f64 = f64::mul(
        counter.into(),
        f64::div(f64::div(f64::mul(1000.0, 343.0), 2.0), 1000000.0),
    );
    Some(result)
}

#[cortex_m_rt::entry]
fn main() -> ! {
    init_allocator();
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
    let mut delay = cp.SYST.delay(&clocks);
    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    // let rows = (
    //     gpioa.pa2.into_pull_up_input(),
    //     gpiob.pb10.into_pull_up_input(),
    //     gpiob.pb4.into_pull_up_input(),
    //     gpiob.pb3.into_pull_up_input(),
    // );
    // let cols = (
    //     gpioa.pa10.into_open_drain_output(),
    //     gpioa.pa3.into_open_drain_output(),
    //     gpiob.pb5.into_open_drain_output(),
    // );

    let mut red_led = gpioa.pa0.into_push_pull_output();
    let mut green_led = gpioa.pa1.into_push_pull_output();

    let rows = (
        gpioa.pa2.into_pull_up_input(),
        gpiob.pb10.into_pull_up_input(),
        gpiob.pb4.into_pull_up_input(),
        gpiob.pb3.into_pull_up_input(),
    );
    let cols = (
        gpioa.pa10.into_open_drain_output(),
        gpioa.pa3.into_open_drain_output(),
        gpiob.pb5.into_open_drain_output(),
    );

    let rs = gpioc.pc7.into_push_pull_output();
    let en = gpiob.pb6.into_push_pull_output();
    let d4 = gpioa.pa7.into_push_pull_output();
    let d5 = gpioa.pa6.into_push_pull_output();
    let d6 = gpioa.pa4.into_push_pull_output();
    let d7 = gpioa.pa8.into_push_pull_output();

    // pa13 nukes the board
    // pa15 dies halfway through

    let mut keypad = Keypad::new(rows, cols);
    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();
    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut delay,
    )
    .unwrap();
    lcd.write_str("Booting...", &mut delay).unwrap();
    lcd.set_cursor_pos(40, &mut delay).unwrap();
    lcd.write_str("Num2", &mut delay).unwrap();

    let mut lcd_driver = LCD::new(lcd, &mut delay);

    let mut led = gpioa.pa5.into_push_pull_output();
    let mut echo = gpioc.pc6.into_pull_down_input();

    let mut ultrasonic = UltrasonicSensor {
        trigger: gpiob.pb9.into_push_pull_output(),
        echo: gpiob.pb8.into_floating_input(),
    };

    defmt::info!("Connected to target!");

    let buzzer = gpioa.pa9.into_alternate();
    let mut buzz_pwm = dp.TIM1.pwm_hz(buzzer, 2000.Hz(), &clocks);
    let max_duty = buzz_pwm.get_max_duty();
    red_led.set_high();
    buzz_pwm.set_duty(Channel::C2, max_duty / 2);

    let tempo = 2;
    let song = Song::new(tempo, get_tune());
    let mut game = RhythmGame::new(song);

    let TICK_RATE = 120;
    let tick_pause: u32 = 1_000_000 / TICK_RATE - 200;
    let mut current_tick: u32 = 0;

    let mut sleep_until = 0;
    let mut points_available = false;
    loop {
        if current_tick % game.tick_period() == 2 {
            green_led.set_low();
            red_led.set_low();
        }
        if sleep_until != 0 && current_tick >= sleep_until {
            buzz_pwm.disable(Channel::C2);
            sleep_until = 0;
            points_available = false;
        }

        if current_tick % 3 == 0 && points_available {
            let distance = read_ultrasonic(&mut ultrasonic, &mut delay);
            if let Some(distance) = distance {
                if distance < ULTRASONIC_VALID_THRESHOLD {
                    green_led.set_high();
                    red_led.set_low();
                    game.score += 1;
                    points_available = false;
                } else {
                    green_led.set_low();
                    red_led.set_high();
                }
            } else {
                red_led.set_high();
            }
        }

        if current_tick % game.tick_period() == 0 {
            game.poll();
            if game.current_tick >= game.max_ticks as usize {
                // game over
                buzz_pwm.disable(Channel::C2);
                lcd_driver.driver.clear(&mut delay);
                lcd_driver.write("Game over!", &mut delay);
                lcd_driver.driver.set_cursor_pos(40, &mut delay);
                lcd_driver.write(
                    &*format!("Score: {}/{}", game.score, game.song.notes.len()),
                    &mut delay,
                );
                panic!("Game over!");
            }

            lcd_driver.print_rhythm_game(&game, tick_pause, &mut delay);

            if let Some((freq, duration)) = game.get_current_note() {
                if let Some(freq) = freq {
                    buzz_pwm.set_period(freq);
                    buzz_pwm.enable(Channel::C2);
                    sleep_until = current_tick + duration * tempo;
                    points_available = true;
                } else {
                    buzz_pwm.disable(Channel::C2);
                    // delay.delay_ms(duration * TICK_RATE / tempo);
                }

                // buzz_pwm.disable(Channel::C2);
            }

            // lcd.clear(&mut delay);
            // lcd.write_str(&*format!("{:?}", distance), &mut delay)
            //     .unwrap();
        }
        // for (freq, duration) in song.iter() {
        //     lcd.clear(&mut delay).unwrap();
        //     if let Some(freq) = freq {
        //         lcd.write_str(&*format!("{:?}", freq.raw()), &mut delay)
        //             .unwrap();
        //         buzz_pwm.set_period(*freq);
        //         green_led.set_high();
        //         buzz_pwm.enable(Channel::C2);
        //         delay.delay_ms(duration * tempo);
        //     } else {
        //         red_led.set_high();
        //         buzz_pwm.disable(Channel::C2);
        //         delay.delay_ms(duration * tempo);
        //     }
        //     green_led.set_low();
        //     red_led.set_low();

        //     buzz_pwm.disable(Channel::C2);
        //     // 4.2 Keep the output off for half a beat between notes
        //     delay.delay_ms(tempo);
        // }
        current_tick += 1;
        delay.delay_us(tick_pause);
    }

    // loop {
    //     let key = keypad.read_char(&mut delay);
    //     let distance = read_ultrasonic(&mut ultrasonic, &mut delay);
    //     lcd.clear(&mut delay);
    //     lcd.write_str(&*format!("{:?}", distance), &mut delay)
    //         .unwrap();
    //     // if let Some(val) = distance {
    //     //     // defmt::info!("New value: {}", val);
    //     //     lcd.write_str(&*val.to_string(), &mut delay).unwrap();
    //     //     delay.delay_ms(1000u16);
    //     // }
    //     // lcd.clear(&mut delay);
    //     // lcd.write_str(&*format!("{:?}", distance), &mut delay).unwrap();
    //     if key != ' ' {
    //         // lcd.write_str(key.encode_utf8(&mut buffer), &mut delay).unwrap();

    //         if is_green_on && counter == 9 {
    //             green_led.set_low();
    //             red_led.set_high();
    //             is_green_on = false;
    //         } else if counter == 9 {
    //             green_led.set_high();
    //             red_led.set_low();
    //             is_green_on = true;
    //         }
    //     }
    //     delay.delay_ms(10u16);
    //     counter += 1;
    //     counter = counter % 10;
    // }
}
