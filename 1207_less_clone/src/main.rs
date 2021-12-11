use std::io::stdout;
use std::fs::File;

use clap::Parser;
use crossterm::{
    cursor::{position, DisableBlinking, MoveTo, MoveUp, MoveDown, MoveLeft, MoveRight, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, ScrollDown, ScrollUp},
    Result,
};
use grep::searcher::SearcherBuilder;
use grep::searcher::sinks::UTF8;
use grep::regex::RegexMatcher;

#[derive(Parser)]
#[clap(version = "0.0.1", author = "hhatto")]
struct Opts {
    input: String,
}

struct DisplayLines {
    start: u64,
    end: u64,
}

impl DisplayLines {
    fn start_mut(&mut self) -> &mut u64 {
        &mut self.start
    }
    fn end_mut(&mut self) -> &mut u64 {
        &mut self.end
    }
}

fn search(filename: &str, search_word: &str) -> Result<Vec<(u64, String)>> {
    let matcher = RegexMatcher::new(search_word).unwrap();
    let mut matches: Vec<(u64, String)> = vec![];
    let mut searcher = SearcherBuilder::new().build();
    searcher.search_path(&matcher, filename, UTF8(|lnum, line| {
        matches.push((lnum, line.to_string()));
        Ok(true)
    }))?;
    Ok(matches)
}

fn less_loop(filename: &str) -> Result<()> {
    let f = File::open(filename)?;
    let lines = ropey::Rope::from_reader(f)?;
    let line_count = lines.len_lines();
    let mut is_search_mode = false;

    let mut search_word_vec: Vec<char> = [].to_vec();
    let (_, window_rows) = terminal::size()?;
    let mut display_lines = DisplayLines { start: 0, end: 0 };

    for idx in 0..window_rows {
        println!("{}", lines.line(idx as usize));
        execute!(stdout(), MoveTo(0, idx as u16))?;
        if idx as usize >= line_count - 1 {
            break
        }
        *display_lines.end_mut() = idx as u64;
    }
    execute!(stdout(), MoveTo(0, 0))?;

    loop {
        let event = read()?;
        let (_, row) = position()?;

        if is_search_mode {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: _,
                }) => {
                    is_search_mode = false;
                    execute!(stdout(), RestorePosition)?;
                    search_word_vec = Vec::new();
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: _,
                }) => {
                    let search_word = String::from_iter(search_word_vec.clone());
                    let result = search(filename, search_word.as_str())?;
                    if result.len() > 0 {
                        // jump to result line
                        let (lnum, _) = result[0];
                        execute!(stdout(), RestorePosition, SavePosition, Clear(ClearType::All))?;

                        for idx in 0..window_rows {
                            println!("{}", lines.line(lnum as usize + idx as usize));
                            execute!(stdout(), MoveTo(0, idx))?;
                            if idx as usize >= line_count - 1 {
                                break
                            }
                            *display_lines.end_mut() = idx as u64;
                        }
                        execute!(stdout(), RestorePosition)?;
                    }

                    is_search_mode = false;
                    execute!(stdout(), RestorePosition)?;
                    search_word_vec = Vec::new();
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _,
                }) => {
                    search_word_vec.push(c);
                    execute!(stdout(), Print(c))?;
                },
                _ => (),
            };
        } else {
            execute!(stdout(), SavePosition)?;

            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: _,
                }) => execute!(stdout(), MoveLeft(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: _,
                }) => {
                    if window_rows-3 == row {
                        *display_lines.start_mut() = display_lines.start + 1;
                        *display_lines.end_mut() = display_lines.end + 1;
                        let l = lines.line(display_lines.end as usize);
                        execute!(stdout(), ScrollUp(1), Print(l), RestorePosition)?;
                    } else {
                        execute!(stdout(), MoveDown(1))?;
                    }
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: _,
                }) => {
                    if 0 == row {
                        *display_lines.start_mut() = display_lines.start - 1;
                        *display_lines.end_mut() = display_lines.end - 1;
                        let l = lines.line(display_lines.end as usize);
                        execute!(stdout(), ScrollDown(1), Print(l), RestorePosition)?;
                    } else {
                        execute!(stdout(), MoveUp(1))?;
                    }
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: _,
                }) => execute!(stdout(), MoveRight(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                }) => execute!(stdout(), MoveUp(20))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                }) => execute!(stdout(), MoveDown(20))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('/'),
                    modifiers: _,
                }) => {
                    is_search_mode = true;
                    execute!(
                        stdout(),
                        MoveTo(0, window_rows+1),
                        Print("/"),
                    )?;
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: _,
                }) => break,
                _ => (),
            };
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut stdout = stdout();

    enable_raw_mode()?;

    execute!(stdout, Clear(ClearType::All))?;

    execute!(
        stdout,
        SavePosition,
        MoveTo(0, 0),
        DisableBlinking,
    )?;

    if let Err(e) = less_loop(opts.input.as_str()) {
        println!("error={:?}\r", e);
    }

    disable_raw_mode()
}
