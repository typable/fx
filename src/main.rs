use console::Color;
use console::Key;
use console::Term;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

macro_rules! color {
    ($str:expr, $fg:expr) => {
        &format!("{}", console::style($str).fg($fg))
    };
    ($str:expr, $fg:expr, $bg:expr) => {
        &format!("{}", console::style($str).fg($fg).bg($bg))
    };
}

macro_rules! pad {
    ($str:expr, $width:expr) => {
        &format!("{: <width$}", $str, width = $width)
    };
}

#[derive(Debug)]
struct State {
    // The current directory path
    path: PathBuf,
    // The current index in the file list
    index: usize,
    /// The list of files in the current directory
    list: Vec<Entry>,
    // The count of printed lines to the screen
    lines: usize,
    // The offset for printing the file list
    offset: usize,
    // The list of selected files
    selected: Vec<usize>,
    // The info message to display on screen
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum EntryKind {
    Dir,
    File,
}

#[derive(Debug, Clone)]
struct Entry {
    file_name: String,
    kind: EntryKind,
}

impl Entry {
    /// Returns the corresponding color for the file type
    fn to_color(&self) -> Color {
        if self.file_name.starts_with(".") {
            return Color::Color256(247);
        }
        Color::White
    }
}

fn main() {
    init_ui().unwrap();
}

/// Initializes the terminal user interface
fn init_ui() -> io::Result<()> {
    let term = Term::stdout();
    let mut arguments = env::args();
    arguments.next();
    let current_dir = match arguments.next() {
        Some(dir) => dir,
        None => "./".into(),
    };
    let path = match fs::canonicalize(&Path::new(&current_dir)) {
        Ok(path) => path,
        Err(_) => {
            term.write_line(color!(
                &format!("Invalid arguments! '{}' is not a valid path!", &current_dir),
                Color::Red
            ))?;
            process::exit(1);
        }
    };
    term.hide_cursor()?;
    term.clear_screen()?;
    let mut state = State {
        path,
        index: 0,
        list: Vec::new(),
        lines: 0,
        offset: 0,
        selected: Vec::new(),
        message: None,
    };
    let mut prev_key = None;
    read_dir(&mut state)?;
    print(&term, &mut state)?;
    loop {
        let mut key = Some(term.read_key()?);
        let (height, _) = term.size();
        match key.clone().unwrap() {
            Key::Char('q') => break,
            Key::Char('j') => {
                if state.list.len() > 0 && state.index < state.list.len() - 1 {
                    state.index += 1;
                    if state.index >= state.lines + state.offset - 5 - 5 {
                        state.offset += 1;
                    }
                    print(&term, &mut state)?;
                }
            }
            Key::Char('k') => {
                if state.index > 0 {
                    state.index -= 1;
                    if state.offset > 0 && state.index - state.offset < 5 {
                        state.offset -= 1;
                    }
                    print(&term, &mut state)?;
                }
            }
            Key::Char('h') => {
                if let Some(parent) = state.path.parent() {
                    state.path = parent.to_path_buf();
                    state.index = 0;
                    state.offset = 0;
                    state.selected.clear();
                    read_dir(&mut state)?;
                    print(&term, &mut state)?;
                }
            }
            Key::Char('l') => {
                if state.list.len() > 0 {
                    let entry = &state.list[state.index];
                    if entry.kind == EntryKind::Dir {
                        state.path.push(&entry.file_name);
                        state.index = 0;
                        state.offset = 0;
                        state.selected.clear();
                        read_dir(&mut state)?;
                        print(&term, &mut state)?;
                    }
                }
            }
            Key::Char('g') => {
                if let Some(prev_key) = &prev_key {
                    match prev_key {
                        Key::Char('g') => {
                            if state.list.len() > 0 {
                                state.index = 0;
                                state.offset = 0;
                                read_dir(&mut state)?;
                                print(&term, &mut state)?;
                                key = None;
                            }
                        }
                        _ => (),
                    }
                }
            }
            Key::Char('e') => {
                if let Some(prev_key) = &prev_key {
                    match prev_key {
                        Key::Char('g') => {
                            if state.list.len() > 0 {
                                state.index = state.list.len() - 1;
                                if state.index >= height as usize + state.offset - 4 - 5 {
                                    state.offset = state.list.len() - height as usize + 5 + 4;
                                }
                                read_dir(&mut state)?;
                                print(&term, &mut state)?;
                                key = None;
                            }
                        }
                        _ => (),
                    }
                }
            }
            Key::Char('x') => {
                if state.list.len() > 0 {
                    let index = state.selected.iter().position(|i| i == &state.index);
                    match index {
                        Some(index) => {
                            state.selected.remove(index);
                        }
                        None => {
                            state.selected.push(state.index);
                        }
                    }
                    match state.selected.len() {
                        0 => state.message = None,
                        _ => {
                            state.message =
                                Some(format!("   {} items selected", state.selected.len()))
                        }
                    }
                    print(&term, &mut state)?;
                }
            }
            _ => (),
        }
        prev_key = key;
    }
    term.clear_last_lines(state.lines)?;
    term.show_cursor()?;
    Ok(())
}

/// Reads the current directory
fn read_dir(state: &mut State) -> io::Result<()> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for dir_entry in fs::read_dir(&state.path)? {
        let item = dir_entry?;
        let path = item.path();
        let file_name = item.file_name().into_string().unwrap();
        let kind = match &path.is_dir() {
            true => EntryKind::Dir,
            false => EntryKind::File,
        };
        let entry = Entry { file_name, kind };
        match entry.kind {
            EntryKind::Dir => dirs.push(entry),
            EntryKind::File => files.push(entry),
        }
    }
    let mut list = Vec::new();
    list.extend_from_slice(&dirs);
    list.extend_from_slice(&files);
    state.list = list;
    Ok(())
}

/// Prints the current directory entries to the screen
fn print(term: &Term, state: &mut State) -> io::Result<()> {
    let (height, _) = term.size();
    let lines = height as usize - 1;
    let path = state.path.display().to_string();
    term.clear_last_lines(state.lines)?;
    for i in 0..lines {
        if i == 1 {
            term.write_line(&format!("   {}", path))?;
            continue;
        }
        if i == 3 && state.offset > 0 {
            term.write_line("   ...")?;
            continue;
        }
        if i == lines - 3 {
            let index = i - 3 + state.offset;
            if state.list.len() > index {
                term.write_line("   ...")?;
                continue;
            }
        }
        if i > 2 && i < lines - 2 {
            let index = i - 3 + state.offset;
            if state.list.len() > index {
                let entry = &state.list[index];
                let pointer = if state.index == index { ">" } else { " " };
                let color = match entry.kind {
                    EntryKind::Dir => Color::Blue,
                    EntryKind::File => entry.to_color(),
                };
                let value = match state.selected.contains(&index) {
                    true => color!(pad!(&entry.file_name, 50), Color::Black, color).to_string(),
                    false => color!(pad!(&entry.file_name, 50), color).to_string(),
                };
                term.write_line(&format!(" {} {}", pointer, value))?;
                continue;
            }
        }
        if i == lines - 1 {
            match &state.message {
                Some(message) => term.write_line(&message)?,
                None => term.write_line("")?,
            }
            continue;
        }
        term.write_line("")?;
    }
    state.lines = lines;
    term.hide_cursor()?;
    Ok(())
}
