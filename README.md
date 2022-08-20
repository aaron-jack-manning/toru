# Toru

A (currently in development) to do app for the command line.

## Planned Features and Changes:
- Options for editing additional config
    - `config`
    - `editor` subcommand for setting default text editor
- Editing individual tasks directly (command: `edit`)
    - Create temporary file for the data
    - Fork process to open the text editor
    - Wait for process to return
    - Open, read and then delete the temporary file
    - Deserialize as a map so each value can be checked and useful errors reported
- Listing tasks in vault (command: `list`)
    - Options for which field to order by, and how to order (ascending or descending)
    - Options for which columns to include
    - If no values given, read a set of defaults from a `list.toml` file, which can be edited from a similar command
- Ability to view, edit, delete, etc. using name
    - Have a file containing a serialized `HashMap<String, Vec<Id>>`
    - Disallow numerical names and have command automatically identify if it is a name or Id
    - Error on operation if two tasks exist with the same name
- Dependency tracker
    - Store dependencies in a file and correctly update them upon creation and removal of notes
    - Error if any circular dependencies are introduced
    - Make sure dependencies written to file are only those that could be successfully created
- Automatically added recurring notes
