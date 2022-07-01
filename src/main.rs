use console::Color;
use console::Key;
use fx::color;
use fx::pad;
use fx::Config;
use fx::Entry;
use fx::EntryKind;
use fx::Error;
use fx::FolderDir;
use fx::Message;
use fx::Mode;
use fx::Move;
use fx::Result;
use fx::State;
use fx::MARGIN;
use fx::PADDING;
use fx::WIDTH;
use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() {
    match init() {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}

fn init() -> Result<()> {
    let config = Config::acquire()?;
    let state = create_state(config)?;
    init_ui(state)?;
    Ok(())
}

fn create_state(config: Config) -> Result<State> {
    let mut arguments = env::args();
    arguments.next();
    let current_dir = match arguments.next() {
        Some(dir) => dir,
        None => "./".into(),
    };
    let path = match fs::canonicalize(&Path::new(&current_dir)) {
        Ok(path) => path,
        Err(_) => {
            return Err(Error::new(&format!(
                "Invalid arguments! '{}' is not a valid path!",
                &current_dir
            )))
        }
    };
    Ok(State::new(config, path))
}

// Initializes the user interface
fn init_ui(mut state: State) -> Result<()> {
    state.term.hide_cursor()?;
    state.term.clear_screen()?;
    read_dir(&mut state)?;
    print(&mut state)?;
    update_loop(&mut state)?;
    state.term.clear_last_lines(state.lines)?;
    state.term.show_cursor()?;
    Ok(())
}

fn update_loop(state: &mut State) -> Result<()> {
    loop {
        let key = state.term.read_key()?;
        match key {
            Key::Char('q') => break,
            Key::Char('j') => move_caret(state, Move::Down)?,
            Key::Char('k') => move_caret(state, Move::Up)?,
            Key::Char('h') => change_dir(state, FolderDir::Parent)?,
            Key::Char('l') => change_dir(state, FolderDir::Child)?,
            Key::Char(' ') => toggle_dotfiles(state)?,
            Key::Char('x') => toggle_select(state)?,
            Key::Char('%') => select_all(state)?,
            Key::Char('n') => move_caret(state, Move::Next)?,
            Key::Char('N') => move_caret(state, Move::Prev)?,
            Key::Char('g') => {
                let key = state.term.read_key()?;
                match key {
                    Key::Char('g') => move_caret(state, Move::Top)?,
                    Key::Char('e') => move_caret(state, Move::Bottom)?,
                    Key::Char('t') => prompt(state, "goto", &do_goto)?,
                    _ => (),
                }
            }
            Key::Char('z') => {
                let key = state.term.read_key()?;
                if let Key::Char('z') = key {
                    change_dir(state, FolderDir::Home)?;
                }
            }
            Key::Char('/') => prompt(state, "search", &do_search)?,
            Key::Escape => {
                state.selected.clear();
                state.message = None;
                print(state)?;
            }
            _ => (),
        }
    }
    Ok(())
}

fn do_search(state: &mut State) -> Result<()> {
    let input = state.input.clone().unwrap_or_default();
    if input.is_empty() {
        return Ok(());
    }
    match Regex::new(&input) {
        Ok(re) => {
            state.selected.clear();
            for (i, entry) in state.list.iter().enumerate() {
                if re.is_match(&entry.file_name) {
                    state.selected.push(i);
                }
            }
            set_select_message(state);
            move_caret(state, Move::First)?;
        }
        Err(_) => {
            state.message = Some(Message::error("Invalid search pattern!"));
        }
    }
    Ok(())
}

fn do_goto(state: &mut State) -> Result<()> {
    let input = state.input.clone().unwrap_or_default();
    if input.is_empty() {
        return Ok(());
    }
    match fs::canonicalize(&Path::new(&input)) {
        Ok(path) => {
            state.path = path;
            state.index = 0;
            state.offset = 0;
            state.message = None;
            state.selected.clear();
            read_dir(state)?;
        }
        Err(_) => {
            state.message = Some(Message::error("Invalid path!"));
        }
    }
    Ok(())
}

fn prompt(state: &mut State, title: &str, f: &dyn Fn(&mut State) -> Result<()>) -> Result<()> {
    let shift = 3 + title.len() + 1;
    state.title = Some(title.into());
    state.mode = Mode::Prompt;
    state.input = None;
    state.cursor = 0;
    print(state)?;
    state.term.move_cursor_to(shift, 1)?;
    state.term.show_cursor()?;
    loop {
        let key = state.term.read_key()?;
        match key {
            Key::Escape => {
                state.mode = Mode::Normal;
                state.term.hide_cursor()?;
                print(state)?;
                break;
            }
            Key::Backspace => {
                let mut search = state.input.clone().unwrap_or_default();
                if !search.is_empty() && state.cursor > 0 {
                    state.cursor -= 1;
                    search.remove(state.cursor);
                    state.input = Some(search);
                    state.term.hide_cursor()?;
                    print(state)?;
                    state.term.move_cursor_to(shift + state.cursor, 1)?;
                    state.term.show_cursor()?;
                }
            }
            Key::Del => {
                let mut search = state.input.clone().unwrap_or_default();
                if state.cursor < search.len() {
                    search.remove(state.cursor);
                    state.input = Some(search);
                    state.term.hide_cursor()?;
                    print(state)?;
                    state.term.move_cursor_to(shift + state.cursor, 1)?;
                    state.term.show_cursor()?;
                }
            }
            Key::Char(char) => {
                let mut search = state.input.clone().unwrap_or_default();
                search.insert(state.cursor, char);
                state.cursor += 1;
                state.input = Some(search);
                state.term.hide_cursor()?;
                print(state)?;
                state.term.move_cursor_to(shift + state.cursor, 1)?;
                state.term.show_cursor()?;
            }
            Key::ArrowLeft => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                    state.term.move_cursor_to(shift + state.cursor, 1)?;
                    state.term.show_cursor()?;
                }
            }
            Key::ArrowRight => {
                if state.cursor < state.input.clone().unwrap_or_default().len() {
                    state.cursor += 1;
                    state.term.move_cursor_to(shift + state.cursor, 1)?;
                    state.term.show_cursor()?;
                }
            }
            Key::Enter => {
                state.mode = Mode::Normal;
                state.term.hide_cursor()?;
                f(state)?;
                print(state)?;
                break;
            }
            _ => (),
        }
    }
    Ok(())
}

