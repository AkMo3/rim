use crossterm::QueueableCommand;
use crossterm::event;
use crossterm::style;

use crate::enums::Action;

use std::io::Result;
use std::io::Stdout;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Mode {
    pub fn handle_event(
        &self,
        stdout: &mut Stdout,
        ev: event::Event,
        command: &mut String,
    ) -> Result<Option<Action>> {
        if matches!(self, Mode::Normal) {
            Mode::handle_normal_event(ev)
        } else if matches!(self, Mode::Insert) {
            Mode::handle_insert_event(ev, stdout)
        } else {
            Mode::handle_command_mode(ev, command)
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
                event::KeyCode::Char(':') => Ok(Some(Action::Command)),
                event::KeyCode::Esc => Ok(Some(Action::SwitchToNormal)),

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

    fn handle_command_mode(ev: event::Event, command: &mut String) -> Result<Option<Action>> {
        return match ev {
            crossterm::event::Event::Key(event) => match event.code {
                event::KeyCode::Esc => Ok(Some(Action::SwitchToNormal)),
                event::KeyCode::Backspace => {
                    let _ = command.pop();
                    Ok(Some(Action::MoveLeft))
                }
                event::KeyCode::Delete => {
                    let _ = command.pop();
                    Ok(Some(Action::MoveLeft))
                }
                event::KeyCode::Char(c) => {
                    command.push(c);
                    Ok(Some(Action::MoveRight))
                }

                _ => Ok(None),
            },
            _ => Ok(None),
        };
    }
}
