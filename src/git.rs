use crate::error;

use std::path;
use std::process;

pub fn run_command(args : Vec<String>, vault_folder : &path::Path) -> Result<(), error::Error> {

    let mut command = process::Command::new("git");

    let mut child = command 
        .current_dir(vault_folder)
        // Force colour output even though run from other process.
        .args(["-c", "color.ui=always"])
        .args(args)
        .spawn()?;

    // No point handling the potential error code as Git will report the error directly with more
    // info.
    let _ = child.wait()?;

    Ok(())
}