fn move_caret(state: &mut State, movement: Move) -> Result<()> {
    match movement {
        Move::Down => {
            if !state.list.is_empty() && state.index < state.list.len() - 1 {
                state.index += 1;
                if state.index >= state.lines + state.offset - MARGIN + 1 - PADDING
                    && state.list.len() - state.index > PADDING
                {
                    state.offset += 1;
                }
                print(state)?;
            }
        }
        Move::Up => {
            if !state.list.is_empty() && state.index > 0 {
                state.index -= 1;
                if state.offset > 0 && state.index - state.offset < PADDING {
                    state.offset -= 1;
                }
                print(state)?;
            }
        }
        Move::Next => {
            if !state.list.is_empty() && !state.selected.is_empty() {
                let mut selected = state.selected.clone();
                selected.sort_unstable();
                let mut next = selected[0];
                for index in selected {
                    if state.index < index {
                        next = index;
                        break;
                    }
                }
                state.index = next;
                if state.index < state.lines - MARGIN - PADDING {
                    // caret is visible on the screen without any offset
                    state.offset = 0;
                } else if state.index - state.offset > state.lines - MARGIN - PADDING {
                    if state.list.len() - state.index <= PADDING {
                        // caret is beyond the screen and (almost) at the end of the list
                        state.offset =
                            state.index - state.lines + MARGIN + state.list.len() - state.index - 1;
                    } else {
                        // caret is beyond the screen
                        state.offset = state.index - (state.lines - MARGIN - PADDING);
                    }
                }
                print(state)?;
            }
        }
        Move::Prev => {
            if !state.list.is_empty() && !state.selected.is_empty() {
                let mut selected = state.selected.clone();
                selected.sort_unstable();
                let mut prev = selected[selected.len() - 1];
                for index in selected.iter().cloned().rev() {
                    if state.index > index {
                        prev = index;
                        break;
                    }
                }
                state.index = prev;
                // TODO: Adjust logic in order to set offset correctly
                // if state.index < state.lines - MARGIN - PADDING {
                //     // caret is visible on the screen without any offset
                //     state.offset = 0;
                // } else if state.index - state.offset > state.lines - MARGIN - PADDING {
                //     if state.list.len() - state.index <= PADDING {
                //         // caret is beyond the screen and (almost) at the end of the list
                //         state.offset =
                //             state.index - state.lines + MARGIN + state.list.len() - state.index - 1;
                //     } else {
                //         // caret is beyond the screen
                //         state.offset = state.index - (state.lines - MARGIN - PADDING);
                //     }
                // }
                print(state)?;
            }
        }
        Move::First => {
            if !state.list.is_empty() && !state.selected.is_empty() {
                let mut selected = state.selected.clone();
                selected.sort_unstable();
                state.index = selected[0];
                if state.index < state.lines - MARGIN - PADDING {
                    // caret is visible on the screen without any offset
                    state.offset = 0;
                } else if state.index - state.offset > state.lines - MARGIN - PADDING {
                    if state.list.len() - state.index <= PADDING {
                        // caret is beyond the screen and (almost) at the end of the list
                        state.offset =
                            state.index - state.lines + MARGIN + state.list.len() - state.index - 1;
                    } else {
                        // caret is beyond the screen
                        state.offset = state.index - (state.lines - MARGIN - PADDING);
                    }
                }
                print(state)?;
            }
        }
        Move::Top => {
            if !state.list.is_empty() {
                state.index = 0;
                state.offset = 0;
                print(state)?;
            }
        }
        Move::Bottom => {
            if !state.list.is_empty() {
                state.index = state.list.len() - 1;
                if state.index < state.lines + MARGIN + state.list.len() - state.index - 1 {
                    state.offset = 0;
                } else {
                    state.offset =
                        state.index - state.lines + MARGIN + state.list.len() - state.index - 1;
                }
                print(state)?;
            }
        }
    }
    Ok(())
}

