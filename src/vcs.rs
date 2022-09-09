use crate::error;

use std::fs;
use std::path;
use std::process;

pub enum Vcs {
    Git,
    Svn,
}

pub fn command(args : Vec<String>, vcs : Vcs, vault_folder : &path::Path) -> Result<(), error::Error> {

    let mut command = match vcs {
        Vcs::Git => process::Command::new("git"),
        Vcs::Svn => process::Command::new("svn"),
    };

    let mut child = command 
        .current_dir(vault_folder)
        .args(args)
        .spawn()?;

    let _ = child.wait()?;

    Ok(())
}

pub fn create_gitignore(vault_folder : &path::Path) -> Result<(), error::Error> {
    Ok(fs::write(vault_folder.join(".gitignore"), "temp.toml\ntemp.md")?)
}

pub fn set_svn_ignore(vault_folder : &path::Path) -> Result<(), error::Error> {

    let mut child = process::Command::new("svn")
        .current_dir(vault_folder)
        .args(&["propset", "svn:ignore", "temp.toml\ntemp.md", "."])
        .spawn()?;

    let _ = child.wait()?;

    Ok(())
}
