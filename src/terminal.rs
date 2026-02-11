use std::io;
use std::io::{stdout, Stdout, Write};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use crossterm::cursor::SetCursorStyle;
use crossterm::event::EnableMouseCapture;

pub struct Terminal {
    pub out: Stdout,
}

pub struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

impl Terminal {
    pub fn enter() -> std::io::Result<(TermGuard, Terminal)> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnableMouseCapture, SetCursorStyle::SteadyBlock)?;
        out.flush()?;
        Ok((TermGuard, Terminal { out }))
    }

    pub fn term_size(&self) -> io::Result<(u16, u16)>  {
        size()
    }
}
