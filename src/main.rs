use std::env;
use std::path::{self, Path};
use std::{fs, io};

use crossterm::event;
use crossterm::event::{
    KeyCode, {Event, KeyEvent, KeyEventKind},
};

use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListState},
};

// App defines the app state
#[derive(Debug)]
struct App {
    list: Vec<DirEntryInfo>,
    list_state: ListState,
    current_path: String,
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

        let starting_list = list_current_directory(&path);

        Self {
            list: starting_list,
            list_state: state,
            current_path: path,
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
        let instructions = Line::from(vec![
            " Q:".into(),
            "Quit |".blue().bold(),
            " ↑/k: ".into(),
            "Up |".blue().bold(),
            " ←/h: ".into(),
            "Left |".blue().bold(),
            " →/l: ".into(),
            "Right |".blue().bold(),
            " ↓/j: ".into(),
            "Down |".blue().bold(),
            " Enter: ".into(),
            "Go to directory ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let better_dir_list: Vec<Line> = self
            .list
            .iter()
            .map(|e| {
                if e.is_dir {
                    Line::from(e.name.clone()).blue().bold()
                } else {
                    Line::from(e.name.clone().white().italic())
                }
            })
            .collect();

        List::new(better_dir_list)
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().bold())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .highlight_spacing(HighlightSpacing::Always)
    }

    fn update_dir_list(&mut self) -> Vec<DirEntryInfo> {
        let dir_entries = list_current_directory(&self.current_path);
        self.list = dir_entries
            .iter()
            .map(|e| DirEntryInfo {
                name: e.name.clone(),
                is_dir: e.is_dir,
            })
            .collect();

        dir_entries
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
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('h') | KeyCode::Left => self.open_previous_dir(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('l') | KeyCode::Right => self.open_next_dir(),
            KeyCode::Enter => self.take_to_path(),
            _ => {}
        }
    }

    fn move_down(&mut self) {
        self.list_state.select_next();
    }

    fn move_up(&mut self) {
        self.list_state.select_previous();
    }

    fn open_next_dir(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => i,
            None => 0,
        };

        // let updated_path = format!("{}/{}", self.path, self.list[i]);
        let new = Path::new(self.list[i].name.as_str());
        let path_curr = Path::new(self.current_path.as_str());
        let path_new_path = path_curr.join(new);

        self.current_path = path_new_path.to_string_lossy().to_string();
        self.update_dir_list();
        self.list_state.select_first();
    }

    fn open_previous_dir(&mut self) {
        let mut path_iter = self.current_path.split(path::MAIN_SEPARATOR);
        path_iter.next_back();

        let p: String = path_iter
            .map(|s| s.to_string() + String::from(path::MAIN_SEPARATOR).as_str())
            .collect();

        self.current_path = p;
        self.update_dir_list();
        self.list_state.select_first();
    }

    // TODO: This needs work -- not working as intended
    fn take_to_path(&mut self) {
        let p = Path::new(&self.current_path);
        let _ = env::set_current_dir(p);
        println!("current working directory: {}", p.display());
        self.exit();
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    app_result
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
    let mut dir_entries: Vec<DirEntryInfo> = dir
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

    dir_entries.sort_by_key(|en| (!en.name.starts_with("."), en.name.to_lowercase()));

    return dir_entries;
}
