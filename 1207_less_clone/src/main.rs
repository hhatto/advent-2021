use std::io::stdout;
use std::fs::File;

use clap::{Parser};
use crossterm::{
    cursor::{DisableBlinking, MoveTo, MoveUp, MoveDown, MoveLeft, MoveRight, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Result,
};

#[derive(Parser)]
#[clap(version = "0.0.1", author = "hhatto")]
struct Opts {
    input: String,
}

fn less_loop(filename: &str) -> Result<()> {
    let f = File::open(filename)?;
    let lines = ropey::Rope::from_reader(f)?;
    let mut is_search_mode = false;

    for idx in 0..50 {
        println!("{}", lines.line(idx));
        execute!(stdout(), MoveTo(0, idx as u16))?;
    }

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
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: _,
                }) => execute!(stdout(), MoveLeft(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: _,
                }) => execute!(stdout(), MoveDown(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: _,
                }) => execute!(stdout(), MoveUp(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: _,
                }) => execute!(stdout(), MoveRight(1))?,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('/'),
                    modifiers: _,
                }) => {
                    is_search_mode = true;
                    execute!(stdout(), SavePosition, MoveTo(0, 0))?;
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
