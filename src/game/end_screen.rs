use std::{
    error::Error,
    io::Stdout,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Spans,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
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
                .title("Options:")
                .borders(tui::widgets::Borders::NONE),
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

        // Create a layouts and widgets
        let main_pane = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)].as_ref())
            .split(Rect {
                x: 0,
                y: 0,
                width: size.width,
                height: 47,
            });
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Type Defender ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let inner_pane = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints([Constraint::Length(4), Constraint::Percentage(100)].as_ref())
            .split(main_pane[0]);
        let end_message_text = vec![
            Spans::from("Game Over!"),
            Spans::from(""),
            Spans::from(format!("Score: {:.1}", game_state.score)),
        ];
        let end_message_paragraph = Paragraph::new(end_message_text);

        // Render terminal
        terminal.draw(|f| {
            f.render_widget(block, main_pane[0]);
            f.render_widget(end_message_paragraph, inner_pane[0]);
            f.render_widget(list.clone(), inner_pane[1]);
            f.render_stateful_widget(list.clone(), inner_pane[1], &mut items.state)
        })?;

        // Listen for a key press events
        let poll_time = FRAME_TIME
            .checked_sub(Duration::from_millis(5))
            .unwrap_or_else(|| Duration::from_micros(0));
        if event::poll(poll_time)? {
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

        // Sleep to maintain desired FPS
        let time_to_sleep = FRAME_TIME
            .checked_sub(elapsed_time)
            .unwrap_or_else(|| Duration::from_micros(0));
        thread::sleep(time_to_sleep);
    }
}
