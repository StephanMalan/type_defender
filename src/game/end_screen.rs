use std::{
    error::Error,
    io::Stdout,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{List, ListItem},
    Terminal,
};

use crate::{GameError, GameState, StatefulList, FRAME_TIME};

pub(crate) fn show_view(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    game_state: &mut GameState,
) -> Result<bool, Box<dyn Error>> {
    let mut last_frame_time = Instant::now();

    // Create a list of options
    let mut items =
        StatefulList::with_items(vec![ListItem::new("Play again?"), ListItem::new("Exit")]);
    items.next();

    // Create the list widget and set its items
    let list = List::new(&*items.items)
        .block(
            tui::widgets::Block::default()
                .title(format!("Score: {:.1}", game_state.score))
                .borders(tui::widgets::Borders::ALL),
        )
        .highlight_symbol(">> ");

    // Set up the terminal and run the event loop
    terminal.clear()?;
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
        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(false),
                    KeyCode::Down => items.next(),
                    KeyCode::Up => items.previous(),
                    KeyCode::Enter => {
                        if items.state.selected().unwrap() == 0 {
                            return Ok(true);
                        }
                        return Ok(false);
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
}
