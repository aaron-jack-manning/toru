use crate::tasks;
use crate::tasks::Id;

use std::path;

impl Args {
    pub fn accept_command() -> Command {
        use clap::Parser;
        Args::parse().command
    }
}

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub command : Command,
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
#[clap(version, help_short='h', about, author, global_setting=clap::AppSettings::DisableHelpSubcommand)]
pub enum Command {
    /// Create a new task.
    New {
        #[clap(short, long)]
        name : String,
        #[clap(short, long)]
        info : Option<String>,
        #[clap(short, long)]
        tag : Vec<String>,
        #[clap(short, long)]
        dependency : Vec<Id>,
        #[clap(short, long, value_enum)]
        priority : Option<tasks::Priority>,
        /// Due date, expecting format yyyy-mm-ddThh:mm:ss
        #[clap(long)]
        due : Option<chrono::NaiveDateTime>,
    },
    /// Displays the specified task in detail.
    View {
        id_or_name : String,
    },
    /// Edit a task directly.
    Edit {
        id_or_name : String,
        /// Edit the info specifically in its own file.
        #[clap(short, long)]
        info : bool,
    },
    /// Delete a task (move file to trash).
    Delete {
        id_or_name : String,
    },
    /// Mark a task as complete.
    Complete {
        id_or_name : String,
    },
    /// Run Git commands at the root of the vault.
    #[clap(trailing_var_arg=true)]
    Git {
        args : Vec<String>,
    },
    /// Run Subversion commands at the root of the vault.
    #[clap(trailing_var_arg=true)]
    Svn {
        args : Vec<String>,
    },
    /// Adds the recommended .gitignore file to the vault.
    #[clap(name="gitignore")]
    GitIgnore,
    /// Lists tasks according to the specified fields, ordering and filters.
    List {
        #[clap(flatten)]
        options : ListOptions,
    },
    /// Adds the recommended svn:ignore property to the top level of the vault.
    #[clap(name="svn:ignore")]
    SvnIgnore,
    /// For tracking time against a task.
    Track {
        id_or_name : String,
        #[clap(short='H', default_value_t=0)]
        hours : u16,
        #[clap(short='M', default_value_t=0)]
        minutes : u16,
        /// Date for the time entry [default: Today]
        #[clap(short, long)]
        date : Option<chrono::NaiveDate>,
        /// Message to identify the time entry.
        #[clap(short, long)]
        message : Option<String>,
    },
    /// For statistics about the state of your vault.
    #[clap(subcommand)]
    Stats(StatsCommand),
    /// For making changes to global configuration.
    #[clap(subcommand)]
    Config(ConfigCommand),
    /// Commands for interacting with vaults.
    #[clap(subcommand)]
    Vault(VaultCommand),
    /// Switches to the specified vault.
    Switch {
        name : String,
    },
}

#[derive(clap::StructOpt, Debug, PartialEq, Eq)]
pub struct ListOptions {
    /// Which columns to include.
    #[clap(short, value_enum)]
    pub column : Vec<Column>,
    /// Field to order by.
    #[clap(long, value_enum, default_value_t=OrderBy::Id)]
    pub order_by : OrderBy,
    /// Sort ascending on descending.
    #[clap(long, value_enum, default_value_t=Order::Asc)]
    pub order : Order,
    /// Tags to include.
    #[clap(short, long)]
    pub tag : Vec<String>,
    /// Tags to exclude.
    #[clap(short, long)]
    pub exclude_tag : Vec<String>,
    /// Priority levels to include.
    #[clap(short, long, value_enum)]
    pub priority : Vec<tasks::Priority>,
    /// Only include tasks due before a certain date (inclusive).
    #[clap(long)]
    pub due_before : Option<chrono::NaiveDate>,
    /// Only include tasks due after a certain date (inclusive).
    #[clap(long)]
    pub due_after : Option<chrono::NaiveDate>,
    /// Only include tasks created before a certain date (inclusive).
    #[clap(long)]
    pub created_before : Option<chrono::NaiveDate>,
    /// Only include tasks created after a certain date (inclusive).
    #[clap(long)]
    pub created_after : Option<chrono::NaiveDate>,
    /// Include completed tasks in the list.
    #[clap(long)]
    pub include_completed : bool,
    /// Only include tasks with no dependencies [alias: bottom-level].
    #[clap(long, alias="bottom-level")]
    pub no_dependencies : bool,
    /// Only include tasks with no dependents [alias: top-level].
    #[clap(long, alias="top-level")]
    pub no_dependents : bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Column {
    Due,
    Priority,
    Created,
    Tracked,
    Tags,
    Status,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum OrderBy {
    #[default]
    Id,
    Name,
    Due,
    Priority,
    Created,
    Tracked,
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
pub enum StatsCommand {
    /// View time tracked per tag recently.
    Tracked {
        #[clap(short, long, default_value_t=7)]
        days : u16,
    },
    /// View recently completed tasks.
    Completed {
        #[clap(short, long, default_value_t=7)]
        days : u16,
    },
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
pub enum ConfigCommand {
    /// For checking or changing default text editor command.
    Editor {
        /// Command to launch editor. Omit to view current editor.
        editor : Option<String>,
    }
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
pub enum VaultCommand {
    /// Creates a new vault at the specified location of the given name.
    New {
        name : String,
        path : path::PathBuf,
    },
    /// Disconnects the specified vault from toru, without altering the files.
    Disconnect {
        name : String,
    },
    /// Connects an existing fault to toru.
    Connect {
        name : String,
        path : path::PathBuf,
    },
    /// Deletes the specified vault along with all of its data.
    Delete {
        name : String,
    },
    /// Lists all configured vaults.
    List,
    /// For renaming an already set up vault.
    Rename {
        old_name : String,
        new_name : String,
    }
}

