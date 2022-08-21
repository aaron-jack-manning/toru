use crate::error;
use crate::state;
use crate::colour;

use std::io;
use std::fs;
use std::fmt;
use std::mem;
use std::path;
use std::io::{Write, Seek};
use std::collections::HashSet;
use colored::Colorize;

pub type Id = u64;

pub struct Task {
    path : path::PathBuf,
    file : fs::File,
    pub data : InternalTask,
}

#[derive(Default, Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        use Priority::*;
        let priority = match self {
            Low => "low",
            Medium => "medium",
            High => "high",
        };
        write!(f, "{}", priority)
    }
}

impl Priority {
    pub fn coloured(&self) -> String {
        use Priority::*;
        let priority = match self {
            Low => "low".truecolor(46, 204, 113),
            Medium => "medium".truecolor(241, 196, 15),
            High => "high".truecolor(231, 76, 60),
        };
        format!("{}", priority)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    hours : u32,
    minutes : u8,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

    /// Less graceful error handling on this for task not existing. Only use this externally when
    /// in edit mode.
    pub fn load_direct(path : path::PathBuf, read_only : bool) -> Result<Self, error::Error> {
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

    /// The read_only flag is so that the file will not be truncated, and therefore doesn't need to
    /// be saved when finished.
    pub fn load(id : Id, vault_folder : path::PathBuf, read_only : bool) -> Result<Self, error::Error> {
        let path = Task::check_exists(id, &vault_folder)?;

        Task::load_direct(path, read_only)
    }

    pub fn path(&self) -> &path::Path {
        &self.path
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

    pub fn display(&self) -> Result<(), error::Error> {

        fn line(len : usize) {
            for _ in 0..len {
                print!("-");
            }
            println!();
        }

        let id = &self.data.id.to_string();
        let discarded = if self.data.discarded { String::from(" (discarded)") } else { String::new() };
        let heading = format!("[{}] {} {}{}", if self.data.complete {"X"} else {" "}, colour::id(&id), colour::task_name(&self.data.name), colour::greyed_out(&discarded));
        println!("{}", heading);

        line(5 + self.data.name.chars().count() + id.chars().count() + discarded.chars().count());
        println!("Priority: {}", self.data.priority.coloured());
        println!("Tags:     [{}]", format_hash_set(&self.data.tags)?);
        println!("Created:  {}", self.data.created);

        let mut max_line_width = 0;
        if let Some(info) = &self.data.info {
            println!("Info:");
            let info_lines : Vec<&str> = info.split("\n").collect();
            for line in info_lines {
                max_line_width = usize::max(max_line_width, line.chars().count() + 4);
                println!("    {}", line);
            }
        }

        line(usize::min(max_line_width, usize::try_from(termsize::get().map(|s| s.cols).unwrap_or(0)).unwrap()));

        // dependencies as a tree
        
        Ok(())
    }
}

fn format_hash_set<T : fmt::Display>(set : &HashSet<T>) -> Result<String, error::Error> {
    let mut output = String::new();

    for value in set.iter() {
        fmt::write(&mut output, format_args!("{}, ", value))?;
    }

    if output.len() != 0 {
        output.pop();
        output.pop();
    }

    Ok(output)
}

pub fn list(vault_folder : &path::Path) -> Result<(), error::Error> {
    let ids : Vec<Id> =
        fs::read_dir(vault_folder.join("notes"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string())
        .filter_map(|n| n.parse::<Id>().ok())
        .collect();

    let mut table = comfy_table::Table::new();

    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

    table.set_header(vec!["Id", "Name", "Tags", "Priority"]);

    for id in ids {
        let task = Task::load(id, vault_folder.to_path_buf(), true)?;

        if !task.data.discarded && !task.data.complete {
            table.add_row(
                vec![
                    task.data.id.to_string(),
                    task.data.name,
                    format_hash_set(&task.data.tags)?,
                    task.data.priority.to_string()
                ]
            );
        }
    }

    println!("{}", table);

    Ok(())
}


