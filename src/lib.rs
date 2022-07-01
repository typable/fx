use console::Color;
use console::Term;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

#[macro_export]
macro_rules! color {
    ($str:expr, $fg:expr) => {
        &format!("{}", console::style($str).fg($fg))
    };
    ($str:expr, $fg:expr, $bg:expr) => {
        &format!("{}", console::style($str).fg($fg).bg($bg))
    };
}

#[macro_export]
macro_rules! pad {
    ($str:expr, $width:expr) => {
        &format!("{: <width$}", $str, width = $width)
    };
    ($str:expr, $width:expr, $max_width:expr) => {
        &format!(
            "{: <width$}",
            if $str.len() > $max_width {
                $str.split_at($max_width).0
            } else {
                $str
            },
            width = $width
        )
    };
}

pub const APP_NAME: &str = "fx";
pub const MARGIN: usize = 8;
pub const PADDING: usize = 2;
pub const WIDTH: usize = 40;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Prompt,
}

#[derive(PartialEq)]
pub enum Move {
    Up,
    Down,
    Next,
    Prev,
    First,
    Top,
    Bottom,
}

#[derive(PartialEq)]
pub enum FolderDir {
    Parent,
    Child,
    Home,
}

#[derive(Clone, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    // The default app for opening files
    pub default: Option<String>,
}

impl Config {
    pub fn acquire() -> Result<Self> {
        match config_path() {
            Some(config_path) => match fs::read_to_string(config_path) {
                Ok(raw) => match toml::from_str(&raw) {
                    Ok(config) => Ok(config),
                    Err(err) => {
                        return Err(Error::new(&format!("Invalid config file! Reason: {}", err)))
                    }
                },
                Err(_) => Ok(Config::default()),
            },
            None => Err(Error::new("Unable to determine config path!")),
        }
    }
}

pub struct Message {
    text: String,
    color: Color,
}

impl Message {
    pub fn info(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: Color::White,
        }
    }
    pub fn error(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: Color::Red,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", color!(&self.text, self.color))
    }
}

pub struct State {
    // The config file
    pub config: Config,
    // The terminal struct
    pub term: Term,
    // The current directory path
    pub path: PathBuf,
    // The current interaction mode
    pub mode: Mode,
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
        Self {
            config,
            term: Term::stdout(),
            path,
            mode: Mode::Normal,
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
}

#[derive(Clone)]
pub struct Entry {
    pub file_name: String,
    pub kind: EntryKind,
}

impl Entry {
    // Returns the corresponding color for the file type
    pub fn to_color(&self) -> Color {
        if self.file_name.starts_with('.') {
            return Color::Color256(247);
        }
        Color::White
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::new(&error.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

fn config_path() -> Option<PathBuf> {
    match dirs::config_dir() {
        Some(mut config_dir) => {
            config_dir.push(APP_NAME);
            config_dir.push("config.toml");
            Some(config_dir)
        }
        None => None,
    }
}
