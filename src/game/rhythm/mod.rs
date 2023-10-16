use alloc::vec::{self, Vec};
use fugit::Rate;

const DISPLAY_LENGTH: u32 = 16;
const NOTE_SPEED_MULTIPLIER: u32 = 100; // period of the note moving across display
type Note = (Option<Rate<u32, 1, 1>>, u32);

#[derive(Clone)]
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
    song: Song,
    pub current_tick: usize,
    pub score: i32,
}

/**
 * Given notes in the form of a vector of tuples of (frequency, duration),
 * set the duration of each note to be the cumulative duration of all notes
 * up to that point
 */
fn build_note_psa(notes: Vec<Note>) -> Vec<Note> {
    let mut psa = vec![];
    let mut current_tick = 0;
    for (freq, duration) in notes.iter() {
        psa.push((*freq, current_tick));
        current_tick += duration;
    }
    psa
}

impl RhythmGame {
    pub fn new(song: &Song) -> Self {
        let mut new_song = song.clone();
        new_song.notes = build_note_psa(new_song.notes);
        RhythmGame {
            song: new_song,
            current_tick: 0,
            score: 0,
        }
    }

    pub fn poll(&mut self) {
        self.current_tick += 1;
    }

    pub fn get_note_positions(&self) -> Vec<Note> {
        let mut note_positions = vec![];
        for (freq, duration) in self.song.notes.iter() {
            let note_position = self.current_tick as i32 - *duration as i32;
            if note_position >= 0 {
                note_positions.push((*freq, note_position as u32));
            }
        }
        note_positions
    }

    pub fn tick_period(&self) -> u32 {
        self.song.tempo
    }
}
