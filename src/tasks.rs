use crate::error;
use crate::state;
use crate::format;

use std::io;
use std::fs;
use std::str;
use std::mem;
use std::cmp;
use std::path;
use std::io::{Write, Seek};
use std::collections::{HashSet, HashMap, BTreeSet};
use chrono::SubsecRound;

pub type Id = u64;

pub struct Task {
    pub path : path::PathBuf,
    // This should only be None for a new task, in which case it should be written from the path.
    file : Option<fs::File>,
    pub data : InternalTask,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InternalTask {
    pub id : Id,
    pub name : String,
    pub tags : HashSet<String>,
    pub dependencies : BTreeSet<Id>,
    pub priority : Priority,
    pub due : Option<chrono::NaiveDateTime>,
    pub created : chrono::NaiveDateTime,
    pub completed : Option<chrono::NaiveDateTime>,
    pub info : Option<String>,
    pub time_entries : Vec<TimeEntry>,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    Backlog,
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    hours : u16,
    minutes : u16,
}

impl Duration {
    pub fn zero() -> Self {
        Self {
            hours : 0,
            minutes : 0,
        }
    }
}

pub mod duration {
    use super::Duration;

    use std::ops;
    use std::str;
    use std::fmt;

    /// Serialize to custom format HH:MM where MM is padded to be two characters wide and HH can be
    /// arbitrarily large.
    impl serde::Serialize for Duration {
        fn serialize<S : serde::Serializer>(&self, serializer : S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&format!("{}:{:0>2}", self.hours, self.minutes))
        }
    }

    /// Deserialize from custom format HH:MM where MM is an integer between 0 and 59 inclusive, and
    /// HH is some integer representable as a u16.
    /// The width of MM is not enforced for deserialization.
    impl<'de> serde::Deserialize<'de> for Duration {
        fn deserialize<D : serde::Deserializer<'de>>(deserializer : D) -> Result<Self, D::Error> {
            let raw = String::deserialize(deserializer)?;

            use std::str::FromStr;
            Self::from_str(&raw)
            .map_err(|x| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&raw), &x.serde_expected()))
        }
    }

    /// Custom type for errors when converting duration from str, with error messages for clap and
    /// serde respectively.
    #[derive(Debug)]
    pub enum DurationRead {
        /// For when the number of minutes is not less than 60.
        Minutes,
        /// For when either value cannot be parsed into a u16.
        Range,
        /// For general formatting error (i.e. split at colon doesn't produce two values).
        General,
    }

    impl fmt::Display for DurationRead {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DurationRead::Minutes => {
                    write!(f, "the number of minutes must be between 0 and 59 inclusive")
                },
                DurationRead::Range => {
                    write!(f, "the number of hours and minutes must be representable as a u16")
                },
                DurationRead::General => {
                    write!(f, "duration must be in the format HH:MM where HH is any integer (representable as a u16) and MM is an integer between 0 and 59 inclusive")
                },
            }
        }
    }

    impl std::error::Error for DurationRead { }

    impl DurationRead {
        /// Gives a str of what was expected (and not provided) when serializing.
        pub fn serde_expected(&self) -> &'static str {
            match self {
                DurationRead::Minutes => {
                    "the number of minutes to be an integer between 0 and 59 inclusive"
                },
                DurationRead::Range => {
                    "the number of hours and minutes to be representable as a u16"
                },
                DurationRead::General => {
                    "a duration in the format HH:MM where HH is any integer (representable as a u16) and MM is an integer between 0 and 59 inclusive"
                },
            }
        }
    }

    impl str::FromStr for Duration {
        type Err = DurationRead;

        fn from_str(s : &str) -> Result<Self, Self::Err> {
            if let &[h, m] = &s.split(':').collect::<Vec<&str>>()[..] {
                if let (Ok(hours), Ok(minutes)) = (h.parse::<u16>(), m.parse::<u16>()) {
                    if minutes < 60 {
                        Ok(Self {
                            hours,
                            minutes,
                        })
                    }
                    else {
                        Err(DurationRead::Minutes)
                    }
                }
                else {
                    Err(DurationRead::Range)
                }
            }
            else {
                Err(DurationRead::General)
            }
        }
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

    /// Same display format as serialization.
    impl fmt::Display for Duration {
        fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}:{:0>2}", self.hours, self.minutes)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    pub logged_date : chrono::NaiveDate,
    pub message : Option<String>,
    pub duration : Duration,
}

impl TimeEntry {
    /// Adds up the times from a collection of time entries.
    pub fn total(entries : &[TimeEntry]) -> Duration {
        entries
        .iter()
        .map(|e| e.duration)
        .fold(Duration::zero(), |a, d| a + d)
    }

    /// Creates a new TimeEntry, correctly validating and setting defaults.
    pub fn new(duration : Duration, date : Option<chrono::NaiveDate>, message : Option<String>) -> Self {

        Self {
            logged_date : date.unwrap_or(chrono::Local::now().naive_local().date()),
            message,
            duration,
        }
    }
}

impl Task {
    /// Creates a new task from the input data.
    pub fn new(name : String, info : Option<String>, tags : Vec<String>, dependencies : Vec<Id>, priority : Option<Priority>, due : Option<chrono::NaiveDateTime>, vault_folder : &path::Path, state : &mut state::State) -> Result<Id, error::Error> {

        // Update the state with the new next Id.
        let id = state.data.next_id;
        state.data.next_id += 1;
        
        let path = vault_folder.join("tasks").join(&format!("{}.toml", id));

        // Adding to dependency graph appropriately.
        state.data.deps.insert_node(id);
        if !dependencies.is_empty() {
            for dependency in &dependencies {
                if state.data.deps.contains_node(*dependency) {
                    state.data.deps.insert_edge(id, *dependency)?;
                }
                else {
                    return Err(error::Error::Generic(format!("No task with an ID of {} exists", format::id(*dependency))));
                }
            }
        }

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
        };

