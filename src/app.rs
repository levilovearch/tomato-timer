use std::{io, sync::mpsc::Receiver};

use crossterm::{event, ExecutableCommand};
use terminal_fonts::to_block_string;
use tui::{Terminal, backend::CrosstermBackend, text::Text, style::Style, widgets::Paragraph, layout::{Alignment, Rect}};

use super::{Opts, Status, Event};
use anyhow::{Result as Result, Ok};
pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    left_seconds: u64,
    schedule: Opts,
    status: Status,
    rx: Receiver<Event<event::KeyEvent>>
}
impl App {
    pub fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>,schedule: Opts, rx: Receiver<Event<event::KeyEvent>>) -> Self {
        let status = Status::Work;
        let left_seconds = schedule.work_time * 60;
        Self { terminal, left_seconds, schedule, status, rx}
    }
    fn render(&mut self) -> Result<()>{
        self.terminal.draw(|f| {
            let minutes = self.left_seconds / 60;
            let seconds = self.left_seconds % 60;
            let block_string = to_block_string(&format!("{:02}:{:02}", minutes, seconds));
            let text = Text::raw(block_string);
            let text_height = text.height() as u16;
            let style = Style::default().fg(self.status.color());
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(style);
            let size = f.size();
            let y = (size.height - text_height) / 2;
            let rect = Rect::new(0, y, size.width, text_height);
            f.render_widget(paragraph, rect);
        })?;
        Ok(())
    }
    pub fn run(&mut self) -> Result<()> {
        self.render()?;
        match self.rx.recv()? {
            Event::Input(input) => {
                if input.code == event::KeyCode::Char('q')
                    || (input.code == event::KeyCode::Char('C')
                        && input.modifiers == event::KeyModifiers::CONTROL)
                {
                    quit(0)?;
                }
            }
            Event::Tick => {
                if self.schedule.repeats!=0 {
                    if self.left_seconds == 0 {
                        match self.status {
                            Status::Work => {
                                self.status = Status::Break;
                                self.left_seconds = self.schedule.break_time * 60;
                                notify("Your work time is up, take a break!");
                            }
                            Status::Break => {
                                notify("Your break time is up!!");
                                self.schedule.repeats -= 1;
                                self.status = Status::Work;
                                self.left_seconds = self.schedule.work_time * 60;
                            }
                        }
                    }
                    if self.left_seconds > 0 {
                        self.left_seconds -= 1;
                    }
                }
                else {
                    quit(0)?;
                }
            }
        }
        Ok(())
    }
}

fn quit(code: i32) -> Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    stdout.execute(crossterm::cursor::Show)?;
    std::process::exit(code);
}

fn notify(msg: &str) {
    let msg = msg.to_string();
    std::thread::spawn(move || {
        let _ = notify_rust::Notification::new()
            .summary("Tomato Timer")
            .body(msg.as_str())
            .show();
    });
}