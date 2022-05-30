use console::Color;
use console::Key;
use console::Term;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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
    path: PathBuf,
    index: usize,
    list: Vec<Entry>,
    lines: usize,
    offset: usize,
    selected: Vec<usize>,
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

fn init_ui() -> io::Result<()> {
    let term = Term::stdout();
    term.hide_cursor()?;
    term.clear_screen()?;
    let path = fs::canonicalize(&Path::new("/home/andreas"))?;
    let mut state = State {
        path,
        index: 0,
        list: Vec::new(),
        lines: 0,
        offset: 0,
        selected: Vec::new(),
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
                    if state.index >= height as usize + state.offset - 4 - 5 {
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
                    state.selected.push(state.index);
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

fn print(term: &Term, state: &mut State) -> io::Result<()> {
    let (height, _) = term.size();
    let path = state.path.display().to_string();
    term.clear_last_lines(state.lines)?;
    term.write_line("")?;
    term.write_line(&format!("   {}", path))?;
    term.write_line("")?;
    state.lines = 3;
    for (i, entry) in state.list.iter().enumerate() {
        if i < state.offset {
            continue;
        }
        if i == height as usize + state.offset - 4 {
            break;
        }
        let pointer = if state.index == i { ">" } else { " " };
        let color = match entry.kind {
            EntryKind::Dir => Color::Blue,
            EntryKind::File => entry.to_color(),
        };
        let value = match state.selected.contains(&i) {
            true => color!(pad!(&entry.file_name, 50), Color::Black, color).to_string(),
            false => color!(pad!(&entry.file_name, 50), color).to_string(),
        };
        term.write_line(&format!(" {} {}", pointer, value))?;
        state.lines += 1;
    }
    term.hide_cursor()?;
    Ok(())
}
