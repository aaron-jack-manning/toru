use std::fs;
use std::mem;
use std::path;
use std::process;

use crate::tasks;
use crate::error;
use crate::graph;
use crate::state;
use crate::format;
use crate::tasks::Id;

pub fn open_editor(path : &path::Path, editor : &str) -> Result<process::ExitStatus, error::Error> {
    let mut command = process::Command::new(editor);

    command 
        .args(vec![&path]);

    let mut child = command.spawn()?;

    child.wait().map_err(error::Error::from)
}

pub fn edit_info(id : Id, vault_folder : path::PathBuf, editor : &str) -> Result<(), error::Error> {
    let mut task = tasks::Task::load(id, &vault_folder, false)?;

    let temp_path = vault_folder.join("temp.md");

    fs::write(&temp_path, &task.data.info.unwrap_or_default().as_bytes())?;

    let status = open_editor(&temp_path, editor)?;

    if !status.success() {
        match status.code() {
            Some(code) => Err(error::Error::Generic(format!("Process responded with a non-zero status code: {}", code))),
            None => Err(error::Error::Generic(String::from("Process was interrupted by signal"))),
        }
    }
    else {
        let file_contents = fs::read_to_string(&temp_path)?;

        // Check if the remaining file is just whitespace, so the info will become None
        task.data.info = if file_contents.trim().is_empty() {
            None
        }
        else {
            Some(file_contents)
        };
        
        task.save()?;

        // Remove the temporary file
        fs::remove_file(&temp_path)?;

        Ok(())
    }
}

pub fn edit_raw(id : Id, vault_folder : path::PathBuf, editor : &str, state : &mut state::State) -> Result<(), error::Error> {

    let mut task = tasks::Task::load(id, &vault_folder, false)?;

    let temp_path = vault_folder.join("temp.toml");

    fs::copy(&task.path, &temp_path)?;

    let status = open_editor(&temp_path, editor)?;
    
    if !status.success() {
        match status.code() {
            Some(code) => Err(error::Error::Generic(format!("Process responded with a non-zero status code: {}", code))),
            None => Err(error::Error::Generic(String::from("Process was interrupted by signal"))),
        }
    }
    else {
        let mut edited_task = tasks::Task::load_direct(temp_path.clone(), true)?;

        // Enforce time entry duration invariant.
        for entry in &edited_task.data.time_entries {
            if !entry.duration.satisfies_invariant() {
                return Err(error::Error::Generic(String::from("Task duration must not have a number of minutes greater than 60")))
            }
        }

        // Make sure ID is not changed.
        if edited_task.data.id != task.data.id {
            Err(error::Error::Generic(String::from("You cannot change the ID of a task in a direct edit")))
        }
        // Enforce non numeric name invariant.
        else if edited_task.data.name.chars().all(|c| c.is_numeric()) {
            Err(error::Error::Generic(String::from("Name must not be purely numeric")))
        }
        else {
            // Dependencies were edited so the graph needs to be updated.
            if edited_task.data.dependencies != task.data.dependencies {
                for dependency in &task.data.dependencies {
                    state.data.deps.remove_edge(id, *dependency);
                }

                for dependency in &edited_task.data.dependencies {
                    if state.data.deps.contains_node(*dependency) {
                        state.data.deps.insert_edge(id, *dependency)?;
                    }
                    else {
                        return Err(error::Error::Generic(format!("No task with an ID of {} exists", format::id(*dependency))));
                    }
                }

                if let Some(cycle) = state.data.deps.find_cycle() {
                    return Err(error::Error::Generic(format!("Note edit aborted due to circular dependency: {}", graph::format_cycle(&cycle))));
                }
            }
            // Name change means index needs to be updated.
            if edited_task.data.name != task.data.name {
                state.data.index.remove(task.data.name.clone(), id);
                state.data.index.insert(edited_task.data.name.clone(), id);
            }

            mem::swap(&mut edited_task.data, &mut task.data);
            mem::drop(edited_task);

            task.save()?;

            fs::remove_file(&temp_path)?;

            Ok(())
        }
    }
}
