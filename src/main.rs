//#![allow(dead_code, unused_variables)]

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

#[derive(clap::Subcommand, Debug)]
#[clap(version, about, author, global_setting=clap::AppSettings::DisableHelpSubcommand)]
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
    /// Lists tasks according to the specified ordering and filters.
    List {
        // Need to have options for:
        // - column to order by
        // - ascending or descending
        // - which columns to include
        // - filters which exclude values
    },
    /// Commands for interacting with vaults.
    #[clap(subcommand)]
    Vault(VaultCommand),
}

#[derive(clap::Subcommand, Debug)]
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
    /// Switches to the specified vault.
    Switch {
        name : String,
    },
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
    match command {
        Vault(command) => {
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
                Switch { name } => {
                    config.switch(&name)?;
                    println!("Switched to vault {}", colour::vault(&name));
                },
            }
        }
        command => {
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
                        edit::edit_info(id, vault_folder.clone(), "nvim")?;
                    }
                    else {
                        edit::edit_raw(id, vault_folder.clone(), "nvim", &mut state)?;
                    }
                    println!("Updated task {}", colour::id(&id.to_string()));
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
                Git { args } => {
                    git::run_command(args, vault_folder)?;
                },
                List {} => {
                    tasks::list(vault_folder)?;
                }
                Vault(_) => unreachable!(),
            }

            state.save()?;
        }
    }

    config.save()?;

    Ok(())
}
