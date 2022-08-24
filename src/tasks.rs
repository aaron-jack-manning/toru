use crate::error;
use crate::state;
use crate::colour;

use std::io;
use std::fs;
use std::fmt;
use std::ops;
use std::mem;
use std::path;
use std::io::{Write, Seek};
use std::collections::HashSet;
use colored::Colorize;
use chrono::SubsecRound;

pub type Id = u64;

pub struct Task {
    path : path::PathBuf,
    file : fs::File,
    pub data : InternalTask,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    pub logged_date : chrono::NaiveDate,
    pub duration : Duration,
}

// Needs to preserve representation invariant of minutes < 60
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Duration {
    hours : u16,
    minutes : u16,
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InternalTask {
    pub id : Id,
    pub name : String,
    pub tags : HashSet<String>,
    pub dependencies : HashSet<Id>,
    pub priority : Priority,
    pub due : Option<chrono::NaiveDateTime>,
    pub created : chrono::NaiveDateTime,
    pub completed : Option<chrono::NaiveDateTime>,
    pub discarded : bool,
    pub info : Option<String>,
    pub time_entries : Vec<TimeEntry>,
}

impl Task {
    pub fn new(name : String, info : Option<String>, tags : Vec<String>, dependencies : Vec<Id>, priority : Option<Priority>, due : Option<chrono::NaiveDateTime>, vault_folder : &path::Path, state : &mut state::State) -> Result<Self, error::Error> {

        if name.chars().all(|c| c.is_numeric()) {
            return Err(error::Error::Generic(String::from("Name must not be purely numeric")));
        };

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
            due,
            time_entries : Vec::new(),
            created : chrono::Local::now().naive_local(),
            completed : None,
            discarded : false,
        };

        let file_contents = toml::to_string(&data)?;

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(file_contents.as_bytes())?;

        state.index_insert(data.name.clone(), id);

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
    pub fn load(id : Id, vault_folder : &path::Path, read_only : bool) -> Result<Self, error::Error> {
        let path = Task::check_exists(id, vault_folder)?;

        Task::load_direct(path, read_only)
    }

    pub fn load_all(vault_folder : &path::Path, read_only : bool) -> Result<Vec<Self>, error::Error> {
        let ids : Vec<Id> =
            fs::read_dir(vault_folder.join("notes"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|p| p.is_file())
            .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string())
            .filter_map(|n| n.parse::<Id>().ok())
            .collect();

        let mut tasks = Vec::with_capacity(ids.len());
        for id in ids {
            tasks.push(Task::load(id, vault_folder, read_only)?);
        }

        Ok(tasks)
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
            path : _,
            mut file,
            data,
        } = self;

        let file_contents = toml::to_string(&data)?;

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(file_contents.as_bytes())?;

        Ok(())
    }

    pub fn delete(self) -> Result<(), error::Error> {
        let Self {
            path,
            file,
            data : _,
        } = self;

        mem::drop(file);
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

        let (heading, heading_length) = {
            let id = &self.data.id.to_string();
            let discarded = if self.data.discarded { String::from(" (discarded)") } else { String::new() };

            (
                format!("[{}] {} {}{}", if self.data.completed.is_some() {"X"} else {" "}, colour::id(id), colour::task_name(&self.data.name), colour::greyed_out(&discarded)),
                5 + self.data.name.chars().count() + id.chars().count() + discarded.chars().count()
            )
        };

        println!("{}", heading);
        line(heading_length);

        println!("Priority:     {}", self.data.priority.coloured());
        println!("Tags:         [{}]", format_hash_set(&self.data.tags)?);
        println!("Created:      {}", self.data.created.round_subsecs(0));
        
        if let Some(due) = self.data.due {
            let due = format_due_date(&due, self.data.completed.is_none(), true);
            println!("Due:          {}", due);
        }

        if let Some(mut info) = self.data.info.clone() {
            let mut max_line_width = 0;
            println!("Info:");

            while info.ends_with('\n') {
                info.pop();
            }

            let info_lines : Vec<&str> = info.split('\n').collect();
            for line in info_lines {
                max_line_width = usize::max(max_line_width, line.chars().count() + 4);
                println!("    {}", line);
            }
        }

        if !self.data.time_entries.is_empty() {

            let mut entries = self.data.time_entries.clone();
            // Sort entries by date.
            entries.sort_by(|e1, e2| e1.logged_date.cmp(&e2.logged_date));

            let mut total = Duration::zero();
            let mut lines = Vec::with_capacity(entries.len());
            for entry in &entries {
                lines.push(format!("    {} [{}]", entry.duration, entry.logged_date));
                total = total + entry.duration;
            }

            println!("Time Entries (totaling {}):", total);
            for line in lines {
                println!("{}", line);
            }
        }

        // dependencies as a tree
        
        Ok(())
    }
}

