use std::io;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
use console::Term;
use console::Key;
use console::Color;

macro_rules! color {
    ($str:expr, $fg:expr) => {
        &format!("{}", console::style($str).fg($fg))
    };
    ($str:expr, $fg:expr, $bg:expr) => {
        &format!("{}", console::style($str).fg($fg).bg($bg))
    };
}

#[derive(Debug)]
struct State {
    path: PathBuf,
    index: usize,
    list: Vec<DirEntry>,
}

fn main() {
    init_ui().unwrap();
}

fn init_ui() -> io::Result<()> {
    let term = Term::stdout();
    term.hide_cursor()?;
    term.clear_screen()?;
    let path = fs::canonicalize(&Path::new("./"))?;
    let mut state = State { path, index: 0, list: Vec::new() };
    let mut prev_key = None;
    read_dir(&mut state)?;
    print(&term, &state)?;
    loop {
        let key = term.read_key()?;
        match key {
            Key::Char('q') => break,
            Key::Char('j') => {
                if state.list.len() > 0 && state.index < state.list.len() - 1 {
                    state.index += 1;
                    term.clear_screen()?;
                    print(&term, &state)?;
                }
            },
            Key::Char('k') => {
                if state.index > 0 {
                    state.index -= 1;
                    term.clear_screen()?;
                    print(&term, &state)?;
                }
            },
            Key::Char('h') => {
                if let Some(parent) = state.path.parent() {
                    state.path = parent.to_path_buf();
                    state.index = 0;
                    read_dir(&mut state)?;
                    term.clear_screen()?;
                    print(&term, &state)?;
                }
            },
            Key::Char('l') => {
                if state.list.len() > 0 {
                    let item = &state.list[state.index];
                    let path = item.path();
                    let file_name = item.file_name().into_string().unwrap();
                    if path.is_dir() {
                        state.path.push(file_name);
                        state.index = 0;
                        read_dir(&mut state)?;
                        term.clear_screen()?;
                        print(&term, &state)?;
                    }
                }
            },
            Key::Char('g') => {
                if let Some(prev_key) = prev_key {
                    if prev_key == Key::Char('g') {
                        if state.list.len() > 0 {
                            state.index = 0;
                            read_dir(&mut state)?;
                            term.clear_screen()?;
                            print(&term, &state)?;
                        }
                    }
                }
            },
            Key::Char('G') => {
                if state.list.len() > 0 {
                    state.index = state.list.len() - 1;
                    read_dir(&mut state)?;
                    term.clear_screen()?;
                    print(&term, &state)?;
                }
            },
            _ => (),
        }
        prev_key = Some(key);
    }
    term.clear_screen()?;
    term.show_cursor()?;
    Ok(())
}

fn read_dir(state: &mut State) -> io::Result<()> {
    let mut list = Vec::new();
    for item in fs::read_dir(&state.path)? {
        list.push(item?);
    }
    state.list = list;
    Ok(())
}

fn print(term: &Term, state: &State) -> io::Result<()> {
    let (height, _) = term.size();
    let path = state.path.display().to_string();
    term.write_line(&format!("   {}", path))?;
    term.write_line("")?;
    for (i, item) in state.list.iter().enumerate() {
        if i < height as usize - 3 {
            let path = item.path();
            let file_name = item.file_name().into_string().unwrap();
            let pointer = if state.index == i { ">" } else { " " };
            let color = if path.is_dir() { Color::Blue } else { Color::White };
            term.write_line(&format!(" {} {}", pointer, color!(file_name, color)))?;
        }
    }
    term.hide_cursor()?;
    Ok(())
}