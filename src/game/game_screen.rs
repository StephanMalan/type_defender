use std::{
    error::Error,
    io::Stdout,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyCode};
use rand::{thread_rng, Rng};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{Asset, GameError, GameState, Word, FPS, FRAME_TIME};

pub(crate) fn show_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    game_state: &mut GameState,
) -> Result<bool, Box<dyn Error>> {
    let mut last_frame_time = Instant::now();
    let mut counter = 20;

    load_words(game_state)?;

    let mut text_input = Input::default();

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
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(main_pane[1]);

        if counter == 0 {
            spawn_new_word(game_state);
            game_state.wpm = 30.0 + game_state.score / 10.0;
            counter = ((60.0 / game_state.wpm) * FPS as f32) as usize
        }

        // Draw the words
        if !generate_display(game_state, size)? {
            return Ok(true);
        }

        if !text_input.value().is_empty()
            && check_if_typed(game_state, text_input.value().to_owned())
        {
            text_input.reset();
        }

        let poll_time = FRAME_TIME
            .checked_sub(Duration::from_millis(5))
            .unwrap_or_else(|| Duration::from_micros(0));
        if event::poll(poll_time)? {
            if let crossterm::event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        text_input.reset();
                        continue;
                    }
                    KeyCode::Char(' ') => continue,
                    KeyCode::Esc => return Ok(false),
                    _ => text_input.handle_event(&crossterm::event::Event::Key(key)),
                };
            }
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
            f.render_widget(paragraph, main_pane[0]);

            let scroll = text_input.visual_scroll((bottom_pane[0].width.max(3) - 3) as usize);
            let text_input_paragraph = Paragraph::new(text_input.value())
                .style(Style::default())
                .scroll((0, scroll as u16))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(text_input_paragraph, bottom_pane[0]);
            f.set_cursor(
                bottom_pane[0].x + ((text_input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                bottom_pane[0].y + 1,
            );

            let score_label = Paragraph::new(format!("{:.1}", game_state.score))
                .block(Block::default().borders(Borders::ALL).title("Score"));
            f.render_widget(score_label, bottom_pane[1]);

            let wpm_label = Paragraph::new(format!("{:.1}", game_state.wpm))
                .block(Block::default().borders(Borders::ALL).title("WPM"));
            f.render_widget(wpm_label, bottom_pane[2])
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

    let speed =
        ((game_state.wpm / FPS as f32 / 20.0) + thread_rng().gen_range(-0.02..0.02)).max(0.01);

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
            game_state.score += 500.0 * (1.0 - w.clone().progress()).powf(3.0) * w.speed;
            game_state.word_slots[w.y] = 0;
        });
    found
}
