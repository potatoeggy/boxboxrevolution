use alloc::{string::String, vec::Vec};
use core::ops::{Div, Mul};
use hd44780_driver::{
    bus::{DataBus, FourBitBus},
    Cursor, CursorBlink, Display, DisplayMode, HD44780,
};
use stm32f4xx_hal::{
    gpio::{Output, Pin},
    timer::SysDelay,
};

use crate::game::rhythm::RhythmGame;

pub type GenericDisplay = HD44780<
    FourBitBus<
        Pin<'C', 7, Output>,
        Pin<'B', 6, Output>,
        Pin<'A', 7, Output>,
        Pin<'A', 6, Output>,
        Pin<'A', 4, Output>,
        Pin<'A', 8, Output>,
    >,
>;

pub struct LCD {
    pub driver: GenericDisplay,
}

const DISPLAY_LENGTH: u32 = 16;
const NOTE_CHARACTER: char = '|';

impl LCD {
    pub fn new(mut display: GenericDisplay, delay: &mut SysDelay) -> Self {
        display
            .set_display_mode(
                DisplayMode {
                    display: Display::On,
                    cursor_visibility: Cursor::Invisible,
                    cursor_blink: CursorBlink::Off,
                },
                delay,
            )
            .expect("Failed to set display mode");
        LCD { driver: display }
    }

    pub fn write(&mut self, text: &str, delay: &mut SysDelay) {
        self.driver.write_str(text, delay).unwrap();
    }

    pub fn print_rhythm_game(
        &mut self,
        rhythm: &RhythmGame,
        game_tick_period: u32,
        delay: &mut SysDelay,
    ) {
        // we want the upcoming n notes that are on the screen
        let notes = rhythm.get_note_positions();

        let mut shown_notes = [' '; DISPLAY_LENGTH as usize];

        for (note, ticks_left) in notes.iter() {
            if ticks_left >= &DISPLAY_LENGTH {
                break;
            }
            if note.is_some() {
                shown_notes[*ticks_left as usize] = NOTE_CHARACTER;
            }
        }

        // for (_, ticks_left) in notes.iter() {
        //     let position_before_boom = u32::div(
        //         u32::mul(
        //             u32::mul(*ticks_left, rhythm.tick_period()),
        //             game_tick_period,
        //         )
        //         .into(),
        //         100,
        //     );

        //     if position_before_boom < DISPLAY_LENGTH.into() {
        //         shown_notes[position_before_boom as usize] = NOTE_CHARACTER;
        //     }
        // }
        let shown_string: String = shown_notes.iter().collect();
        self.driver.clear(delay).unwrap();
        // let positions = notes.iter().map(|(_, data)| *data).collect::<Vec<u32>>();
        // self.write(&*format!("{:?}", shown_string), delay);
        self.write(&shown_string, delay);
    }
}
