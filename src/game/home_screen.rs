use std::{
    error::Error,
    io::Stdout,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use strum::IntoEnumIterator;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{List, ListItem},
    Terminal,
};

use crate::{GameError, GameState, Language, StatefulList, FRAME_TIME};

pub(crate) fn show_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    game_state: &mut GameState,
) -> Result<bool, Box<dyn Error>> {
    let mut last_frame_time = Instant::now();

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
    loop {
        let elapsed_time = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        // Get the size of the terminal
        let size = terminal.size()?;
        if size.height < 47 {
            return Err(Box::new(GameError(
                "Console should be at least 47 lines tall",
            )));
        }

        // Create a layout with one row and one column
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)].as_ref())
            .split(Rect {
                x: 0,
                y: 0,
                width: size.width,
                height: 47,
            });

        // Wait for a key press event
        // let tick_rate = Duration::from_millis(250);
        // let timeout = tick_rate
        //     .checked_sub(last_tick.elapsed())
        //     .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(false),
                    KeyCode::Down => items.next(),
                    KeyCode::Up => items.previous(),
                    KeyCode::Enter => {
                        game_state.language = languages
                            .get(items.state.selected().unwrap())
                            .unwrap()
                            .clone();
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Draw the list in the center of the layout
        terminal.draw(|f| {
            let inner_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(0)].as_ref())
                .split(layout[0]);
            f.render_widget(list.clone(), inner_layout[0]);
            f.render_stateful_widget(list.clone(), inner_layout[0], &mut items.state)
        })?;

        // Sleep to maintain desired FPS
        let time_to_sleep = FRAME_TIME
            .checked_sub(elapsed_time)
            .unwrap_or_else(|| Duration::from_micros(0));
        thread::sleep(time_to_sleep);
    }
    Ok(true)
}
