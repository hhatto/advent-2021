use std::io::stdout;
use std::fs::File;

use clap::{Parser};
use crossterm::{
    cursor::{position, DisableBlinking, MoveTo, MoveUp, MoveDown, MoveLeft, MoveRight, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, ScrollDown, ScrollUp},
    Result,
};

#[derive(Parser)]
#[clap(version = "0.0.1", author = "hhatto")]
struct Opts {
    input: String,
}

struct DisplayLines {
    start: u16,
    end: u16,
}

impl DisplayLines {
    fn start_mut(&mut self) -> &mut u16 {
        &mut self.start
    }
    fn end_mut(&mut self) -> &mut u16 {
        &mut self.end
    }
}

fn less_loop(filename: &str) -> Result<()> {
    let f = File::open(filename)?;
    let lines = ropey::Rope::from_reader(f)?;
    let line_count = lines.len_lines();
    let mut is_search_mode = false;

    let (_, window_rows) = terminal::size()?;
    let mut display_lines = DisplayLines { start: 0, end: 0 };

    for idx in 0..window_rows {
        println!("{}", lines.line(idx as usize));
        execute!(stdout(), MoveTo(0, idx as u16))?;
        if idx as usize >= line_count - 1 {
            break
        }
        *display_lines.end_mut() = idx;
    }
    execute!(stdout(), MoveTo(0, 0))?;

    loop {
        let event = read()?;

        if is_search_mode {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: _,
                }) => {
                    is_search_mode = false;
                    execute!(stdout(), RestorePosition)?;
                },
                _ => (),
            };
        } else {
            let (_, row) = position()?;
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
                        let l = lines.line(display_lines.end.into());
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
                        let l = lines.line(display_lines.end.into());
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
