use console::Color;
use console::Key;
use console::Term;
use fx::color;
use fx::pad;
use fx::Config;
use fx::Context;
use fx::Entry;
use fx::EntryKind;
use fx::Error;
use fx::Mode;
use fx::Move;
use fx::Result;
use fx::State;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> Result<()> {
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
        } // Err(_) => {
          //     term.write_line(color!(
          //         &format!("Invalid arguments! '{}' is not a valid path!", &current_dir),
          //         Color::Red
          //     ))
          //     .unwrap();
          //     process::exit(1);
          // }
    };
    Ok(State::new(config, path))
}

// Initializes the user interface
fn init_ui(mut state: State) -> Result<()> {
    let term = Term::stdout();
    let mut ctx = (&term, &mut state);
    term.hide_cursor()?;
    term.clear_screen()?;
    read_dir(&mut ctx)?;
    print(&mut ctx)?;
    update_loop(&mut ctx)?;
    term.clear_last_lines(state.lines)?;
    term.show_cursor()?;
    Ok(())
}

fn update_loop(ctx: &mut Context) -> Result<()> {
    loop {
        let key = ctx.0.read_key()?;
        match key {
            Key::Char('q') => break,
            Key::Char('j') => move_caret(ctx, Move::Down)?,
            Key::Char('k') => move_caret(ctx, Move::Up)?,
            Key::Char('h') => change_dir(ctx, Move::Down)?,
            Key::Char('l') => change_dir(ctx, Move::Up)?,
            Key::Char(' ') => toggle_dotfiles(ctx)?,
            _ => (),
        }
    }
    Ok(())
}

fn move_caret(ctx: &mut Context, movement: Move) -> Result<()> {
    let (_, state) = ctx;
    match movement {
        Move::Down => {
            if state.list.len() > 0 && state.index < state.list.len() - 1 {
                state.index += 1;
                if state.index >= state.lines + state.offset - 5 - state.config.padding {
                    if state.list.len() - state.index > state.config.padding {
                        state.offset += 1;
                    }
                }
                print(ctx)?;
            }
        }
        Move::Up => {
            if state.index > 0 {
                state.index -= 1;
                if state.offset > 0 && state.index - state.offset < state.config.padding {
                    state.offset -= 1;
                }
                print(ctx)?;
            }
        }
    }
    Ok(())
}

fn change_dir(ctx: &mut Context, movement: Move) -> Result<()> {
    let (_, state) = ctx;
    match movement {
        Move::Down => {
            if let Some(parent) = state.path.parent() {
                state.path = parent.to_path_buf();
                state.index = 0;
                state.offset = 0;
                state.selected.clear();
                state.message = None;
                read_dir(ctx)?;
                print(ctx)?;
            }
        }
        Move::Up => {
            if state.list.len() > 0 {
                let entry = &state.list[state.index];
                if entry.kind == EntryKind::Dir {
                    state.path.push(&entry.file_name);
                    state.index = 0;
                    state.offset = 0;
                    state.selected.clear();
                    state.message = None;
                    read_dir(ctx)?;
                    print(ctx)?;
                }
            }
        }
    }
    Ok(())
}

fn toggle_dotfiles(ctx: &mut Context) -> Result<()> {
    ctx.1.show_dotfiles = !ctx.1.show_dotfiles;
    ctx.1.index = 0;
    ctx.1.offset = 0;
    ctx.1.selected.clear();
    ctx.1.message = None;
    read_dir(ctx)?;
    print(ctx)?;
    Ok(())
}

