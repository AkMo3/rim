pub mod editor;
pub mod enums;
pub mod mode;

use editor::Editor;

fn main() -> anyhow::Result<()> {
    let mut editor = Editor::new();
    editor.run()?;

    Ok(())
}
