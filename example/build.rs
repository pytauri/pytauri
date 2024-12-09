use anyhow::Result;

fn main() -> Result<()> {
    tauri_build::build();

    Ok(())
}