// Reads the current directory
fn read_dir(ctx: &mut Context) -> io::Result<()> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for dir_entry in fs::read_dir(&ctx.1.path)? {
        let item = dir_entry?;
        let path = item.path();
        let file_name = item.file_name().into_string().unwrap();
        if !ctx.1.show_dotfiles && file_name.starts_with(".") {
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
    ctx.1.list = list;
    Ok(())
}

// Prints the current directory entries to the screen
fn print(ctx: &mut Context) -> Result<()> {
    let (term, state) = ctx;
    let (height, _) = term.size();
    let lines = height as usize - 1;
    let path = state.path.display().to_string();
    term.clear_last_lines(state.lines)?;
    for i in 0..lines {
        if i == 1 {
            match state.mode {
                Mode::Normal => {
                    term.write_line(&format!("   {}", path))?;
                }
                Mode::Search => {
                    term.write_line(&format!("   search:{}", state.search.clone().unwrap()))?;
                }
            }
            continue;
        }
        // if i == 3 && state.offset > 0 {
        //     term.write_line("   ...")?;
        //     continue;
        // }
        // if i == lines - 3 {
        //     let index = i - 3 + state.offset;
        //     if state.list.len() > index {
        //         term.write_line("   ...")?;
        //         continue;
        //     }
        // }
        if i > 2 && i < lines - 2 {
            let index = i - 3 + state.offset;
            if state.list.len() > index {
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
                    pad!(&entry.file_name, 40),
                    pad!(
                        match entry.kind {
                            EntryKind::Dir => "dir",
                            EntryKind::File => "file",
                        },
                        10
                    )
                );
                let value = match state.selected.contains(&index) {
                    true => color!(&line, Color::Black, color).to_string(),
                    false => color!(&line, color).to_string(),
                };
                term.write_line(&format!(" {} {}", caret, value))?;
                continue;
            }
        }
        if i == lines - 1 {
            let length = state.list.len();
            let digits = length.to_string().len();
            match &state.message {
                Some(message) => term.write_line(&format!(
                    "   {:0>width$}/{}   {}",
                    state.index + 1,
                    length,
                    &message,
                    width = digits,
                ))?,
                None => term.write_line(&format!(
                    "   {:0>width$}/{}",
                    state.index + 1,
                    length,
                    width = digits
                ))?,
            }
            continue;
        }
        term.write_line("")?;
    }
    ctx.1.lines = lines;
    Ok(())
}

