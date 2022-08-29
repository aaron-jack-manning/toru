use crate::tasks;
use crate::error;
use crate::colour;
use crate::tasks::Id;

use std::fmt::Write;
use std::collections::HashMap;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Index {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    map : HashMap<String, Vec<Id>>
}

impl Index {
    pub fn create(tasks : &Vec<tasks::Task>) -> Index {
        let mut map : HashMap<String, Vec<Id>> = HashMap::with_capacity(tasks.len());
        for task in tasks {
            match map.get_mut(&task.data.name) {
                Some(ids) => {
                    ids.push(task.data.id);
                },
                None => {
                    map.insert(task.data.name.clone(), vec![task.data.id]);
                },
            }
        }

        Self {
            map
        }
    }

    pub fn insert(&mut self, name : String, id : Id) {
        match self.map.get_mut(&name) {
            Some(ids) => {
                ids.push(id);
            },
            None => {
                self.map.insert(name, vec![id]);
            }
        }
    }

    pub fn remove(&mut self, name : String, id : Id) {
        if let Some(mut ids) = self.map.remove(&name) {
            if let Some(index) = ids.iter().position(|i| i == &id) {
                ids.swap_remove(index);

                if !ids.is_empty() {
                    self.map.insert(name, ids);
                }
            }
        }
    }

    pub fn lookup(&self, name_or_id : &String) -> Result<Id, error::Error> {
        match name_or_id.parse::<Id>() {
            Ok(id) => Ok(id),
            Err(_) => {
                let name = name_or_id;
                match self.map.get(name) {
                    Some(ids) => {
                        if ids.len() == 1 {
                            Ok(ids[0])
                        }
                        else {
                            let coloured_ids : Vec<_> =
                                ids.iter()
                                .map(|i| colour::text::id(*i))
                                .collect();

                            let mut display_ids = String::new();

                            for id in coloured_ids {
                                write!(&mut display_ids, "{}, ", id).unwrap();
                            }

                            if !display_ids.is_empty() {
                                display_ids.pop();
                                display_ids.pop();
                            }

                            Err(error::Error::Generic(format!("Multiple notes (Ids: [{}]) by that name exist", display_ids)))
                        }
                    },
                    None => Err(error::Error::Generic(format!("A note by the name {} does not exist", colour::text::task(name)))),
                }
            }
        }
    }
}
