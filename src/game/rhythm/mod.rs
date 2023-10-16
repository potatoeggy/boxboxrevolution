use alloc::vec::{self, Vec};
use fugit::Rate;

const DISPLAY_LENGTH: u32 = 16;
const MAX_NOTES_ON_SCREEN: usize = 3; // prevent OOM
const NOTE_SPEED_MULTIPLIER: u32 = 100; // period of the note moving across display
type Note = (Option<Rate<u32, 1, 1>>, u32);

pub struct Song {
    tempo: u32, // number of ticks per unit note
    pub notes: Vec<Note>,
}

impl Song {
    pub fn new(tempo: u32, notes: Vec<Note>) -> Self {
        Song { tempo, notes }
    }
}

pub struct RhythmGame {
    pub song: Song,
    pub current_tick: usize,
    pub score: i32,
    pub max_ticks: u32,
}

/**
 * Given notes in the form of a vector of tuples of (frequency, duration),
 * set the duration of each note to be the cumulative duration of all notes
 * up to that point
 */
fn build_note_psa(notes: &mut Vec<Note>) -> u32 {
    let mut current_tick = 0;
    for (_, duration) in notes.iter_mut() {
        current_tick += *duration;
        *duration = current_tick;
    }
    current_tick
}

impl RhythmGame {
    pub fn new(mut song: Song) -> Self {
        let max_ticks = build_note_psa(&mut song.notes);
        RhythmGame {
            song: song,
            current_tick: 0,
            score: 0,
            max_ticks: max_ticks,
        }
    }

    pub fn poll(&mut self) {
        self.current_tick += 1;
    }

    pub fn get_note_positions(&self) -> Vec<Note> {
        let mut note_positions = vec![];
        for (freq, duration) in self.song.notes.iter() {
            let note_position = *duration as i32 - self.current_tick as i32;
            if note_position >= 0 as i32 {
                note_positions.push((*freq, note_position as u32));
            }

            if note_positions.len() > MAX_NOTES_ON_SCREEN || note_position >= DISPLAY_LENGTH as i32
            {
                break;
            }
        }
        note_positions
    }

    pub fn tick_period(&self) -> u32 {
        self.song.tempo
    }

    pub fn get_current_note(&self) -> Option<Note> {
        let mut current_note = None;
        for (freq, duration) in self.song.notes.iter() {
            if *duration == self.current_tick as u32 {
                current_note = Some((*freq, *duration));
                break;
            }
        }
        current_note
    }
}
