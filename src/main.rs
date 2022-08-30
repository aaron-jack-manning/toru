mod vcs;
mod edit;
mod vault;
mod index;
mod error;
mod tasks;
mod state;
mod graph;
mod stats;
mod config;
mod colour;

use tasks::Id;

use std::path;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command : Command,
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
#[clap(version, help_short='h', about, author, global_setting=clap::AppSettings::DisableHelpSubcommand)]
enum Command {
    /// Create a new task.
    New {
        #[clap(short, long)]
        name : String,
        #[clap(short, long)]
        info : Option<String>,
        #[clap(short, long)]
        tags : Vec<String>,
        #[clap(short, long)]
        dependencies : Vec<Id>,
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
    column : Vec<Column>,
    /// Field to order by.
    #[clap(long, value_enum, default_value_t=OrderBy::Id)]
    order_by : OrderBy,
    /// Sort ascending on descending.
    #[clap(long, value_enum, default_value_t=Order::Asc)]
    order : Order,
    /// Tags to include.
    #[clap(short, long)]
    tag : Vec<String>,
    /// Only include tasks due before a certain date (inclusive).
    #[clap(long)]
    due_before : Option<chrono::NaiveDate>,
    /// Only include tasks due after a certain date (inclusive).
    #[clap(long)]
    due_after : Option<chrono::NaiveDate>,
    /// Only include tasks created before a certain date (inclusive).
    #[clap(long)]
    created_before : Option<chrono::NaiveDate>,
    /// Only include tasks created after a certain date (inclusive).
    #[clap(long)]
    created_after : Option<chrono::NaiveDate>,
    /// Include completed tasks in the list.
    #[clap(long)]
    include_completed : bool,
    /// Only include notes with no dependencies [alias: bottom-level].
    #[clap(long, alias="bottom-level")]
    no_dependencies : bool,
    /// Only include notes with no dependents [alias: top-level].
    #[clap(long, alias="top-level")]
    no_dependents : bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}

#[derive(Default, Hash, Clone, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Column {
    #[default]
    Due,
    Priority,
    Created,
    Tags,
    Status,
    Tracked,
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
enum StatsCommand {
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
enum ConfigCommand {
    /// For checking or changing default text editor command.
    Editor {
        /// Command to launch editor. Omit to view current editor.
        editor : Option<String>,
    }
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
enum VaultCommand {
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

fn main() {
    let result = program();

    match result {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
        }
    }
}

fn program() -> Result<(), error::Error> {
    let command = {
        use clap::Parser;
        Args::parse().command
    };

    let mut config = config::Config::load()?;

    use Command::*;
    if let Vault(command) = command {
        use VaultCommand::*;
        match command {
            New { name, path } => {
                vault::new(name.clone(), path, &mut config)?;
                println!("Created vault {}", colour::text::vault(&name));
            },
            Disconnect { name } => {
                vault::disconnect(&name, &mut config)?;
                println!("Disconnected vault {}", colour::text::vault(&name));
            },
            Connect { name , path } => {
                vault::connect(name.clone(), path, &mut config)?;
                println!("Connected vault {}", colour::text::vault(&name));
            },
            Delete { name } => {
                vault::delete(&name, &mut config)?;
                println!("Deleted vault {}", colour::text::vault(&name));
            },
            List => {
                config.list_vaults()?;
            },
            Rename { old_name, new_name } => {
                config.rename_vault(&old_name, new_name.clone())?;
                println!("Renamed vault {} to {}", colour::text::vault(&old_name), colour::text::vault(&new_name));
            }
        }
    }
    else if let Config(command) = command {
        use ConfigCommand::*;
        match command {
            Editor { editor } => {
                match editor {
                    Some(editor) => {
                        config.editor = editor;
                        println!("Updated editor command to: {}", config.editor);
                    },
                    None => {
                        println!("Current editor command: {}", config.editor);
                    }
                }
            }
        }
    }
    else if let Switch { name } = command {
        config.switch(&name)?;
        println!("Switched to vault {}", colour::text::vault(&name));
    }
    else if let Git { args } = command {
        let vault_folder = &config.current_vault()?.1;
        vcs::command(args, vcs::Vcs::Git, vault_folder)?;
    }
    else if command == GitIgnore {
        let vault_folder = &config.current_vault()?.1;
        vcs::create_gitignore(vault_folder)?;
        println!("Default {} file created", colour::text::file(".gitignore"));
    }
    else if let Svn { args } = command {
        let vault_folder = &config.current_vault()?.1;
        vcs::command(args, vcs::Vcs::Svn, vault_folder)?;
    }
    // Commands that require loading in the state.
    else {
        let vault_folder = &config.current_vault()?.1;
        let mut state = state::State::load(vault_folder)?;

        match command {
            New { name, info, tags, dependencies, priority, due } => {
                let task = tasks::Task::new(name, info, tags, dependencies, priority, due, vault_folder, &mut state)?;
                println!("Created task {} (ID: {})", colour::text::task(&task.data.name), colour::text::id(task.data.id));
            },
            Delete { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, false)?;
                let name = task.data.name.clone();
                state.data.index.remove(task.data.name.clone(), task.data.id);
                state.data.deps.remove_node(task.data.id);
                task.delete()?;

                println!("Deleted task {} (ID: {})", colour::text::task(&name), colour::text::id(id));
            },
            View { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, true)?;
                task.display(vault_folder, &state)?;
            },
            Edit { id_or_name, info } => {
                let id = state.data.index.lookup(&id_or_name)?;
                if info {
                    edit::edit_info(id, vault_folder.clone(), &config.editor)?;
                }
                else {
                    edit::edit_raw(id, vault_folder.clone(), &config.editor, &mut state)?;
                }
                println!("Updated task {}", colour::text::id(id));
            },
            Track { id_or_name, hours, minutes, date, message } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                let entry =  tasks::TimeEntry::new(hours, minutes, date, message);
                task.data.time_entries.push(entry);
                task.save()?;
            },
            Stats(command) => {
                use StatsCommand::*;
                match command {
                    Tracked { days } => {
                        stats::time_per_tag(days, vault_folder)?;
                    },
                    Completed { days } => {
                        stats::completed_tasks(days, vault_folder)?;
                    }
                }
            },
            Complete { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                task.data.completed = Some(chrono::Local::now().naive_local());
                task.save()?;
                println!("Marked task {} as complete", colour::text::id(id));
            },
            List { options } => {
                tasks::list(options, vault_folder, &state)?;
            },
            // All commands which are dealt with in if let chain at start.
            Vault(_) | Config(_) | Git { args : _ } | Svn { args : _ } | Switch { name : _ } | GitIgnore => unreachable!(),
        }

        state.save()?;
    }

    config.save()?;

    Ok(())
}