fn change_dir(state: &mut State, dir: FolderDir) -> Result<()> {
    match dir {
        FolderDir::Parent => {
            if let Some(parent) = state.path.parent() {
                state.path = parent.to_path_buf();
                state.index = 0;
                state.offset = 0;
                state.selected.clear();
                state.message = None;
                read_dir(state)?;
                print(state)?;
            }
        }
        FolderDir::Child => {
            if !state.list.is_empty() {
                let entry = &state.list[state.index];
                match entry.kind {
                    EntryKind::Dir => {
                        state.path.push(&entry.file_name);
                        state.index = 0;
                        state.offset = 0;
                        state.selected.clear();
                        state.message = None;
                        read_dir(state)?;
                        print(state)?;
                    }
                    EntryKind::File => {
                        if let Some(default) = state.config.default.clone() {
                            let status = Command::new("bash")
                                .args(&["-c", &format!("{} {}", default, &entry.file_name)])
                                .current_dir(&state.path)
                                .status()
                                .unwrap();
                            if !status.success() {
                                state.message = Some(Message::error("Unable to open file!"));
                                print(state)?;
                            }
                        } else {
                            state.message =
                                Some(Message::info("No default app defined to open file"));
                            print(state)?;
                        }
                    }
                }
            }
        }
        FolderDir::Home => {
            if let Some(home) = dirs::home_dir() {
                state.path = home;
                state.index = 0;
                state.offset = 0;
                state.selected.clear();
                state.message = None;
                read_dir(state)?;
                print(state)?;
            }
        }
    }
    Ok(())
}

fn toggle_dotfiles(state: &mut State) -> Result<()> {
    state.show_dotfiles = !state.show_dotfiles;
    state.index = 0;
    state.offset = 0;
    state.selected.clear();
    state.message = Some(Message::info(&format!(
        "Dotfiles are {}",
        if state.show_dotfiles {
            "displayed"
        } else {
            "hidden"
        }
    )));
    read_dir(state)?;
    print(state)?;
    Ok(())
}