// fn _init_ui() -> io::Result<()> {
//     let mut prev_key = None;
//     loop {
//         let mut key = Some(term.read_key()?);
//         match state.mode {
//             Mode::Normal => match key.clone().unwrap() {
//                 Key::Escape => {
//                     state.selected.clear();
//                     state.message = None;
//                     print(&term, &mut state)?;
//                 }
//                 Key::Char('/') => {
//                     state.mode = Mode::Search;
//                     state.search = Some(String::new());
//                     term.hide_cursor()?;
//                     print(&term, &mut state)?;
//                     term.show_cursor()?;
//                     term.move_cursor_to(10, 1)?;
//                 }
//                 Key::Char('g') => {
//                     if let Some(prev_key) = &prev_key {
//                         match prev_key {
//                             Key::Char('g') => {
//                                 if state.list.len() > 0 {
//                                     state.index = 0;
//                                     state.offset = 0;
//                                     read_dir(&mut state)?;
//                                     print(&term, &mut state)?;
//                                     key = None;
//                                 }
//                             }
//                             _ => (),
//                         }
//                     }
//                 }
//                 Key::Char('e') => {
//                     if let Some(prev_key) = &prev_key {
//                         match prev_key {
//                             Key::Char('g') => {
//                                 if state.list.len() > 0 {
//                                     state.index = state.list.len() - 1;
//                                     if state.index >= state.lines - 6 - PADDING {
//                                         state.offset = state.index - state.lines + 6;
//                                     }
//                                     read_dir(&mut state)?;
//                                     print(&term, &mut state)?;
//                                     key = None;
//                                 }
//                             }
//                             _ => (),
//                         }
//                     }
//                 }
//                 Key::Char('x') => {
//                     if state.list.len() > 0 {
//                         let index = state.selected.iter().position(|i| i == &state.index);
//                         match index {
//                             Some(index) => {
//                                 state.selected.remove(index);
//                             }
//                             None => {
//                                 state.selected.push(state.index);
//                             }
//                         }
//                         match state.selected.len() {
//                             0 => state.message = None,
//                             _ => {
//                                 let word = if state.selected.len() == 1 {
//                                     "item"
//                                 } else {
//                                     "items"
//                                 };
//                                 state.message =
//                                     Some(format!("{} {} selected", state.selected.len(), word))
//                             }
//                         }
//                         print(&term, &mut state)?;
//                     }
//                 }
//                 Key::Char('n') => {
//                     if state.list.len() > 0 && state.selected.len() > 0 {
//                         let mut next = state.selected[0];
//                         for index in state.selected.clone() {
//                             if state.index < index {
//                                 next = index;
//                                 break;
//                             }
//                         }
//                         state.index = next;
//                         if state.index < state.lines - 6 - PADDING {
//                             state.offset = 0;
//                         } else if state.index - state.offset > state.lines - 6 - PADDING {
//                             if state.list.len() - state.index <= PADDING {
//                                 state.offset = state.index - state.lines + 6 + state.list.len()
//                                     - state.index
//                                     - 1;
//                             } else {
//                                 state.offset = state.index - (state.lines - 6 - PADDING);
//                             }
//                         }
//                         print(&term, &mut state)?;
//                     }
//                 }
//                 // TODO: set previous selected correctly
//                 // Key::Char('N') => {
//                 //     if state.list.len() > 0 && state.selected.len() > 0 {
//                 //         let mut previous = state.selected[state.selected.len() - 1];
//                 //         let mut last = previous;
//                 //         for index in state.selected.clone() {
//                 //             if state.index > last {
//                 //                 previous = index;
//                 //                 break;
//                 //             }
//                 //             last = index;
//                 //         }
//                 //         state.index = previous;
//                 //         print(&term, &mut state)?;
//                 //     }
//                 // }
//                 _ => (),
//             },
//             Mode::Search => match key.clone().unwrap() {
//                 Key::Escape => {
//                     state.mode = Mode::Normal;
//                     state.search = None;
//                     term.hide_cursor()?;
//                     print(&term, &mut state)?;
//                 }
//                 Key::Enter => {
//                     if let Some(search) = state.search.clone() {
//                         match Regex::new(&search) {
//                             Ok(regex) => {
//                                 state.selected.clear();
//                                 for (i, entry) in state.list.iter().enumerate() {
//                                     if regex.is_match(&entry.file_name) {
//                                         state.selected.push(i);
//                                     }
//                                 }
//                                 match state.selected.len() {
//                                     0 => state.message = None,
//                                     _ => {
//                                         state.index = state.selected[0];
//                                         if state.index < state.lines + 6 + PADDING {
//                                             state.offset = 0;
//                                         } else {
//                                             if state.list.len() - state.index <= PADDING {
//                                                 state.offset = state.index - state.lines
//                                                     + 6
//                                                     + state.list.len()
//                                                     - state.index
//                                                     - 1;
//                                             } else {
//                                                 state.offset =
//                                                     state.index - state.lines + 6 + PADDING;
//                                             }
//                                         }
//                                         let word = if state.selected.len() == 1 {
//                                             "item"
//                                         } else {
//                                             "items"
//                                         };
//                                         state.message = Some(format!(
//                                             "{} {} selected",
//                                             state.selected.len(),
//                                             word,
//                                         ))
//                                     }
//                                 }
//                             }
//                             Err(_) => (),
//                         };
//                         state.mode = Mode::Normal;
//                         state.search = None;
//                         term.hide_cursor()?;
//                         print(&term, &mut state)?;
//                     }
//                 }
//                 Key::Char(char) => {
//                     if let Some(mut search) = state.search.clone() {
//                         search.push(char);
//                         let length = search.len();
//                         state.search = Some(search);
//                         term.hide_cursor()?;
//                         print(&term, &mut state)?;
//                         term.show_cursor()?;
//                         term.move_cursor_to(10 + length, 1)?;
//                     }
//                 }
//                 Key::Backspace => {
//                     if let Some(mut search) = state.search.clone() {
//                         search.pop();
//                         let length = search.len();
//                         state.search = Some(search);
//                         term.hide_cursor()?;
//                         print(&term, &mut state)?;
//                         term.show_cursor()?;
//                         term.move_cursor_to(10 + length, 1)?;
//                     }
//                 }
//                 _ => (),
//             },
//         }
//         prev_key = key;
//     }
//     term.clear_last_lines(state.lines)?;
//     term.show_cursor()?;
//     Ok(())
// }
