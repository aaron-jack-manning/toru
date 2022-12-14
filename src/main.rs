mod vcs;
mod edit;
mod args;
mod list;
mod vault;
mod index;
mod error;
mod tasks;
mod state;
mod graph;
mod stats;
mod config;
mod format;

use args::*;

fn main() {
    let result = program();

    match result {
        Ok(()) => {
            std::process::exit(0);
        },
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    }
}

fn program() -> Result<(), error::Error> {
    let command = Args::accept_command();

    let mut config = config::Config::load()?;

    if let Command::Vault(command) = command {
        match command {
            VaultCommand::New { name, path } => {
                vault::new(name.clone(), path, &mut config)?;
                println!("Created vault {}", format::vault(&name));
            },
            VaultCommand::Disconnect { name } => {
                vault::disconnect(&name, &mut config)?;
                println!("Disconnected vault {}", format::vault(&name));
            },
            VaultCommand::Connect { name , path } => {
                vault::connect(name.clone(), path, &mut config)?;
                println!("Connected vault {}", format::vault(&name));
            },
            VaultCommand::Delete { name } => {
                vault::delete(&name, &mut config)?;
                println!("Deleted vault {}", format::vault(&name));
            },
            VaultCommand::List => {
                config.list_vaults()?;
            },
            VaultCommand::Rename { old_name, new_name } => {
                config.rename_vault(&old_name, new_name.clone())?;
                println!("Renamed vault {} to {}", format::vault(&old_name), format::vault(&new_name));
            }
        }
    }
    else if let Command::Config(command) = command {
        match command {
            ConfigCommand::Editor { editor } => {
                match editor {
                    Some(editor) => {
                        config.editor = editor;
                        println!("Updated editor command to: {}", config.editor);
                    },
                    None => {
                        println!("Current editor command: {}", config.editor);
                    }
                }
            },
            ConfigCommand::Profile(command) => {
                match command {
                    ProfileCommand::New { name, options } => {
                        config.create_profile(name.clone(), options)?;
                        println!("Created profile {}", format::profile(&name))
                    },
                    ProfileCommand::Delete { name } => {
                        config.delete_profile(&name)?;
                        println!("Deleted profile {}", format::profile(&name))
                    },
                    ProfileCommand::List => {
                        config.list_profiles()?;
                    }
                }
            }
        }
    }
    else if let Command::Switch { name } = command {
        config.switch(&name)?;
        println!("Switched to vault {}", format::vault(&name));
    }
    else if let Command::Git { args } = command {
        let vault_folder = &config.current_vault()?.1;
        vcs::command(args, vcs::Vcs::Git, vault_folder)?;
    }
    else if let Command::Svn { args } = command {
        let vault_folder = &config.current_vault()?.1;
        vcs::command(args, vcs::Vcs::Svn, vault_folder)?;
    }
    else if command == Command::GitIgnore {
        let vault_folder = &config.current_vault()?.1;
        vcs::create_gitignore(vault_folder)?;
        println!("Default {} file created", format::file(".gitignore"));
    }
    else if command == Command::SvnIgnore {
        let vault_folder = &config.current_vault()?.1;
        vcs::set_svn_ignore(vault_folder)?;
        println!("Default svn:ignore property set");
    }
    // Commands that require loading in the state.
    else {
        let vault_folder = &config.current_vault()?.1;
        let mut state = state::State::load(vault_folder)?;

        match command {
            Command::New { name, info, tag, dependency, priority, due } => {
                let id = tasks::Task::new(name.clone(), info, tag, dependency, priority, due, vault_folder, &mut state)?;
                println!("Created task {} (ID: {})", format::task(&name), format::id(id));
            },
            Command::Delete { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, false)?;
                let name = task.data.name.clone();
                state.data.index.remove(task.data.name.clone(), task.data.id);
                // Removing the task from others which list it as a dependency.
                if let (true, dependents) = state.data.deps.remove_node(task.data.id) {
                    for dependent in dependents {
                        let mut task = tasks::Task::load(dependent, vault_folder, false)?;
                        task.data.dependencies.remove(&id);
                        task.save()?;
                    }
                }
                task.delete()?;

                println!("Deleted task {} (ID: {})", format::task(&name), format::id(id));
            },
            Command::View { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let task = tasks::Task::load(id, vault_folder, true)?;
                task.display(vault_folder, &state)?;
            },
            Command::Edit { id_or_name, info } => {
                let id = state.data.index.lookup(&id_or_name)?;
                if info {
                    edit::edit_info(id, vault_folder.clone(), &config.editor)?;
                }
                else {
                    edit::edit_raw(id, vault_folder.clone(), &config.editor, &mut state)?;
                }
                println!("Updated task {}", format::id(id));
            },
            Command::Track { id_or_name, duration, date, message } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                let entry =  tasks::TimeEntry::new(duration, date, message);
                task.data.time_entries.push(entry);
                task.save()?;
            },
            Command::Stats(command) => {
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
            Command::Complete { id_or_name } => {
                let id = state.data.index.lookup(&id_or_name)?;
                let mut task = tasks::Task::load(id, vault_folder, false)?;
                task.data.completed = Some(chrono::Local::now().naive_local());
                task.save()?;
                println!("Marked task {} as complete", format::id(id));
            },
            Command::List { profile : profile_name, options : additional } => {
                let options = match profile_name {
                    Some(profile_name) => {
                        let profile = config.get_profile(&profile_name)?;
                        ListOptions::combine(profile, &additional)
                    },
                    None => {
                        additional
                    }
                };
                list::list(options, vault_folder, &state)?;
            },
            // All commands which are dealt with in if let chain at start.
            Command::Vault(_) | Command::Config(_) | Command::Git { args : _ } | Command::Svn { args : _ } | Command::Switch { name : _ } | Command::GitIgnore | Command::SvnIgnore => unreachable!(),
        }

        state.save()?;
    }

    config.save()?;

    Ok(())
}
