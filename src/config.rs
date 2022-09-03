use crate::args;
use crate::error;
use crate::format;

use std::path;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Paths for all vaults, ordered according to recent usage, with current at the front.
    pub vaults : Vec<(String, path::PathBuf)>,
    pub editor : String,
    pub profiles : Vec<Profile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    name : String,
    options : args::ListOptions,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vaults : Vec::default(),
            editor : String::from("vim"),
            profiles : Vec::default(),
        }
    }
}

impl Config {
    pub fn current_vault(&self) -> Result<&(String, path::PathBuf), error::Error> {
        self.vaults.get(0).ok_or_else(|| error::Error::Generic(String::from("The attempted operation requires a vault, none of which have been set up")))
    }

    pub fn save(self) -> Result<(), error::Error> {
        Ok(confy::store::<Config>("toru", self)?)
    }

    pub fn load() -> Result<Config, error::Error> {
        Ok(confy::load::<Config>("toru")?)
    }

    pub fn contains_name(&self, name : &String) -> bool {
        self.vaults.iter().any(|(n, _)| n == name)
    }
    
    pub fn contains_path(&self, path : &path::PathBuf) -> bool {
        self.vaults.iter().any(|(_, p)| p == path)
    }

    pub fn rename_vault(&mut self, old_name : &String, new_name : String) -> Result<(), error::Error> {
        let mut to_change = None;

        for (name, _) in &mut self.vaults {
            if *name == new_name {
                return Err(error::Error::Generic(format!("A vault named {} already exists", format::vault(&new_name))));
            }

            if name == old_name {
                to_change = Some(name);
            }
        }

        match to_change {
            Some(name) => {
                *name = new_name;
                Ok(())
            },
            None => {
                Err(error::Error::Generic(format!("No vault named {} exists", format::vault(old_name))))
            }
        }
    }

    /// Adds the vault to the configuration.
    pub fn add(&mut self, name : String, path : path::PathBuf) {
        debug_assert!(!self.contains_name(&name));
        debug_assert!(!self.contains_path(&path));

        self.vaults.push((name, path));
    }

    pub fn remove(&mut self, name : &String) -> Result<path::PathBuf, error::Error> {
        match self.vaults.iter().position(|(n, _)| n == name) {
            Some(index) => {
                let (_, path) = self.vaults.swap_remove(index);
                Ok(path)
            },
            None => {
                Err(error::Error::Generic(format!("No vault by the name {} exists", format::vault(name))))
            }
        }
    }

    pub fn switch(&mut self, name : &String) -> Result<(), error::Error> {
        match self.vaults.iter().position(|(n, _)| n == name) {
            Some(index) => {
                self.vaults.swap(index, 0);
                Ok(())
            },
            None => {
                Err(error::Error::Generic(format!("No vault by the name {} exists", format::vault(name))))
            }
        }
    }

    /// Lists all vaults to stdout.
    pub fn list_vaults(&self) -> Result<(), error::Error> {

        let width = self.vaults.iter().fold(usize::MIN, |c, (n, _)| c.max(n.len()));

        if self.vaults.is_empty() {
            Err(error::Error::Generic(format!("No vaults currently set up, try running: {}", format::command("toru vault new <NAME> <PATH>"))))
        }
        else {
            for (i, (name, path)) in self.vaults.iter().enumerate() {

                if i == 0 {
                    print!("* ");
                }
                else {
                    print!("  ");
                }

                print!("{}", format::vault(name));

                let padding = width - name.len() + 1;

                for _ in 0..padding {
                    print!(" ")
                }

                print!("{}", path.display());

                println!();
            }

            Ok(())
        }
    }
    
    pub fn create_profile(&mut self, name : String, options : args::ListOptions) -> Result<(), error::Error> {
        if self.profiles.iter().any(|Profile { name : n, options : _ }| n == &name) {
            Err(error::Error::Generic(format!("A profile by the name {} already exists", format::profile(&name))))
        }
        else {
            self.profiles.push(Profile { name, options });
            Ok(())
        }
    }

    pub fn get_profile(&self, name : &String) -> Result<&args::ListOptions, error::Error> {
        self.profiles
            .iter()
            .find(|Profile { name : n, options : _ }| n == name)
            .map(|Profile { name : _, options : o }| o)
            .ok_or(error::Error::Generic(format!("No profile by the name {} exists", format::profile(name))))
    }

    pub fn delete_profile(&mut self, name : &String) -> Result<(), error::Error> {
        match self.profiles.iter().position(|Profile { name : n, options : _ }| n == name) {
            Some(index) => {
                let _ = self.profiles.swap_remove(index);
                Ok(())
            },
            None => {
                Err(error::Error::Generic(format!("No profile by the name {} exists", format::profile(name))))
            }
        }
    }


    /// Lists all profiles to stdout.
    pub fn list_profiles(&self) -> Result<(), error::Error> {
        if self.profiles.is_empty() {
            Err(error::Error::Generic(format!("No profiles currently set up, try running: {}", format::command("toru config profile new <NAME> <OPTIONS>"))))
        }
        else {
            for Profile { name, options : _ } in self.profiles.iter() {
                println!("{}", format::profile(name));
            }

            Ok(())
        }
    }
}





