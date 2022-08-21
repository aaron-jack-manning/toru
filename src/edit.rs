use std::fs;
use std::mem;
use std::path;
use std::process;

use crate::tasks;
use crate::error;
use crate::tasks::Id;

pub fn open_editor(path : &path::Path, editor : &str) -> Result<process::ExitStatus, error::Error> {
    let mut command = process::Command::new(editor);

    command 
        .args(vec![&path]);

    let mut child = command.spawn()?;

    child.wait().map_err(|err| error::Error::from(err))
}

pub fn edit_info(id : Id, vault_folder : path::PathBuf, editor : &str) -> Result<(), error::Error> {
    let mut task = tasks::Task::load(id, vault_folder.clone(), false)?;

    let temp_path = vault_folder.join("temp.md");

    fs::write(&temp_path, &task.data.info.unwrap_or(String::new()).as_bytes())?;

    let status = open_editor(&temp_path, editor)?;

    if !status.success() {
        match status.code() {
            Some(code) => Err(error::Error::Generic(format!("Process responded with a non-zero status code: {}", code))),
            None => Err(error::Error::Generic(String::from("Process was interrupted by signal"))),
        }
    }
    else {
        let file_contents = fs::read_to_string(&temp_path)?;

        task.data.info = if file_contents.is_empty() {
            None
        }
        else {
            Some(file_contents)
        };
        
        task.save()?;

        Ok(())
    }
}

pub fn edit_raw(id : Id, vault_folder : path::PathBuf, editor : &str) -> Result<(), error::Error> {

    let mut task = tasks::Task::load(id, vault_folder.clone(), false)?;

    let temp_path = vault_folder.join("temp.toml");

    fs::copy(task.path(), &temp_path)?;

    let status = open_editor(&temp_path, editor)?;
    
    if !status.success() {
        match status.code() {
            Some(code) => Err(error::Error::Generic(format!("Process responded with a non-zero status code: {}", code))),
            None => Err(error::Error::Generic(String::from("Process was interrupted by signal"))),
        }
    }
    else {
        let file_contents = fs::read_to_string(&temp_path)?;

        let mut edited_task = tasks::Task::load_direct(temp_path.clone(), true)?;

        if edited_task.data.id != task.data.id {
            Err(error::Error::Generic(String::from("You cannot change the ID of a task in a direct edit")))
        }
        else {
            if edited_task.data.dependencies != task.data.dependencies {
                // This is where the other dependencies graph needs to be updated.
            }
            if edited_task.data.name != task.data.name {
                // This is where the hashmap from id to string needs to be updated.
            }

            mem::swap(&mut edited_task.data, &mut task.data);
            mem::drop(edited_task);

            task.save()?;

            fs::remove_file(&temp_path)?;

            Ok(())
        }
    }
}