fn toggle_select(state: &mut State) -> Result<()> {
    if !state.list.is_empty() {
        let index = state.selected.iter().position(|i| i == &state.index);
        match index {
            Some(index) => {
                state.selected.remove(index);
            }
            None => {
                state.selected.push(state.index);
            }
        }
        set_select_message(state);
        print(state)?;
    }
    Ok(())
}

fn select_all(state: &mut State) -> Result<()> {
    if !state.list.is_empty() {
        state.selected.clear();
        for i in 0..state.list.len() {
            state.selected.push(i);
        }
        set_select_message(state);
        print(state)?;
    }
    Ok(())
}

fn set_select_message(state: &mut State) {
    let length = state.selected.len();
    if length == 0 {
        state.message = None;
        return;
    }
    state.message = Some(Message::info(&format!(
        "{} {} selected",
        length,
        if length == 1 { "item" } else { "items" }
    )));
}

// Reads the current directory
fn read_dir(state: &mut State) -> io::Result<()> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for dir_entry in fs::read_dir(&state.path)? {
        let item = dir_entry?;
        let path = item.path();
        let file_name = item.file_name().into_string().unwrap();
        if !state.show_dotfiles && file_name.starts_with('.') {
            continue;
        }
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

// Prints the current directory entries to the screen
fn print(state: &mut State) -> Result<()> {
    let (height, _) = state.term.size();
    let lines = height as usize - 1;
    state.term.clear_last_lines(state.lines)?;
    for i in 0..lines {
        if i == 1 {
            print_head(state)?;
            continue;
        }
        if i == 3 {
            let line = format!("{}{}", pad!("NAME", WIDTH, WIDTH - 2), pad!("TYPE", 10, 8));
            state.term.write_str(&format!("   {}", line))?;
        }
        if i == 4 {
            let line = "-".repeat(WIDTH + 10);
            state.term.write_str(&format!("   {}", line))?;
        }
        if i > 4 && i < lines - 2 {
            let index = i - 5 + state.offset;
            if state.list.len() > index {
                print_entry(state, index)?;
                continue;
            }
        }
        if i == lines - 1 {
            print_message(state)?;
            continue;
        }
        state.term.write_line("")?;
    }
    state.lines = lines;
    Ok(())
}

fn print_head(state: &mut State) -> Result<()> {
    match state.mode {
        Mode::Normal => {
            let path = state.path.display().to_string();
            state.term.write_line(&format!("   {}", path))?;
        }
        Mode::Prompt => {
            state.term.write_line(&format!(
                "   {}:{}",
                state.title.clone().unwrap_or_default(),
                state.input.clone().unwrap_or_default(),
            ))?;
        }
    }
    Ok(())
}

fn print_entry(state: &mut State, index: usize) -> Result<()> {
    let entry = &state.list[index];
    let caret = if state.mode == Mode::Normal && state.index == index {
        ">"
    } else {
        " "
    };
    let color = match entry.kind {
        EntryKind::Dir => Color::Blue,
        EntryKind::File => entry.to_color(),
    };
    let line = format!(
        "{}{}",
        pad!(&entry.file_name, WIDTH, WIDTH - 2),
        pad!(
            match entry.kind {
                EntryKind::Dir => "dir",
                EntryKind::File => "file",
            },
            10,
            8
        )
    );
    let value = match state.selected.contains(&index) {
        true => color!(&line, Color::Black, color).to_string(),
        false => color!(&line, color).to_string(),
    };
    state.term.write_line(&format!(" {} {}", caret, value))?;
    Ok(())
}

fn print_message(state: &mut State) -> Result<()> {
    let length = state.list.len();
    let digits = length.to_string().len();
    let index = if length == 0 { 0 } else { state.index + 1 };
    match &state.message {
        Some(message) => state.term.write_line(&format!(
            "   {:0>width$}/{}   {}",
            index,
            length,
            message,
            width = digits,
        ))?,
        None => {
            state
                .term
                .write_line(&format!("   {:0>width$}/{}", index, length, width = digits))?
        }
    }
    Ok(())
}
