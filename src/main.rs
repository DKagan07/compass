use std::{env, fs, io};

use crossterm::event;
use crossterm::event::{
    KeyCode, {Event, KeyEvent, KeyEventKind},
};
use ratatui::style::Style;
// use ratatui::widgets::StatefulWidget;
use ratatui::{
    DefaultTerminal,
    Frame,
    // buffer::Buffer,
    // layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListState /*Widget*/},
};

// App defines the app state
#[derive(Debug)]
struct App {
    list_state: ListState,
    current_path: String,
    dir_list: Vec<DirEntryInfo>,
    exit: bool, // if multiple modes or main states, might need an enum for this
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DirEntryInfo {
    name: String,
    is_dir: bool,
}

impl DirEntryInfo {
    fn new(name: String, is_dir: bool) -> Self {
        Self {
            name: name,
            is_dir: is_dir,
        }
    }
}

impl Default for App {
    fn default() -> Self {
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

        let mut state = ListState::default();
        state.select_first();

        Self {
            list_state: state,
            current_path: path,
            dir_list: vec![],
            exit: false,
        }
    }
}

impl App {
    // This is the application's main loop
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            // This is what shows the TUI
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let list = self.build_list_widget();
        frame.render_stateful_widget(list, frame.area(), &mut self.list_state)
    }

    fn build_list_widget(&self) -> List<'static> {
        let title = Line::from(" Directory ").bold();
        let instructions = Line::from(vec![" Q:".into(), "Quit ".blue().bold()]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let dir_entries = list_current_directory(self.current_path.as_str());
        let better_dir_list: Vec<String> = dir_entries.iter().map(|e| e.name.clone()).collect();

        List::new(better_dir_list)
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().bold())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .highlight_spacing(HighlightSpacing::Always)
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
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('j') | KeyCode::Down => {}
            KeyCode::Char('h') | KeyCode::Left => {}
            KeyCode::Char('k') | KeyCode::Up => {}
            KeyCode::Char('l') | KeyCode::Right => {}
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
        println!("{}", entry.name);
    }
}

fn list_current_directory(path: &str) -> Vec<DirEntryInfo> {
    let dir_result = fs::read_dir(path);
    let dir = match dir_result {
        Ok(dir) => dir,
        Err(err) => panic!("unable to read current dirctory: {err:?}"),
    };

    // for now, it's okay to ignore the errors.
    // TODO: when the time is right, do not ignore these errors and display something else that
    // would be better
    let dir_entries: Vec<DirEntryInfo> = dir
        .filter_map(|entry| entry.ok())
        .map(|e| {
            let is_dir_bool = match e.file_type() {
                Ok(d) => d,
                Err(err) => panic!("unable to read file type for file {err:?}"),
            };

            DirEntryInfo::new(
                e.file_name().to_string_lossy().to_string(),
                is_dir_bool.is_dir(),
            )
        })
        .collect();

    return dir_entries;
}
