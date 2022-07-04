use console::Color;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

mod config;
mod state;

pub mod error;

use error::Error;

pub use config::Config;
pub use state::State;

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
    File,
    Dir,
    Symlink,
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
    pub fn warn(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: Color::Yellow,
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

#[derive(Clone)]
pub struct Entry {
    pub file_name: String,
    pub kind: EntryKind,
    pub created: Option<SystemTime>,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        EntryKind::Dir.eq(&self.kind)
    }
    pub fn is_file(&self) -> bool {
        EntryKind::File.eq(&self.kind)
    }
}

pub struct Column {
    pub name: String,
    pub width: usize,
    pub visible: bool,
}

impl Column {
    pub fn new(name: &str, width: usize) -> Self {
        Self {
            name: name.to_string(),
            width,
            visible: true,
        }
    }
}

pub fn expand_tilde(path: PathBuf) -> Option<PathBuf> {
    if !path.starts_with("~") {
        return Some(path);
    }
    if path == Path::new("~") {
        return dirs::home_dir();
    }
    let mut home = match dirs::home_dir() {
        Some(home) => home,
        None => return None,
    };
    for item in path.iter().skip(1) {
        home.push(item);
    }
    Some(home)
}
