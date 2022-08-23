mod git;
mod edit;
mod vault;
mod error;
mod tasks;
mod state;
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
#[clap(version, help_short='H', about, author, global_setting=clap::AppSettings::DisableHelpSubcommand)]
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
    },
    /// Displays the specified task in detail.
    View {
        id_or_name : String,
    },
    /// Edit a note directly.
    Edit {
        id_or_name : String,
        /// Edit the info specifically in its own file.
        #[clap(short, long)]
        info : bool,
    },
    /// Delete a task completely.
    Delete {
        id_or_name : String,
    },
    /// Deletes all discarded tasks.
    Clean,
    /// Discard a task without deleting the underlying file.
    Discard {
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
    /// Adds the recommended .gitignore file to the vault.
    #[clap(name="gitignore")]
    GitIgnore,
    /// Lists tasks according to the specified ordering and filters.
    List {
        // Need to have options for:
        // - column to order by
        // - ascending or descending
        // - which columns to include
        // - filters which exclude values
    },
    /// For tracking time against a note.
    Track {
        id_or_name : String,
        #[clap(short, default_value_t=0)]
        hours : u16,
        #[clap(short, default_value_t=0)]
        minutes : u16,
    },
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
        Err(error::Error::Generic(message)) => {
            println!("{} {}", colour::error("Error:"), message);
        }
        result => println!("{:?}", result),
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
                println!("Created vault {}", colour::vault(&name));
            },
            Disconnect { name } => {
                vault::disconnect(&name, &mut config)?;
                println!("Disconnected vault {}", colour::vault(&name));
            },
            Connect { name , path } => {
                vault::connect(name.clone(), path, &mut config)?;
                println!("Connected vault {}", colour::vault(&name));
            },
            Delete { name } => {
                vault::delete(&name, &mut config)?;
                println!("Deleted vault {}", colour::vault(&name));
            },
            List => {
                config.list_vaults()?;
            },
            Rename { old_name, new_name } => {
                config.rename_vault(&old_name, new_name.clone())?;
                println!("Renamed vault {} to {}", colour::vault(&old_name), colour::vault(&new_name));
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
        println!("Switched to vault {}", colour::vault(&name));
    }
    else if let Git { args } = command {
        let vault_folder = &config.current_vault()?.1;
        git::run_command(args, vault_folder)?;
    }
    else if command == GitIgnore {
        let vault_folder = &config.current_vault()?.1;
        git::create_gitignore(vault_folder)?;
        println!("Default .gitignore file created");
    }
    // Commands that require loading in the state.
    else {
        let vault_folder = &config.current_vault()?.1;
        let mut state = state::State::load(vault_folder)?;

        match command {
            New { name, info, tags, dependencies, priority } => {
                let task = tasks::Task::new(name, info, tags, dependencies, priority, vault_folder, &mut state)?;
                println!("Created task {} (ID: {})", colour::task_name(&task.data.name), colour::id(&task.data.id.to_string()));
            },
            Delete { id_or_name } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, false)?;
                let name = task.data.name.clone();
                state.index_remove(task.data.name.clone(), task.data.id);
                task.delete()?;

                println!("Deleted task {} (ID: {})", colour::task_name(&name), colour::id(&id.to_string()));
            },
            View { id_or_name } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, true)?;
                task.display()?;
            },
            Edit { id_or_name, info } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                if info {
                    edit::edit_info(id, vault_folder.clone(), &config.editor)?;
                }
                else {
                    edit::edit_raw(id, vault_folder.clone(), &config.editor, &mut state)?;
                }
                println!("Updated task {}", colour::id(&id.to_string()));
            },
            Track { id_or_name, hours, minutes } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                let entry =  tasks::TimeEntry::new(hours, minutes);
                task.data.time_entries.push(entry);
                task.save()?;
            },
            Discard { id_or_name } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                task.data.discarded = true;
                task.save()?;
                println!("Discarded task {}", colour::id(&id.to_string()));
            },
            Complete { id_or_name } => {
                let id = state.name_or_id_to_id(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                task.data.complete = true;
                task.save()?;
                println!("Marked task {} as complete", colour::id(&id.to_string()));
            },
            List {} => {
                tasks::list(vault_folder)?;
            },
            Clean => {
                tasks::clean(vault_folder)?;
                println!("Deleted all discarded tasks");
            }
            // All commands which are dealt with in if let chain at start.
            Vault(_) | Config(_) | Git { args : _ } | Switch { name : _ } | GitIgnore => unreachable!(),
        }

        state.save()?;
    }

    config.save()?;

    Ok(())
}
