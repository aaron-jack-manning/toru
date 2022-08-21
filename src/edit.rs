use std::fs;
use std::mem;
use std::path;
use std::process;

use crate::tasks;
use crate::error;
use crate::tasks::Id;

pub fn edit_raw(id : Id, vault_folder : path::PathBuf) -> Result<(), error::Error> {

    let mut task = tasks::Task::load(id, vault_folder.clone(), false)?;

    let temp_path = vault_folder.join("temp.toml");

    fs::copy(task.path(), &temp_path)?;

    // This will be a matter of configuration later on.
    let mut command = process::Command::new("nvim");

    command 
        .current_dir(&vault_folder)
        .args(vec![&temp_path]);

    let mut child = command.spawn()?;

    let status = child.wait()?;

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
