use std::time::Duration;

use color_eyre::eyre::Result;
use color_eyre::eyre::{WrapErr, bail};
use ratatui::widgets::{Row, Table};
use reqwest::StatusCode;
use serde_json::Value;
use tokio;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::layout::{Constraint, Layout};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::default().run(terminal).await;
    ratatui::restore();
    result
}

async fn get_items() -> Result<Vec<Value>> {
    let res = reqwest::get("https://jsonplaceholder.typicode.com/users").await?;
    match res.status() {
        StatusCode::OK => {
            let body = res.text().await?;
            let items: Vec<Value> = serde_json::from_str(&body).wrap_err("Error parsing json")?;
            Ok(items)
        }
        _ => {
            bail!("Got status: {}", &res.status());
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    // Event stream.
    event_stream: EventStream,
    last_error: String,
    status: String,
    items: Vec<Value>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            status: "Starting...".to_string(),
            ..Default::default()
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let result = get_items().await;
                if tx.send(result).await.is_err() {
                    break;
                }
            }
        });
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events(&mut rx).await?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn draw(&mut self, frame: &mut Frame) {
        let [main_area, status_area] = Layout::vertical([
            Constraint::Min(0),    // main content: takes all remaining space
            Constraint::Length(1), // status bar: exactly 1 line tall
        ]).areas(frame.area());
        let header = Row::new(vec!["ID", "Name", "Email"]).bold();
        let rows: Vec<Row> = self.items.iter().map(|item| {
            Row::new(vec![
                item["id"].to_string(),
                item["name"].as_str().unwrap_or("").to_string(),
                item["email"].as_str().unwrap_or("").to_string(),
            ])
        }).collect();
        let table = Table::new(rows, [
            Constraint::Length(4),
            Constraint::Min(20),
            Constraint::Min(30),
        ]).header(header).block(Block::bordered().title("Users"));
        frame.render_widget(
            table,
            main_area,
        );
        let status_line = if !self.last_error.is_empty() {
            Line::from(self.last_error.as_str()).red()
        } else {
            Line::from(self.status.as_str()).green()
        };
        frame.render_widget(Paragraph::new(status_line), status_area);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(
        &mut self,
        rx: &mut tokio::sync::mpsc::Receiver<Result<Vec<Value>>>,
    ) -> color_eyre::Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                 match event {
                     Some(Ok(evt)) => match evt {
                         Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                         Event::Mouse(_) => {}
                         Event::Resize(_, _) => {}
                         _ => {}
                     },
                     _ => {}
                }
            }
            Some(data) = rx.recv() => {
                match data {
                    Ok(items) => {
                        self.items = items;
                        self.status = format!("Got {} items", self.items.len());
                        self.last_error.clear();
                    }
                    Err(e) => {
                        self.last_error = e.to_string();
                        self.status = "Error".to_string();
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (modifier, key) => { self.status = format!("Got keypress {} {}", modifier, key); }
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
