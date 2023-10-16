use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use fugit::Rate;
use stm32f4xx_hal::prelude::*;

pub fn tone_string_to_hz(tone: &str) -> Option<Rate<u32, 1, 1>> {
    let tones = [
        ("e0", 165.Hz()),
        ("f0", 175.Hz()),
        ("f0+", 185.Hz()),
        ("g0", 196.Hz()),
        ("g0+", 208.Hz()),
        ("a0", 220.Hz()),
        ("a0+", 233.Hz()),
        ("b0", 245.Hz()),
        ("c", 261.Hz()),
        ("c+", 277.Hz()),
        ("d", 294.Hz()),
        ("d+", 311.Hz()),
        ("e", 329.Hz()),
        ("f", 349.Hz()),
        ("f+", 370.Hz()),
        ("g", 392.Hz()),
        ("g+", 415.Hz()),
        ("a", 440.Hz()),
        ("a+", 466.Hz()),
        ("b", 493.Hz()),
        ("c2", 523.Hz()),
        ("d2", 594.Hz()),
    ];

    tones.iter().find(|(s, _)| s == &tone).map(|(_, hz)| *hz)
}

pub fn get_tune() -> Vec<(Option<Rate<u32, 1, 1>>, u32)> {
    let mario = [
        ("e", 2),
        ("e", 2),
        (" ", 2),
        ("e", 2),
        (" ", 2),
        ("c", 2),
        ("e", 4),
        ("g", 4),
        (" ", 4),
        ("g0", 4),
        (" ", 4),
        // main part
        ("c", 4),
        (" ", 2),
        ("g0", 4),
        (" ", 2),
        ("e0", 4),
        (" ", 2),
        ("a0", 4),
        ("b0", 4),
        ("a0+", 2),
        ("a0", 4),
        ("g0", 3),
        ("e", 3),
        ("g", 3),
        ("a", 4),
        ("f", 2),
        ("g", 2),
        (" ", 2),
        ("e", 4),
        ("c", 2),
        ("d", 2),
        ("b0", 4),
        (" ", 10000),
    ];

    let tune = mario;
    tune.iter()
        .map(|(tone, duration)| (tone_string_to_hz(tone), *duration))
        .collect()
}
