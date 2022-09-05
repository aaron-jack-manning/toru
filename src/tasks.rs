use crate::args;
use crate::error;
use crate::state;
use crate::format;

use std::io;
use std::fs;
use std::fmt;
use std::ops;
use std::mem;
use std::cmp;
use std::path;
use std::io::{Write, Seek};
use std::collections::{HashSet, HashMap};
use chrono::SubsecRound;

pub type Id = u64;

pub struct Task {
    pub path : path::PathBuf,
    file : fs::File,
    pub data : InternalTask,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    Backlog,
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    pub logged_date : chrono::NaiveDate,
    pub message : Option<String>,
    pub duration : Duration,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
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
    pub info : Option<String>,
    pub time_entries : Vec<TimeEntry>,
}

impl Task {
    /// Creates a new task from the input data.
    pub fn new(name : String, info : Option<String>, tags : Vec<String>, dependencies : Vec<Id>, priority : Option<Priority>, due : Option<chrono::NaiveDateTime>, vault_folder : &path::Path, state : &mut state::State) -> Result<Self, error::Error> {

        // Exclude numeric names in the interest of allowing commands that take in ID or name.
        if name.chars().all(|c| c.is_numeric()) {
            return Err(error::Error::Generic(String::from("Name must not be purely numeric")));
        };

        // Update the state with the new next Id.
        let id = state.data.next_id;
        state.data.next_id += 1;
        
        let path = vault_folder.join("tasks").join(&format!("{}.toml", id));

        let mut file = fs::File::options()
            .write(true)
            .create(true)
            .open(&path)?;

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

        let file_contents = toml::to_string(&data)?;

        file.set_len(0)?;
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(file_contents.as_bytes())?;

        state.data.index.insert(data.name.clone(), id);

        Ok(Task {
            path,
            file,
            data,
        })
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
            file,
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

impl Duration {
    pub fn zero() -> Self {
        Self {
            minutes : 0,
            hours : 0,
        }
    }

    pub fn satisfies_invariant(&self) -> bool {
        self.minutes < 60
    }
}

impl TimeEntry {
    /// Adds up the times from a collection of time entries.
    fn total(entries : &[TimeEntry]) -> Duration {
        entries
        .iter()
        .map(|e| e.duration)
        .fold(Duration::zero(), |a, d| a + d)
    }

    /// Creates a new TimeEntry, correctly validating and setting defaults.
    pub fn new(hours : u16, minutes : u16, date : Option<chrono::NaiveDate>, message : Option<String>) -> Self {

        let (hours, minutes) = {
            (hours + minutes / 60, minutes % 60)
        };

        Self {
            logged_date : date.unwrap_or(chrono::Utc::now().naive_local().date()),
            message,
            duration : Duration {
                hours,
                minutes,
            }
        }
    }
}

/// Compares due dates correctly, treating None as at infinity.
fn compare_due_dates<T : Ord>(first : &Option<T>, second : &Option<T>) -> cmp::Ordering {
    match (first, second) {
        (None, None) => cmp::Ordering::Equal,
        (Some(_), None) => cmp::Ordering::Less,
        (None, Some(_)) => cmp::Ordering::Greater,
        (Some(first), Some(second)) => first.cmp(second),
    }
}


impl args::ListOptions {
    /// Combines list options coming from a profile and from the additional arguments given. Order
    /// of the arguments provided matters, hence the argument names (because optional arguments
    /// from the profile are overwritten by the additional arguments).
    pub fn combine(profile : &Self, additional : &Self) -> Self {
        /// Joins two vectors together one after the other, creating a new allocation.
        fn concat<T : Clone>(a : &Vec<T>, b : &Vec<T>) -> Vec<T> {
            let mut a = a.clone();
            a.extend(b.iter().cloned());
            a
        }

        /// Takes two options, and prioritises the second if it is provided in the output, using
        /// the first as a fallback, and returning None if both are None.
        fn join_options<T : Clone>(a : &Option<T>, b : &Option<T>) -> Option<T> {
            match (a, b) {
                (Some(_), Some(b)) => Some(b.clone()),
                (Some(a), None) => Some(a.clone()),
                (None, Some(b)) => Some(b.clone()),
                (None, None) => None,
            }
        }

        Self {
            column : concat(&profile.column, &additional.column),
            order_by : join_options(&profile.order_by, &additional.order_by),
            order : join_options(&profile.order, &profile.order),
            tag : concat(&profile.tag, &additional.tag),
            exclude_tag : concat(&profile.exclude_tag, &additional.exclude_tag),
            priority : concat(&profile.priority, &additional.priority),
            due_before : join_options(&profile.due_before, &additional.due_before),
            due_after : join_options(&profile.due_after, &additional.due_after),
            created_before : join_options(&profile.created_before, &additional.created_before),
            created_after : join_options(&profile.created_after, &additional.created_after),
            include_completed : profile.include_completed || additional.include_completed,
            no_dependencies : profile.no_dependencies || additional.no_dependencies,
            no_dependents : profile.no_dependents || additional.no_dependents,
        }
    }
}

/// Lists all tasks in the specified vault.
pub fn list(mut options : args::ListOptions, vault_folder : &path::Path, state : &state::State) -> Result<(), error::Error> {

    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);


    let mut tasks : Box<dyn Iterator<Item = Task>> = Box::new(Task::load_all(vault_folder, true)?.into_iter());

    // Filter the tasks.
    if let Some(date) = options.created_before {
        tasks = Box::new(tasks.filter(move |t| t.data.created.date() <= date));
    }
    if let Some(date) = options.created_after {
        tasks = Box::new(tasks.filter(move |t| t.data.created.date() >= date));
    }

    if let Some(date) = options.due_before {
        tasks = Box::new(tasks.filter(move |t| {
            match compare_due_dates(&t.data.due.map(|d| d.date()), &Some(date)) {
                cmp::Ordering::Less | cmp::Ordering::Equal => true,
                cmp::Ordering::Greater => false,
            }
        }));
    }
    if let Some(date) = options.due_after {
        tasks = Box::new(tasks.filter(move |t| {
            match compare_due_dates(&t.data.due.map(|d| d.date()), &Some(date)) {
                cmp::Ordering::Greater | cmp::Ordering::Equal => true,
                cmp::Ordering::Less => false,
            }
        }));
    }

    if !options.include_completed {
        tasks = Box::new(tasks.filter(|t| t.data.completed.is_none()));
    }

    if !options.tag.is_empty() {
        let specified_tags : HashSet<_> = options.tag.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            let task_tags : HashSet<_> = t.data.tags.iter().collect();

            // Non empty intersection of tags means the task should be displayed
            specified_tags.intersection(&task_tags).next().is_some()
        }));
    }

    if !options.exclude_tag.is_empty() {
        let specified_tags : HashSet<_> = options.exclude_tag.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            let task_tags : HashSet<_> = t.data.tags.iter().collect();

            // If the task contains a tag which was supposed to be excluded, it should be filtered
            // out
            !specified_tags.intersection(&task_tags).next().is_some()
        }));
    }

    if !options.priority.is_empty() {
        let specified_priority_levels : HashSet<_> = options.priority.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            specified_priority_levels.contains(&t.data.priority)
        }));
    }

    if options.no_dependencies {
        tasks = Box::new(tasks.filter(move |t| {
            t.data.dependencies.is_empty()
        }));
    }

    if options.no_dependents {
        let tasks_with_dependents = state.data.deps.get_tasks_with_dependents();

        tasks = Box::new(tasks.filter(move |t| {
            !tasks_with_dependents.contains(&t.data.id)
        }));
    }

    let mut tasks : Vec<Task> = tasks.collect();


    // Sort the tasks.
    use super::{OrderBy, Order};
    match options.order_by.unwrap_or_default() {
        OrderBy::Id => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.id.cmp(&t2.data.id));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.id.cmp(&t1.data.id));
                },
            }
        },
        OrderBy::Name => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.name.cmp(&t2.data.name));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.name.cmp(&t1.data.name));
                },
            }
        },
        OrderBy::Due => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| compare_due_dates(&t1.data.due, &t2.data.due));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| compare_due_dates(&t2.data.due, &t1.data.due));
                },
            }
        },
        OrderBy::Priority => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.priority.cmp(&t2.data.priority));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.priority.cmp(&t1.data.priority));
                },
            }
        },
        OrderBy::Created => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.created.cmp(&t2.data.created));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.created.cmp(&t1.data.created));
                },
            }
        },
        OrderBy::Tracked => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| TimeEntry::total(&t1.data.time_entries).cmp(&TimeEntry::total(&t2.data.time_entries)));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| TimeEntry::total(&t2.data.time_entries).cmp(&TimeEntry::total(&t1.data.time_entries)));
                },
            }
        }
    }

    // Include the required columns
    let mut headers = vec!["Id", "Name"];

    // Remove duplicate columns.
    options.column = {
        let mut columns = HashSet::new();

        options.column.clone()
            .into_iter()
            .filter(|c| {
                if columns.contains(c) {
                    false
                }
                else {
                    columns.insert(c.clone());
                    true
                }
            })
            .collect()
    };
    
    use super::Column;
    for column in &options.column {
        match column {
            Column::Tracked => {
                headers.push("Tracked");
            },
            Column::Due => {
                headers.push("Due");
            },
            Column::Tags => {
                headers.push("Tags");
            },
            Column::Priority => {
                headers.push("Priority");
            },
            Column::Status => {
                headers.push("Status");
            },
            Column::Created => {
                headers.push("Created");
            },
        }
    }

    table.set_header(headers);

    for task in tasks {

        use comfy_table::Cell;
        let mut row = vec![Cell::from(task.data.id), Cell::from(task.data.name)];

        for column in &options.column {
            match column {
                Column::Tracked => {
                    let duration = TimeEntry::total(&task.data.time_entries);
                    row.push(
                        Cell::from(if duration == Duration::zero() { String::new() } else { duration.to_string() })
                    );
                },
                Column::Due => {
                    row.push(match task.data.due {
                        Some(due) => format::cell::due_date(&due, task.data.completed.is_none()),
                        None => Cell::from(String::new())
                    });
                },
                Column::Tags => {
                    row.push(Cell::new(format::hash_set(&task.data.tags)?));
                },
                Column::Priority => {
                    row.push(format::cell::priority(&task.data.priority));
                },
                Column::Status => {
                    row.push(
                        Cell::new(if task.data.completed.is_some() {
                            String::from("complete")
                        }
                        else {
                            String::from("incomplete")
                        })
                    );
                },
                Column::Created => {
                    row.push(Cell::new(task.data.created.round_subsecs(0).to_string()));
                },
            }
        }

        table.add_row(row);
    }

    println!("{}", table);

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


