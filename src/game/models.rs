use alloc::boxed::Box;

pub trait GameScreenStage {}
pub struct ScreenSelect;
pub struct ScreenPlaying;
pub struct ScreenPaused;
pub struct ScreenTally;
impl GameScreenStage for ScreenSelect {}
impl GameScreenStage for ScreenPlaying {}
impl GameScreenStage for ScreenPaused {}
impl GameScreenStage for ScreenTally {}

pub struct GameState {}

pub struct Game<S: GameScreenStage> {
    _stage: S,
    pub state: Box<GameState>,
}

impl Game<ScreenSelect> {
    pub fn new() -> Self {
        Game {
            _stage: ScreenSelect,
            state: Box::new(GameState {}),
        }
    }

    pub fn to_playing(self) -> Game<ScreenPlaying> {
        Game {
            _stage: ScreenPlaying,
            state: self.state,
        }
    }
}

impl Game<ScreenPlaying> {
    pub fn to_paused(self) -> Game<ScreenPaused> {
        Game {
            _stage: ScreenPaused,
            state: self.state,
        }
    }
}
