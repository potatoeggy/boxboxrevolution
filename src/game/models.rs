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
    stage: S,
    pub state: Box<GameState>,
}

impl Game<ScreenSelect> {
    fn new() -> Self {
        Game {
            stage: ScreenSelect,
            state: Box::new(GameState {}),
        }
    }

    fn to_playing(self) -> Game<ScreenPlaying> {
        Game {
            stage: ScreenPlaying,
            state: self.state,
        }
    }
}

impl Game<ScreenPlaying> {
    fn to_paused(self) -> Game<ScreenPaused> {
        Game {
            stage: ScreenPaused,
            state: self.state,
        }
    }
}
