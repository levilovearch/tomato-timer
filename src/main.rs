use anyhow::Result;
use clap::Parser;
use crossterm::{event, ExecutableCommand};
use std::sync::mpsc;
use std::thread;
use std::{io, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};
mod app;
use app::App;

/// A tomato timer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    /// Work timer in minutes
    #[arg(short, long, default_value = "45")]
    work_time: u64,
    /// Break timer in minutes
    #[arg(short, long, default_value = "15")]
    break_time: u64,
    /// Repetition
    #[arg(short, long, default_value = "1")]
    repeats: u64,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let (tx, rx) = mpsc::channel();

    let tx_key_event: mpsc::Sender<Event<event::KeyEvent>> = tx.clone();
    thread::spawn(move || loop {
        if let event::Event::Key(key) = event::read().unwrap() {
            tx_key_event.send(Event::Input(key)).unwrap();
        }
    });
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        tx.send(Event::Tick).unwrap();
    });
    let mut app = App::new(terminal, opts, rx);
    loop {
        app.run()?;
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}
