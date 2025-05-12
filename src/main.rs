use std::io::{Result, Stdout, Write, stdout};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, read},
    style::{self},
    terminal,
};

enum Action {
    MoveUp,
    MoveLeft,
    MoveRight,
    MoveDown,
    Quit,
    SwitchToInsert,
    SwitchToNormal,
}

enum Mode {
    Normal,
    Insert,
}

impl Mode {
    fn handle_event(&self, stdout: &mut Stdout, ev: event::Event) -> Result<Option<Action>> {
        if matches!(self, Mode::Normal) {
            Mode::handle_normal_event(ev)
        } else {
            Mode::handle_insert_event(ev, stdout)
        }
    }

    fn handle_normal_event(ev: event::Event) -> Result<Option<Action>> {
        return match ev {
            crossterm::event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                event::KeyCode::Char('i') => Ok(Some(Action::SwitchToInsert)),
                event::KeyCode::Up | event::KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                event::KeyCode::Down | event::KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                event::KeyCode::Left | event::KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Right | event::KeyCode::Char('l') => Ok(Some(Action::MoveRight)),

                _ => Ok(None),
            },
            _ => Ok(None),
        };
    }
    fn handle_insert_event(ev: event::Event, stdout: &mut Stdout) -> Result<Option<Action>> {
        return match ev {
            crossterm::event::Event::Key(event) => match event.code {
                event::KeyCode::Esc => Ok(Some(Action::SwitchToNormal)),
                event::KeyCode::Char(c) => {
                    stdout.queue(style::Print(c))?;
                    Ok(Some(Action::MoveRight))
                }

                _ => Ok(None),
            },
            _ => Ok(None),
        };
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    // Instiate Terminal States
    let mut mode = Mode::Normal;
    let mut cx = 0;
    let mut cy = 0;

    loop {
        stdout.queue(cursor::MoveTo(cx, cy))?;
        stdout.flush()?;

        if let Some(action) = mode.handle_event(&mut stdout, read()?)? {
            match action {
                Action::Quit => break,
                Action::MoveUp => {
                    cy = cy.saturating_sub(1);
                }
                Action::MoveDown => {
                    cy = cy.saturating_add(1);
                }
                Action::MoveRight => {
                    cx = cx.saturating_add(1);
                }
                Action::MoveLeft => {
                    cx = cx.saturating_sub(1);
                }
                Action::SwitchToInsert => mode = Mode::Insert,
                Action::SwitchToNormal => mode = Mode::Normal,
            };
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;

    terminal::disable_raw_mode()?;

    Ok(())
}
