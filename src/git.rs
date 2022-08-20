use crate::error;

use std::io;
use std::str;
use std::path;
use std::process;
use std::io::Write;

pub fn run_command(args : Vec<String>, vault_folder : &path::Path) -> Result<(), error::Error> {

    let mut command = process::Command::new("git");

    command
        .current_dir(vault_folder)
        // Force colour output even though run from other process.
        .args(["-c", "color.ui=always"])
        .args(args);

    let output = command.output()?;
    let output_string = str::from_utf8(&output.stdout)?;

    print!("{}", output_string);
    io::stdout().flush()?;

    Ok(())
}
