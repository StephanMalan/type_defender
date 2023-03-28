use core::panic;
use rand::Rng;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Stdout};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self};
use std::time::{Duration, Instant};
use strum::{Display, IntoEnumIterator};
use strum_macros::EnumIter;
use termion::event::Event as TermEvent;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

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
    fn new(language: Language) -> Self {
        GameState {
            language: language,
            score: 0.0,
            word_pool: vec![],
            words: vec![],
            word_slots: [0; 40],
            display_rows: vec![],
        }
    }
}

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
            text: text,
            found: false,
            x: 0.0,
            y: y,
            speed: speed,
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

enum Event {
    Term(TermEvent),
    Tick,
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

fn main() {
    let result = std::panic::catch_unwind(|| {
        // panic!("test");
        let result = run_game();
        match result {
            Ok(_) => print!(""),
            Err(error) => println!("{}", error),
        }
    });
    match result {
        Ok(_) => println!("The code ran successfully"),
        Err(err) => println!("{:?}", err.downcast_ref::<&str>()),
    }
}

fn run_game() -> Result<(), String> {
    let stdout = io::stdout()
        .into_raw_mode()
        .expect("Failed to created raw terminal");
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let mut terminal =
        Terminal::new(TermionBackend::new(stdout)).expect("Failed to create Termion terminal");

    let events = {
        let events = io::stdin().events();
        let (tx, rx) = mpsc::channel();
        let keys_tx = tx.clone();
        thread::spawn(move || {
            for event in events.flatten() {
                keys_tx.send(Event::Term(event)).unwrap();
            }
        });
        thread::spawn(move || loop {
            tx.send(Event::Tick).unwrap();
            thread::sleep(Duration::from_millis(30));
        });
        rx
    };

    let mut game_state = GameState::new(Language::English);

    terminal.clear().expect("Failed to clear terminal");
    language_selection(&mut terminal, &mut game_state, &events);
    terminal.clear().expect("Failed to clear terminal");
    game_loop(&mut terminal, &mut game_state, &events)?;
    terminal.clear().expect("Failed to clear terminal");

    Ok(())
}

fn language_selection(
    terminal: &mut Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    game_state: &mut GameState,
    events: &Receiver<Event>,
) {
    // Create a list of options
    let languages: Vec<Language> = Language::iter().collect();
    let mut items = StatefulList::with_items(
        languages
            .iter()
            .map(|l| ListItem::new(l.to_string()))
            .collect(),
    );
    items.next();

    // Create the list widget and set its items
    let list = List::new(&*items.items)
        .block(
            tui::widgets::Block::default()
                .title("Please select a language")
                .borders(tui::widgets::Borders::ALL),
        )
        .highlight_symbol(">> ");

    // Set up the terminal and run the event loop
    terminal.clear().expect("Failed to clear terminal");
    loop {
        // Get the size of the terminal
        let size = terminal.size().expect("Failed to retrieve Terminal size");

        // Create a layout with one row and one column
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)].as_ref())
            .split(size);

        // Wait for a key press event
        match events.recv().expect("Failed to match event") {
            Event::Term(event) => match event.into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } => {
                    game_state.language = languages
                        .get(items.state.selected().unwrap())
                        .unwrap()
                        .clone();
                    println!("Selected Language: {:?}", items.state.selected());
                    break;
                }
                Input { key: Key::Up, .. } => items.previous(),
                Input { key: Key::Down, .. } => items.next(),
                _input => continue,
            },
            Event::Tick => {}
        };

        // Draw the list in the center of the layout
        terminal
            .draw(|f| {
                let inner_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(0)].as_ref())
                    .split(layout[0]);
                f.render_widget(list.clone(), inner_layout[0]);
                f.render_stateful_widget(list.clone(), inner_layout[0], &mut items.state)
            })
            .expect("Failed to render terminal");
    }
}

