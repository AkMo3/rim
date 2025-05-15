use std::{
    io::{Stdout, Write, stdout},
    u16,
};

use crate::enums::Action;
use crate::mode::Mode;

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::read,
    style::{self, Color, Stylize},
    terminal,
};

struct Cursor {
    cx: u16,
    cy: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { cx: 0, cy: 0 }
    }

    pub fn mode_right(&mut self) {
        self.cx = self.cx.saturating_add(1);
    }

    pub fn mode_left(&mut self) {
        self.cx = self.cx.saturating_sub(1);
    }

    pub fn mode_down(&mut self) {
        self.cy = self.cy.saturating_add(1);
    }

    pub fn mode_up(&mut self) {
        self.cy = self.cy.saturating_sub(1);
    }
}

pub struct Editor {
    mode: Mode,
    size: (u16, u16),
    command: String,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            mode: Mode::Normal,
            size: (0, 0),
            command: String::new(),
        }
    }

    fn draw(&self, stdout: &mut Stdout, cursor: &mut Cursor) -> anyhow::Result<()> {
        stdout.queue(cursor::MoveTo(cursor.cx, cursor.cy))?;
        stdout.flush()?;

        Ok(())
    }

    fn draw_status_line(&self, stdout: &mut Stdout, cursor: &mut Cursor) -> anyhow::Result<()> {
        stdout.queue(cursor::MoveTo(0, cursor.cy))?;

        // Define a struct for our style
        struct StatusStyle<'a> {
            content: &'a str,
            background: Color,
        }

        // Create the appropriate style based on mode
        let style = if self.mode == Mode::Normal {
            StatusStyle {
                content: " NORMAL ",
                background: Color::Blue,
            }
        } else if self.mode == Mode::Insert {
            StatusStyle {
                content: " INSERT ",
                background: Color::DarkYellow,
            }
        } else {
            StatusStyle {
                content: " COMMAND ",
                background: Color::Magenta,
            }
        };

        let _ = stdout.queue(style::PrintStyledContent(
            style
                .content
                .with(Color::Black)
                .on(style.background)
                .attribute(style::Attribute::Bold),
        ));

        let _ = stdout.queue(style::PrintStyledContent("".with(style.background)));

        if self.mode == Mode::Command {
            let _ = stdout.queue(style::Print(format!(":{}", self.command)));
            stdout.flush()?;
        }

        Ok(())
    }

    pub fn clear_line(&self, stdout: &mut Stdout, line_number: u16) -> anyhow::Result<()> {
        stdout
            .queue(cursor::MoveTo(0, line_number))?
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::BeginSynchronizedUpdate)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let mut text_cursor = Cursor::new();
        let mut command_cursor = &mut Cursor::new();

        loop {
            let current_size = terminal::size()?;
            if current_size != self.size {
                if self.size.1 > 2 {
                    self.clear_line(&mut stdout, self.size.1 - 2)?;
                }
                self.size = current_size;
                command_cursor.cx = 9;
                command_cursor.cy = self.size.1 - 2
            }

            self.draw_status_line(&mut stdout, command_cursor)?;
            if self.mode != Mode::Command {
                self.draw(&mut stdout, &mut text_cursor)?;
            }

            let current_cursor = if self.mode == Mode::Command {
                &mut command_cursor
            } else {
                &mut text_cursor
            };

            if let Some(action) = self
                .mode
                .handle_event(&mut stdout, read()?, &mut self.command)?
            {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        current_cursor.mode_up();
                    }
                    Action::MoveDown => {
                        current_cursor.mode_down();
                    }
                    Action::MoveRight => {
                        current_cursor.mode_right();
                    }
                    Action::MoveLeft => {
                        current_cursor.mode_left();
                    }
                    Action::SwitchToInsert => {
                        self.mode = Mode::Insert;
                    }
                    Action::SwitchToNormal => {
                        if self.mode == Mode::Command {
                            self.command.clear();
                            let _ = self.clear_line(&mut stdout, command_cursor.cy);
                        }

                        stdout.execute(cursor::MoveTo(text_cursor.cx, text_cursor.cy))?;
                        self.mode = Mode::Normal;
                    }
                    Action::Command => {
                        self.mode = Mode::Command;
                        stdout.execute(cursor::MoveTo(command_cursor.cx, command_cursor.cy))?;
                    }
                };

                if self.mode == Mode::Command {
                    let _ = self.clear_line(&mut stdout, command_cursor.cy);
                }
            }
        }

        stdout
            .execute(terminal::EndSynchronizedUpdate)?
            .execute(terminal::LeaveAlternateScreen)?;

        terminal::disable_raw_mode()?;
        Ok(())
    }
}
