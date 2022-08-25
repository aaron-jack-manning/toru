use crate::error;
use crate::tasks;
use crate::index;
use crate::tasks::Id;

use std::fs;
use std::path;
use std::io;
use std::io::{Write, Seek};


pub struct State {
    file : fs::File,
    pub data : InternalState,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InternalState {
    pub next_id : Id,
    pub index : index::Index,
}

impl State {
    /// This function should be called after creating or checking that the "notes" folder exists.
    pub fn load(vault_location : &path::Path) -> Result<Self, error::Error> {
        let path = vault_location.join("state.toml");

        if path.exists() && path.is_file() {
            // Read file before opening (and truncating).
            let contents = fs::read_to_string(&path)?;

            let file = fs::File::options()
                .write(true)
                .create(true)
                .open(&path)?;

            let data = toml::from_str::<InternalState>(&contents)?;

            Ok(Self {
                file,
                data,
            })
        }
        else {

            // Calculating the next ID if necessary.
            let mut max_id : i128 = -1;
            for id in vault_location.join("notes").read_dir()?.filter_map(|p| p.ok()).map(|p| p.path()).filter(|p| p.extension().map(|s| s.to_str()) == Some(Some("toml"))).filter_map(|p| p.file_stem().map(|x| x.to_str().map(|y| y.to_string()))).flatten().filter_map(|p| p.parse::<Id>().ok()) {

                if i128::try_from(id).unwrap() > max_id {
                    max_id = i128::from(id);
                }
            }

            // Calculating out the index.
            let tasks = tasks::Task::load_all(vault_location, true)?;

            let index = index::Index::create(&tasks);

            let data = InternalState {
                next_id : u64::try_from(max_id + 1).unwrap(),
                index,
            };

            let mut file = fs::File::options()
                .write(true)
                .create(true)
                .open(&path)?;

            let file_contents = toml::to_string(&data)?;

            file.set_len(0)?;
            file.seek(io::SeekFrom::Start(0))?;
            file.write_all(file_contents.as_bytes())?;

            let task = Self {
                file,
                data,
            };

            Ok(task)
        }
    }

    pub fn save(self) -> Result<(), error::Error> {

        let Self {
            mut file,
            data,
        } = self; 

        let file_contents = toml::to_string(&data)?;

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(file_contents.as_bytes())?;

        Ok(())
    }

}