        state.data.index.insert(data.name.clone(), id);

        let task = Task {
            path,
            file : None,
            data,
        };

        task.save()?;

        Ok(id)
    }

    /// Loads a task directly from its path, for use with the temporary edit file.
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
            file : Some(file),
            data,
        })
    }

    /// Loads a task in to memory.
    pub fn load(id : Id, vault_folder : &path::Path, read_only : bool) -> Result<Self, error::Error> {
        let path = Task::check_exists(id, vault_folder)?;

        Task::load_direct(path, read_only)
    }

    /// Get an iterator over the IDs of tasks in a vault.
    fn id_iter(vault_folder : &path::Path) -> impl Iterator<Item = u64> {
        fs::read_dir(vault_folder.join("tasks"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string())
        .filter_map(|n| n.parse::<Id>().ok())
    }

    /// Load all tasks of a vault into a `Vec`.
    pub fn load_all(vault_folder : &path::Path, read_only : bool) -> Result<Vec<Self>, error::Error> {
        let ids = Task::id_iter(vault_folder);
        
        let mut tasks = Vec::new();
        for id in ids {
            tasks.push(Task::load(id, vault_folder, read_only)?);
        }

        Ok(tasks)
    }

    /// Load all tasks of a vault into a `HashMap`.
    pub fn load_all_as_map(vault_folder : &path::Path, read_only : bool) -> Result<HashMap<Id, Self>, error::Error> {
        let ids = Task::id_iter(vault_folder);

        let mut tasks = HashMap::new();
        for id in ids {
            tasks.insert(id, Task::load(id, vault_folder, read_only)?);
        }

        Ok(tasks)
    }

    /// Checks that a task with the prodided ID exists in the provided vault_folder. Returns the
    /// path of that task.
    pub fn check_exists(id : Id, vault_folder : &path::Path) -> Result<path::PathBuf, error::Error> {
        let path = vault_folder.join("tasks").join(format!("{}.toml", id));
        if path.exists() && path.is_file() {
            Ok(path)
        }
        else {
            Err(error::Error::Generic(format!("No task with the ID {} exists", format::id(id))))
        }
    }

    /// Saves the in memory task data to the corresponding file.
    pub fn save(self) -> Result<(), error::Error> {

        // Enforce any additional invariants which need to be checked for both edits and now tasks
        // at the point of save.
        {
            // Exclude numeric names in the interest of allowing commands that take in ID or name.
            if self.data.name.chars().all(|c| c.is_numeric()) {
                return Err(error::Error::Generic(String::from("Name must not be purely numeric")));
            };
        }

        let Self {
            path,
            file,
            data,
        } = self;

        let file_contents = toml::to_string(&data)?;

        // Check if the file exists, if not it is a new task and the file must be written from the
        // path.
        match file {
            Some(mut file) => {
                file.set_len(0)?;
                file.seek(io::SeekFrom::Start(0))?;
                file.write_all(file_contents.as_bytes())?;
            },
            None => {
                fs::write(path, file_contents.as_bytes())?;
            }
        }

        Ok(())
    }

    /// Deletes the task.
    pub fn delete(self) -> Result<(), error::Error> {
        let Self {
            path,
            file,
            data : _,
        } = self;

        mem::drop(file);
        trash::delete(&path)?;

        Ok(())
    }

    /// Displays a task to the terminal.
    pub fn display(&self, vault_folder : &path::Path, state : &state::State) -> Result<(), error::Error> {
        
        /// Displays a line of hyphens of a specified length.
        fn line(len : usize) {
            for _ in 0..len {
                print!("-");
            }
            println!();
        }

        let (heading, heading_length) =
            (
                format!("[{}] {} {}", if self.data.completed.is_some() {"X"} else {" "}, format::id(self.data.id), format::task(&self.data.name)),
                5 + self.data.name.chars().count() + self.data.id.to_string().chars().count()
            );

        println!("{}", heading);
        line(heading_length);

        println!("Priority:     {}", format::priority(&self.data.priority));
        println!("Tags:         [{}]", format::hash_set(&self.data.tags)?);
        println!("Created:      {}", self.data.created.round_subsecs(0));
        
        if let Some(due) = self.data.due {
            let due = format::due_date(&due, self.data.completed.is_none());
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

        // Display tracked time.
        if !self.data.time_entries.is_empty() {

            let mut entries = self.data.time_entries.clone();
            // Sort time entries by date.
            entries.sort_by(|e1, e2| e1.logged_date.cmp(&e2.logged_date));

            let mut total = Duration::zero();
            let mut lines = Vec::with_capacity(entries.len());
            for entry in &entries {
                lines.push(format!(
                    "    {} [{}] {}",
                    entry.duration,
                    entry.logged_date,
                    entry.message.as_ref().unwrap_or(&String::new())
                ));
                total = total + entry.duration;
            }

            println!("Time Entries (totaling {}):", total);
            for line in lines {
                println!("{}", line);
            }
        }

        // Display dependencies as tree.
        if !self.data.dependencies.is_empty() {

            println!("Dependencies:");
            format::dependencies(self.data.id, vault_folder, &state.data.deps)?;
        }
        
        Ok(())
    }
}


/// Compares due dates correctly, treating None as at infinity.
pub fn compare_due_dates<T : Ord>(first : &Option<T>, second : &Option<T>) -> cmp::Ordering {
    match (first, second) {
        (None, None) => cmp::Ordering::Equal,
        (Some(_), None) => cmp::Ordering::Less,
        (None, Some(_)) => cmp::Ordering::Greater,
        (Some(first), Some(second)) => first.cmp(second),
    }
}

