use crate::error;
use crate::tasks;
use crate::colour;
use crate::tasks::Id;

use std::fs;
use std::path;
use std::io;
use std::io::{Write, Seek};
use std::collections::HashMap;

use serde_with::{serde_as, DisplayFromStr};

pub struct State {
    file : fs::File,
    pub data : InternalState,
}

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct InternalState {
    pub next_id : Id,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub index : HashMap<String, Vec<Id>>,
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
            let mut index : HashMap<String, Vec<Id>> = HashMap::with_capacity(tasks.len());
            for task in tasks {
                match index.get_mut(&task.data.name) {
                    Some(ids) => {
                        ids.push(task.data.id);
                    },
                    None => {
                        index.insert(task.data.name.clone(), vec![task.data.id]);
                    },
                }
            }

            let data = InternalState {
                next_id : u64::try_from(max_id + 1).unwrap(),
                index,
            };

            let mut file = fs::File::options()
                .write(true)
                .create(true)
                .open(&path)?;

            file.set_len(0)?;
            file.seek(io::SeekFrom::Start(0))?;
            file.write_all(toml::to_string(&data)?.as_bytes())?;

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

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(toml::to_string(&data)?.as_bytes())?;

        Ok(())
    }

    pub fn index_insert(&mut self, name : String, id : Id) {
        match self.data.index.get_mut(&name) {
            Some(ids) => {
                ids.push(id);
            },
            None => {
                self.data.index.insert(name, vec![id]);
            }
        }
    }

    pub fn index_remove(&mut self, name : String, id : Id) {
        if let Some(mut ids) = self.data.index.remove(&name) {
            if let Some(index) = ids.iter().position(|i| i == &id) {
                ids.swap_remove(index);

                if !ids.is_empty() {
                    self.data.index.insert(name, ids);
                }
            }
        }
    }

    pub fn name_or_id_to_id(&self, name : &String) -> Result<Id, error::Error> {
        match name.parse::<Id>() {
            Ok(id) => Ok(id),
            Err(_) => {
                match self.data.index.get(name) {
                    Some(ids) => {
                        if ids.len() == 1 {
                            Ok(ids[0])
                        }
                        else {
                            let coloured_ids : Vec<_> =
                                ids.into_iter()
                                .map(|i| colour::id(&i.to_string()))
                                .collect();

                            let mut display_ids = String::new();

                            for id in coloured_ids {
                                display_ids.push_str(&format!("{}, ", id));
                            }

                            if !display_ids.is_empty() {
                                display_ids.pop();
                                display_ids.pop();
                            }

                            Err(error::Error::Generic(format!("Multiple notes (Ids: [{}]) by that name exist", display_ids)))
                        }
                    },
                    None => Err(error::Error::Generic(format!("A note by the name {} does not exist", colour::task_name(&name)))),
                }
            }
        }
    }
}
