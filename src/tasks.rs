use crate::error;
use crate::state;
use crate::colour;

use std::fs;
use std::mem;
use std::path;
use std::io;
use std::io::{Write, Seek};
use std::collections::HashSet;

pub type Id = u64;

pub struct Task {
    path : path::PathBuf,
    file : fs::File,
    pub data : InternalTask,
}

#[derive(Default, Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    #[default]
    Unspecified,
    Low,
    Medium,
    High,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    hours : u32,
    minutes : u8,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InternalTask {
    pub id : Id,
    pub name : String,
    pub info : Option<String>,
    pub tags : HashSet<String>,
    pub dependencies : HashSet<Id>,
    pub priority : Priority,
    //due : Option<chrono::NaiveDateTime>,
    pub time_entries : Vec<TimeEntry>,
    pub created : chrono::NaiveDateTime,
    pub complete : bool,
    pub discarded : bool,
}

impl Task {
    pub fn new(name : String, info : Option<String>, tags : Vec<String>, dependencies : Vec<Id>, priority : Option<Priority>, vault_folder : &path::Path, state : &mut state::State) -> Result<Self, error::Error> {

        let id = state.data.next_id;
        state.data.next_id += 1;
        
        let path = vault_folder.join("notes").join(&format!("{}.toml", id));

        let mut file = fs::File::options()
            .write(true)
            .create(true)
            .open(&path)?;

        let data = InternalTask {
            id,
            name,
            info,
            tags : tags.into_iter().collect(),
            dependencies : dependencies.into_iter().collect(),
            priority : priority.unwrap_or_default(),
            time_entries : Vec::new(),
            created : chrono::Utc::now().naive_local(),
            complete : false,
            discarded : false,
        };


        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(toml::to_string(&data)?.as_bytes())?;

        Ok(Task {
            path,
            file,
            data,
        })
    }

    /// The read_only flag is so that the file will not be truncated, and therefore doesn't need to
    /// be saved when finished.
    pub fn load(id : Id, vault_folder : path::PathBuf, read_only : bool) -> Result<Self, error::Error> {
        let path = Task::check_exists(id, &vault_folder)?;

        let file_contents = fs::read_to_string(&path)?;
        let file = if read_only {
            fs::File::open(&path)?
        }
        else {
            fs::File::options()
                .write(true)
                .create(true)
                .open(&path)?
        };

        let data = toml::from_str(&file_contents)?;

        Ok(Self {
            path,
            file,
            data,
        })
    }

    pub fn check_exists(id : Id, vault_folder : &path::Path) -> Result<path::PathBuf, error::Error> {
        let path = vault_folder.join("notes").join(format!("{}.toml", id));
        if path.exists() && path.is_file() {
            Ok(path)
        }
        else {
            Err(error::Error::Generic(format!("No task with the ID {} exists", colour::id(&id.to_string()))))
        }
    }

    pub fn save(self) -> Result<(), error::Error> {
        let Self {
            path,
            mut file,
            data,
        } = self;

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(toml::to_string(&data)?.as_bytes())?;

        Ok(())
    }

    pub fn delete(self) -> Result<(), error::Error> {
        let Self {
            path,
            file,
            data,
        } = self;

        mem::drop(file);
        fs::remove_file(&path)?;

        Ok(())
    }

    pub fn delete_by_id(id : Id, vault_folder : &path::Path) -> Result<(), error::Error> {
        let path = Task::check_exists(id, vault_folder)?;
        fs::remove_file(&path)?;
        Ok(())
    }
}


