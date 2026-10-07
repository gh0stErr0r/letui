use std::{io, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{file, lsp::LspPanel, ABBREVIATIONS};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct App {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll: usize,
    pub mode: Mode,
    pub command: String,
    pub status: String,
    pub should_quit: bool,
    pub dirty: bool,
    pub lsp: LspPanel,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            lines: file::load(path.as_deref())?,
            path,
            cursor_row: 0,
            cursor_col: 0,
            scroll: 0,
            mode: Mode::Normal,
            command: String::new(),
            status: String::new(),
            should_quit: false,
            dirty: false,
            lsp: LspPanel::new(),
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.normal_key(key),
            Mode::Insert => self.insert_key(key),
            Mode::Command => self.command_key(key),
        }
    }

    fn normal_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('z') => self.should_quit = true,
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('a') => {
                self.move_right();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('o') => {
                self.lines.insert(self.cursor_row + 1, String::new());
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.mode = Mode::Insert;
                self.dirty = true;
            }
            KeyCode::Char('h') | KeyCode::Left => self.move_left(),
            KeyCode::Char('l') | KeyCode::Right => self.move_right(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('x') => self.delete_char(),
            KeyCode::Char('s') => self.save(),
            KeyCode::Char(':') => {
                self.command.clear();
                self.mode = Mode::Command;
            }
            _ => {}
        }
    }

    fn insert_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let tail = self.lines[self.cursor_row].split_off(self.cursor_col);
                self.lines.insert(self.cursor_row + 1, tail);
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.dirty = true;
            }
            KeyCode::Backspace => self.backspace(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Char(' ') => self.insert_space_with_abbreviation(),
            KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.lines[self.cursor_row].insert(self.cursor_col, character);
                self.cursor_col += character.len_utf8();
                self.dirty = true;
            }
            _ => {}
        }
    }

    fn insert_space_with_abbreviation(&mut self) {
        let line = &mut self.lines[self.cursor_row];
        let word_end = self.cursor_col;
        let word_start = line[..word_end]
            .rfind(char::is_whitespace)
            .map(|index| index + 1)
            .unwrap_or(0);
        let word = &line[word_start..word_end];

        if let Some(lookup_key) = word.strip_prefix('\\') {
            if let Some(abbreviation) = ABBREVIATIONS
                .iter()
                .find(|abbreviation| abbreviation.key == lookup_key && !lookup_key.is_empty())
            {
                line.replace_range(word_start..word_end, abbreviation.value);
                self.cursor_col = word_start + abbreviation.value.len();
            }
        }

        line.insert(self.cursor_col, ' ');
        self.cursor_col += 1;
        self.dirty = true;
    }

    fn command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let command = self.command.trim().to_owned();
                self.command.clear();
                self.mode = Mode::Normal;
                match command.as_str() {
                    "q" => self.should_quit = true,
                    "w" => self.save(),
                    "wq" => {
                        self.save();
                        self.should_quit = true;
                    }
                    _ => self.status = format!("unknown command: {command}"),
                }
            }
            KeyCode::Backspace => {
                self.command.pop();
            }
            KeyCode::Char(character) => self.command.push(character),
            _ => {}
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            while !self.lines[self.cursor_row].is_char_boundary(self.cursor_col) {
                self.cursor_col -= 1;
            }
        }
    }

    fn move_right(&mut self) {
        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            self.cursor_col += 1;
            while !line.is_char_boundary(self.cursor_col) {
                self.cursor_col += 1;
            }
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_column();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_column();
        }
    }

    fn clamp_column(&mut self) {
        self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        while !self.lines[self.cursor_row].is_char_boundary(self.cursor_col) {
            self.cursor_col -= 1;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let previous = self.cursor_col - 1;
            self.cursor_col = previous;
            while !self.lines[self.cursor_row].is_char_boundary(self.cursor_col) {
                self.cursor_col -= 1;
            }
            self.lines[self.cursor_row].drain(self.cursor_col..previous + 1);
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current);
        }
        self.dirty = true;
    }

    fn delete_char(&mut self) {
        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            let end = line[self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(index, _)| self.cursor_col + index)
                .unwrap_or(line.len());
            line.drain(self.cursor_col..end);
            self.dirty = true;
        }
    }

    fn save(&mut self) {
        match file::save(self.path.as_deref(), &self.lines) {
            Ok(()) => {
                self.dirty = false;
                self.status = match &self.path {
                    Some(path) => format!("written {}", path.display()),
                    None => String::new(),
                };
            }
            Err(error) => self.status = error.to_string(),
        }
    }
}
