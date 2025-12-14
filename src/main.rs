use std::{env, fs, io};

use crossterm::event;
use crossterm::event::{
    KeyCode, {Event, KeyEvent, KeyEventKind},
};
use ratatui::style::Style;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, List, Widget},
};

// App defines the app state
#[derive(Debug, Default)]
struct App {
    current_path: String,
    exit: bool, // if multiple modes or main states, might need an enum for this
}

impl App {
    // This is the application's main loop
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let /*mut*/ path = match env::args().nth(1) {
            Some(path) => path,
            None => String::from("."),
        };
        println!("path: {:?}", path);
        self.current_path = path;

        while !self.exit {
            // This is what shows the TUI
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Directory").bold();
        let instructions = Line::from(vec!["Q:".into(), "Quit".blue().bold()]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        List::new(list_current_directory(self.current_path.as_str()))
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    print_entries();
    app_result
}

fn print_entries() {
    let mut path = match env::args().nth(1) {
        Some(path) => path,
        None => String::from(""),
    };

    if path.eq("") {
        let current_dir_result = match env::current_dir() {
            Ok(cur) => cur,
            Err(err) => panic!("unable to get current directory: {err:?}"),
        };
        let current_dir_buf = current_dir_result.to_string_lossy().to_string();
        path = String::from(current_dir_buf.trim_ascii_end().trim_ascii_start());
    }

    let entries = list_current_directory(&path);

    for entry in entries {
        println!("{}", entry);
    }
}

fn list_current_directory(path: &str) -> Vec<String> {
    let dir_result = fs::read_dir(path);
    let dir = match dir_result {
        Ok(dir) => dir,
        Err(err) => panic!("unable to read current dirctory: {err:?}"),
    };

    let dir_entries = dir
        // for now, it's okay to ignore the errors.
        // TODO: when the time is right, do not ignore these errors and display something else that
        // would be better
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    return dir_entries;
}
