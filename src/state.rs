use std::{collections::HashMap, path::PathBuf};

use console::Term;

use crate::Column;
use crate::Config;
use crate::Entry;
use crate::Message;
use crate::Mode;

pub struct State {
    // The config file
    pub config: Config,
    // The terminal struct
    pub term: Term,
    // The current directory path
    pub path: PathBuf,
    // The current interaction mode
    pub mode: Mode,
    // The displayable columns
    pub columns: Vec<Column>,
    // The current index in the file list
    pub index: usize,
    // The list of files in the current directory
    pub list: Vec<Entry>,
    // The count of printed lines to the screen
    pub lines: usize,
    // The offset for printing the file list
    pub offset: usize,
    // The list of selected files
    pub selected: Vec<usize>,
    // The info message to display on screen
    pub message: Option<Message>,
    // The prompt title
    pub title: Option<String>,
    // The input field
    pub input: Option<String>,
    // The cursor index for the input field
    pub cursor: usize,
    // The flag if dotfiles should be listed
    pub show_dotfiles: bool,
    // The history index
    pub history_index: usize,
    // The history
    pub history: HashMap<String, Vec<String>>,
}

impl State {
    pub fn new(config: Config, path: PathBuf) -> Self {
        let columns = config.get_columns();
        Self {
            config,
            term: Term::stdout(),
            path,
            mode: Mode::Normal,
            columns,
            index: 0,
            list: Vec::new(),
            lines: 0,
            offset: 0,
            selected: Vec::new(),
            message: None,
            title: None,
            input: None,
            cursor: 0,
            show_dotfiles: true,
            history_index: 0,
            history: HashMap::new(),
        }
    }
    // Get currently selected entry in list
    pub fn get_current(&self) -> Option<&Entry> {
        if self.list.is_empty() {
            return None;
        }
        Some(&self.list[self.index])
    }
    // Set message
    pub fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }
}