fn format_hash_set<T : fmt::Display>(set : &HashSet<T>) -> Result<String, error::Error> {
    let mut output = String::new();

    for value in set.iter() {
        fmt::write(&mut output, format_args!("{}, ", value))?;
    }

    // Remove the trailing comma and space.
    if !output.is_empty() {
        output.pop();
        output.pop();
    }

    Ok(output)
}

fn format_due_date(due : &chrono::NaiveDateTime, include_fuzzy_period : bool, colour : bool) -> String {
    let remaining = *due - chrono::Local::now().naive_local();

    let fuzzy_period = if remaining.num_days() != 0 {
        let days = remaining.num_days().abs();
        format!("{} day{}", days, if days == 1 {""} else {"s"})
    }
    else if remaining.num_hours() != 0 {
        let hours = remaining.num_hours().abs();
        format!("{} hour{}", hours, if hours == 1 {""} else {"s"})
    }
    else if remaining.num_minutes() != 0 {
        let minutes = remaining.num_minutes().abs();
        format!("{} minute{}", minutes, if minutes == 1 {""} else {"s"})
    }
    else {
        let seconds = remaining.num_seconds().abs();
        format!("{} second{}", seconds, if seconds == 1 {""} else {"s"})
    };

    if include_fuzzy_period {
        if colour {
            if remaining < chrono::Duration::zero() {
                format!("{} {}", due.round_subsecs(0), colour::due_date::overdue(&format!("({} overdue)", fuzzy_period)))
            }
            else if remaining < chrono::Duration::days(1) {
                format!("{} {}", due.round_subsecs(0), colour::due_date::very_close(&format!("({} remaining)", fuzzy_period)))

            }
            else if remaining < chrono::Duration::days(3) {
                format!("{} {}", due.round_subsecs(0), colour::due_date::close(&format!("({} remaining)", fuzzy_period)))

            }
            else {
                format!("{} {}", due.round_subsecs(0), colour::due_date::plenty_of_time(&format!("({} remaining)", fuzzy_period)))
            }
        }
        else {
            if remaining < chrono::Duration::zero() {
                format!("{} {}", due.round_subsecs(0), format!("({} overdue)", fuzzy_period))
            }
            else if remaining < chrono::Duration::days(1) {
                format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period))

            }
            else if remaining < chrono::Duration::days(3) {
                format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period))

            }
            else {
                format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period))
            }
        }
    }
    else {
        format!("{}", due.round_subsecs(0))
    }
}

pub fn list(vault_folder : &path::Path) -> Result<(), error::Error> {

    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_header(vec!["Id", "Name", "Tags", "Priority", "Tracked", "Due"]);

    let mut tasks = Task::load_all(vault_folder, true)?;
    tasks.sort_by(|t1, t2| t2.data.priority.cmp(&t1.data.priority));

    for task in tasks {
        if !task.data.discarded && task.data.completed.is_none() {

            let duration = TimeEntry::total(&task.data.time_entries);

            table.add_row(
                vec![
                    task.data.id.to_string(),
                    task.data.name,
                    format_hash_set(&task.data.tags)?,
                    task.data.priority.to_string(),
                    if duration == Duration::zero() { String::new() } else { duration.to_string() },
                    match task.data.due { Some(due) => format_due_date(&due, task.data.completed.is_none(), false), None => String::new() },
                ]
            );
        }
    }

    println!("{}", table);

    Ok(())
}

pub fn clean(vault_folder : &path::Path) -> Result<(), error::Error> {

    let tasks = Task::load_all(vault_folder, false)?;

    for task in tasks {
        if task.data.discarded {
            task.delete()?;
        }
    }

    Ok(())
}

impl ops::Add for Duration {
    type Output = Self;

    fn add(self, other : Self) -> Self::Output {

        Self {
            hours : self.hours + other.hours + (self.minutes + other.minutes) / 60,
            minutes : (self.minutes + other.minutes) % 60,
        }
    }
}

impl Duration {
    pub fn zero() -> Self {
        Self {
            minutes : 0,
            hours : 0,
        }
    }
}


impl ops::Div<usize> for Duration {
    type Output = Self;

    fn div(self, divisor : usize) -> Self::Output {
        let total_mins = f64::from(self.hours * 60 + self.minutes);
        let divided_mins = total_mins / (divisor as f64);
        let divided_mins = divided_mins.round() as u16;

        Self {
            hours : divided_mins / 60,
            minutes : divided_mins % 60,
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:0>2}", self.hours, self.minutes)
    }
}

impl TimeEntry {
    /// Adds up the times from a collection of time entries.
    fn total(entries : &Vec<TimeEntry>) -> Duration {
        entries
        .into_iter()
        .map(|e| e.duration)
        .fold(Duration::zero(), |a, d| a + d)
    }
}

impl TimeEntry {
    pub fn new(hours : u16, minutes : u16) -> Self {

        let (hours, minutes) = {
            (hours + minutes / 60, minutes % 60)
        };

        Self {
            logged_date : chrono::Utc::now().naive_local().date(),
            duration : Duration {
                hours,
                minutes,
            }
        }
    }
}


