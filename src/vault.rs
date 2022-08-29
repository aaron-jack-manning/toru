use crate::error;
use crate::state;
use crate::colour;
use crate::config;

use std::fs;
use std::path;

pub fn new(name : String, path : path::PathBuf, config : &mut config::Config) -> Result<(), error::Error> {

    fn create_all_metadata(path : &path::Path) -> Result<(), error::Error> {
        fs::create_dir(path.join("notes"))?;
        let _ = state::State::load(path)?;

        Ok(())
    }

    // Configuration already contains a vault by the given name.
    if config.contains_name(&name) {
        Err(error::Error::Generic(format!("A vault named \"{}\" already exists", name)))
    }
    else if config.contains_path(&path) {
        Err(error::Error::Generic(format!("A vault at the path {:?} already exists", path)))
    }
    else {
        // Folder exists and contains data.
        if path.exists() && path.is_dir() && path.read_dir()?.next().is_some() {
            Err(error::Error::Generic(String::from("The specified folder already exists and contains other data, please provide a path to a new or empty folder")))
        }
        // Folder exists and is empty, so set up the vault metadata.
        else if path.exists() && path.is_dir() {
            
            // Create the vault metadata.
            create_all_metadata(&path)?;

            config.add(name, path);

            Ok(())
        }
        // Provided path is to a file, not a directory.
        else if path.exists() {
            Err(error::Error::Generic(String::from("The specified path already points to a file, please provide a path to a new or empty folder")))
        }
        // Path does not yet exist, and should be created.
        else {
            fs::create_dir_all(&path)?;

            // Create the vault metadata.
            create_all_metadata(&path)?;

            config.add(name, path);

            Ok(())
        }
    }
}

pub fn connect(name : String, path : path::PathBuf, config : &mut config::Config) -> Result<(), error::Error> {
    // Configuration already contains a vault by the given name.
    if config.contains_name(&name) {
        Err(error::Error::Generic(format!("A vault named \"{}\" already exists", name)))
    }
    else if config.contains_path(&path) {
        Err(error::Error::Generic(format!("A vault at the path {:?} is already set up", path)))
    }
    else {
        // Folder exists and contains data.
        if path.exists() && path.is_dir()  {
            // Vault is missing required metadata files.
            if !path.join("notes").exists() {
                Err(error::Error::Generic(format!("Cannot connect the vault as it is missing the {} folder", colour::text::file("notes"))))
            }
            else if !path.join("state.toml").exists() {
                Err(error::Error::Generic(format!("Cannot connect the vault as it is missing the {} file", colour::text::file("state.toml"))))
            }
            // Required metadata exists, so the vault is connected.
            else {
                config.add(name, path);

                Ok(())
            }
        }
        // Provided path is to a file, not a directory.
        else if path.exists() {
            Err(error::Error::Generic(String::from("The specified path points to a file, not a folder")))
        }
        // Path does not yet exist.
        else {
            Err(error::Error::Generic(format!("The path {:?} does not exist", path)))
        }
    }
}

pub fn disconnect(name : &String, config : &mut config::Config) -> Result<(), error::Error> {
    config.remove(name)?;
    Ok(())
}

pub fn delete(name : &String, config : &mut config::Config) -> Result<(), error::Error> {
    let path = config.remove(name)?;
    trash::delete(path)?;
    Ok(())
}

