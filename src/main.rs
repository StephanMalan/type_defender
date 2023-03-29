mod game;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use rust_embed::RustEmbed;
use std::error::Error;
use std::io::stdout;
use std::time::Duration;
use std::{fmt, io};
use strum::Display;
use strum_macros::EnumIter;
use tui::backend::CrosstermBackend;
use tui::text::Spans;
use tui::widgets::ListState;
use tui::Terminal;

const FPS: i32 = 30;
const FRAME_TIME: Duration = Duration::from_micros(1_000_000 / FPS as u64);

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Asset;

#[derive(Clone, Display, EnumIter)]
enum Language {
    Afrikaans,
    English,
    Korean,
}

struct GameState<'a> {
    language: Language,
    score: f32,
    word_pool: Vec<String>,
    words: Vec<Word>,
    word_slots: [i32; 40],
    display_rows: Vec<Spans<'a>>,
}

impl GameState<'_> {
    fn new() -> Self {
        GameState {
            language: Language::English,
            score: 0.0,
            word_pool: vec![],
            words: vec![],
            word_slots: [0; 40],
            display_rows: vec![],
        }
    }
}

#[derive(Debug)]
struct GameError<'a>(&'a str);

impl fmt::Display for GameError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GameError<'_> {}

#[derive(Debug, Clone)]
struct Word {
    text: String,
    found: bool,
    x: f32,
    y: usize,
    speed: f32,
}

impl Word {
    fn new(text: String, y: usize, speed: f32) -> Self {
        Word {
            text,
            found: false,
            x: 0.0,
            y,
            speed,
        }
    }

    fn increment(&mut self) {
        self.x += self.speed;
    }

    fn progress(self) -> f32 {
        self.x / 100.0
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let result = std::panic::catch_unwind(|| {
        let result = run_game();
        match result {
            Ok(_) => (),
            Err(error) => println!("Error: {}", error),
        }
    });
    match result {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err.downcast_ref::<&str>()),
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_game() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let mut game_state = GameState::new();
        terminal.clear()?;
        if !game::home_screen::show_view(&mut terminal, &mut game_state)? {
            return Ok(());
        }
        terminal.clear()?;
        if !game::game_screen::show_view(&mut terminal, &mut game_state)? {
            return Ok(());
        }
        terminal.clear()?;
        if !game::end_screen::show_view(&mut terminal, &mut game_state)? {
            return Ok(());
        }
    }
}
