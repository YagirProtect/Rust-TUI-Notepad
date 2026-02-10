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
        execute!(out, EnterAlternateScreen, EnableMouseCapture, Clear(ClearType::All), Hide)?;
        out.flush()?;
        Ok((TermGuard, Terminal { out }))
    }

    /// Блокирующее ожидание 1 события
    pub fn next_event(&mut self) -> io::Result<Event> {
        loop {
            let ev = event::read()?;
            // фильтр: берем только key press (чтобы не ловить repeat/release)
            if let Event::Key(k) = &ev {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
            }
            return Ok(ev);
        }
    }

    pub fn term_size(&self) -> io::Result<(u16, u16)>  {
        size()
    }

    pub fn set_cursor(&mut self, x: u16, y: u16, visible: bool) -> io::Result<()> {
        if visible {
            execute!(self.out, Show, MoveTo(x, y))?;
        } else {
            execute!(self.out, Hide)?;
        }
        self.out.flush().ok();
        Ok(())
    }

    /// Пока без твоего ScreenBuf: просто очистим экран
    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self.out, Clear(ClearType::All))?;
        execute!(self.out, MoveTo(0, 0))?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush().ok();
        Ok(())
    }
}
