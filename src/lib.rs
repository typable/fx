use console::Color;
use console::Term;
use serde::{Deserialize, Serialize};
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
}

pub const APP_NAME: &'static str = "fx";

pub type Result<T> = std::result::Result<T, Error>;

pub type Context<'a> = (&'a Term, &'a mut State);

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Search,
}

#[derive(PartialEq)]
pub enum Move {
    Up,
    Down,
}

#[derive(Clone, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    // The offset to the top and bottom of the file list
    pub padding: usize,
}

impl Config {
    pub fn acquire() -> Result<Self> {
        match config_path() {
            Some(config_path) => match fs::read_to_string(config_path) {
                Ok(raw) => match toml::from_str(&raw) {
                    Ok(config) => return Ok(config),
                    Err(_) => return Err(Error::new("Invalid config file!")),
                },
                Err(_) => return Ok(Config::default()),
            },
            None => return Err(Error::new("Unable to determine config path!")),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { padding: 2 }
    }
}

pub struct State {
    // The config file
    pub config: Config,
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
    pub message: Option<String>,
    // The search for filtering the file list
    pub search: Option<String>,
}

impl State {
    pub fn new(config: Config, path: PathBuf) -> Self {
        Self {
            config,
            path,
            mode: Mode::Normal,
            index: 0,
            list: Vec::new(),
            lines: 0,
            offset: 0,
            selected: Vec::new(),
            message: None,
            search: None,
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
        if self.file_name.starts_with(".") {
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
