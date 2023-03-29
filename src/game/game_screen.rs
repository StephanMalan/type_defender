use std::{
    error::Error,
    io::Stdout,
    thread,
    time::{Duration, Instant},
};

use crossterm::event;
use rand::Rng;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use tui_textarea::{Input, Key, TextArea};

use crate::{Asset, GameError, GameState, Word, FPS, FRAME_TIME};

pub(crate) fn show_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    game_state: &mut GameState,
) -> Result<bool, Box<dyn Error>> {
    let mut last_frame_time = Instant::now();
    let mut counter = 20;

    load_words(game_state)?;

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
        let size = terminal.size()?;
        if size.height < 47 {
            return Err(Box::new(GameError(
                "Console should be at least 47 lines tall",
            )));
        }

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

        if counter == 0 {
            spawn_new_word(game_state);
            let highest_wpm = 80.0;
            let lowest_wpm = 30.0;
            let wpm =
                highest_wpm - ((highest_wpm - lowest_wpm) * (500.0 / (500.0 + game_state.score)));
            counter = ((60.0 / wpm) * FPS as f32) as usize
        }

        // Draw the words
        if !generate_display(game_state, size)? {
            return Ok(true);
        }

        if !text_input.lines()[0].is_empty() {
            if check_if_typed(game_state, text_input.lines()[0].to_owned()) {
                text_input.delete_line_by_head();
            }
        }

        let poll_time = FRAME_TIME
            .checked_sub(Duration::from_millis(5))
            .unwrap_or_else(|| Duration::from_micros(0));
        if event::poll(poll_time)? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => return Ok(false),
                Input {
                    key: Key::Enter, ..
                } => continue,
                input => text_input.input(input),
            };
        }

        // Draw the text
        terminal.draw(|f| {
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
        })?;

        // Sleep to maintain desired FPS
        let time_to_sleep = FRAME_TIME
            .checked_sub(elapsed_time)
            .unwrap_or_else(|| Duration::from_micros(0));
        thread::sleep(time_to_sleep);
    }
}

fn load_words(game_state: &mut GameState) -> Result<(), Box<dyn Error>> {
    let asset = Asset::get(
        format!(
            "{}_words.txt",
            game_state.language.to_string().to_lowercase()
        )
        .as_str(),
    )
    .unwrap();
    let data = asset.data.as_ref();
    let words: Vec<String> = std::str::from_utf8(data)?
        .lines()
        .map(|line| line.to_string())
        .filter(|l| !l.is_empty())
        .collect();
    game_state.word_pool = words;

    Ok(())
}

fn spawn_new_word(game_state: &mut GameState) {
    if game_state.word_pool.is_empty() {
        panic!("No more words left.");
    }

    // Get random, open y value
    let indices: Vec<usize> = game_state
        .word_slots
        .iter()
        .enumerate()
        .filter(|(_, &val)| val == 0)
        .map(|(i, _)| i)
        .collect();
    if indices.is_empty() {
        return;
    }
    let random_index = indices[rand::thread_rng().gen_range(0..indices.len())];
    game_state.word_slots[random_index] = 1;

    // Get random word from
    let index = rand::thread_rng().gen_range(0..game_state.word_pool.len());
    let new_word = game_state.word_pool.remove(index);

    let mut speed = 0.2 - ((0.15) * (200.0 / (200.0 + game_state.score)));
    speed += rand::thread_rng().gen_range(-0.02..0.02);
    game_state
        .words
        .push(Word::new(new_word.to_lowercase(), random_index, speed));
}

fn generate_display(game_state: &mut GameState, size: Rect) -> Result<bool, String> {
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
            .unwrap();
        let text = word.text.to_string();
        let progress = &word.clone().progress();
        if progress >= &1.0 {
            return Ok(false);
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
    Ok(true)
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
            game_state.score += 100.0 * (1.0 - w.clone().progress()) * w.speed;
            game_state.word_slots[w.y] = 0;
        });
    found
}
