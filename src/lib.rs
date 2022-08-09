use console::Color;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

mod config;
mod state;

pub mod consts;
pub mod error;

use error::Error;

pub use config::Config;
pub use state::State;

#[macro_export]
macro_rules! color {
    ($str:expr, $fg:expr$(,)?) => {
        &format!("{}", console::style($str).fg($fg))
    };
    ($str:expr, $fg:expr, $bg:expr$(,)?) => {
        &format!("{}", console::style($str).fg($fg).bg($bg))
    };
}

#[macro_export]
macro_rules! pad {
    ($str:expr, $width:expr$(,)?) => {
        &format!("{: <width$}", $str, width = $width)
    };
    ($str:expr, $width:expr, $max_width:expr$(,)?) => {
        &format!(
            "{: <width$}",
            if $str.len() > $max_width {
                $str.split_at($max_width).0.to_string()
            } else {
                $str.to_string()
            },
            width = $width
        )
    };
}

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
    pub size: usize,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        EntryKind::Dir.eq(&self.kind)
    }
    pub fn is_file(&self) -> bool {
        EntryKind::File.eq(&self.kind)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Column {
    Name,
    Type,
    Size,
    Created,
}

impl Column {
    pub fn get_width(&self) -> usize {
        match *self {
            Self::Name => 40,
            Self::Type => 10,
            Self::Size => 15,
            Self::Created => 22,
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Name => "NAME",
                Self::Type => "TYPE",
                Self::Size => "SIZE",
                Self::Created => "CREATED",
            }
        )
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