fn game_loop(
    terminal: &mut Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    game_state: &mut GameState,
    events: &Receiver<Event>,
) -> Result<(), String> {
    let fps = 30;
    let frame_time = Duration::from_micros(1_000_000 / fps as u64);
    let mut last_frame_time = Instant::now();
    let mut counter = 20;

    load_words(game_state);

    let mut text_input = TextArea::default();
    text_input.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Attack console"),
    );

    loop {
        let elapsed_time = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        counter -= 1;

        // Calculate the layout for the terminal
        let size = terminal.size().expect("Failed to retrieve Terminal size");
        let main_pane = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(42), Constraint::Length(1)].as_ref())
            .split(Rect {
                x: 0,
                y: 0,
                width: size.width,
                height: 47,
            });
        let bottom_pane = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_pane[1]);

        if size.height < 47 {
            panic!("Please resize console")
        }

        if counter <= 0 {
            spawn_new_word(game_state);
            let highest_wpm = 100.0;
            let lowest_wpm = 30.0;
            let wpm =
                highest_wpm - ((highest_wpm - lowest_wpm) * (100.0 / (100.0 + game_state.score)));
            counter = ((60.0 / wpm) * fps as f32) as usize
        }

        // Draw the words
        generate_display(game_state, size)?;

        if text_input.lines()[0].len() > 0 {
            if check_if_typed(game_state, text_input.lines()[0].to_owned()) {
                text_input.delete_line_by_head();
            }
        }

        match events.recv().expect("Failed to match event") {
            Event::Term(event) => match event.into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } => continue,
                input => {
                    text_input.input(input);
                }
            },
            Event::Tick => {}
        };

        // Draw the text
        terminal
            .draw(|f| {
                let block = Block::default()
                    .title("Type Defender")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White));

                let paragraph = Paragraph::new(game_state.display_rows.to_owned())
                    .block(block)
                    .style(Style::default().fg(Color::White))
                    .alignment(tui::layout::Alignment::Left)
                    .wrap(Wrap { trim: false });

                let score_label =
                    Paragraph::new(format!("{:.1} counter: {}", game_state.score, counter))
                        .block(Block::default().borders(Borders::ALL).title("Score"));

                f.render_widget(paragraph, main_pane[0]);
                // f.render_widget(bottom_pane, main_pane[1]);
                f.render_widget(text_input.widget(), bottom_pane[0]);
                f.render_widget(score_label, bottom_pane[1])
            })
            .expect("Failed to render terminal");

        // Sleep to maintain desired FPS
        let time_to_sleep = frame_time
            .checked_sub(elapsed_time)
            .unwrap_or_else(|| Duration::from_micros(0));
        thread::sleep(time_to_sleep);
    }
    Ok(())
}

fn load_words(game_state: &mut GameState) {
    let file_name = format!(
        "resources/{}_words.txt",
        game_state.language.to_string().to_lowercase()
    );
    let file = File::open(file_name).expect("Failed to read file");
    let reader = BufReader::new(file);
    let words: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Error reading line"))
        .filter(|l| !l.is_empty())
        .collect();
    game_state.word_pool = words;
}

fn spawn_new_word(game_state: &mut GameState) {
    if game_state.word_pool.len() == 0 {
        panic!("No more words left");
    }

    // Get random, open y value
    let indices: Vec<usize> = game_state
        .word_slots
        .iter()
        .enumerate()
        .filter(|(_, &val)| val == 0)
        .map(|(i, _)| i)
        .collect();
    if indices.len() == 0 {
        return;
    }
    let random_index = indices[rand::thread_rng().gen_range(0..indices.len())];
    game_state.word_slots[random_index] = 1;

    // Get random word from
    let index = rand::thread_rng().gen_range(0..game_state.word_pool.len());
    let new_word = game_state.word_pool.remove(index);

    let mut speed = 0.3 - ((0.25) * (200.0 / (200.0 + game_state.score)));
    speed += rand::thread_rng().gen_range(-0.02..0.02);
    game_state
        .words
        .push(Word::new(new_word.to_lowercase(), random_index, speed));
}

fn generate_display(game_state: &mut GameState, size: Rect) -> Result<(), String> {
    game_state.display_rows = vec![];
    for (i, val) in game_state.word_slots.iter().enumerate() {
        if val == &0 {
            game_state
                .display_rows
                .push(Spans::from(vec![Span::raw("")]));
            continue;
        }
        let word = game_state
            .words
            .iter_mut()
            .find(|w| w.y == i && !w.found)
            .expect("Invalid game state");
        let text = word.text.to_string();
        let progress = &word.clone().progress();
        if progress >= &1.0 {
            return Err(format!("Failed to stop {:?}", word));
        }

        let color = Color::Rgb(
            (progress * 255.0) as u8,
            (255.0 - (progress * 255.0)) as u8,
            0,
        );
        game_state.display_rows.push(Spans::from(vec![
            Span::raw(
                " ".repeat((progress * (size.width as f32 - text.len() as f32 - 3.0)) as usize),
            ),
            Span::styled(text.to_owned(), Style::default().fg(color)),
        ]));
        word.increment();
    }
    Ok(())
}

fn check_if_typed(game_state: &mut GameState, text: String) -> bool {
    let mut found = false;
    game_state
        .words
        .iter_mut()
        .filter(|w| !w.found && w.text.to_uppercase() == text.to_uppercase())
        .for_each(|w| {
            found = true;
            w.found = true;
            game_state.score += 100.0 * w.clone().progress() * w.speed;
            game_state.word_slots[w.y] = 0;
        });
    return found;
}
