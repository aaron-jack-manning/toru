# Toru

A (currently in development) to do app for the command line.

---

## Design

The general idea of Toru is to have a to-do app which uses distinct, mutually exclusive vaults of tasks with configuration which is in a human readable and easy to export and import format (to completely separate personal, work, study, etc), however within a vault, to use tags and dependencies as a means of organising notes, rather than mutually exclusive folders.

For example, in a given vault, one may have a big project they are working on. This project, and all of the subtasks are listed together on the top level (and not organised according to projects). In order to conveniently organise and view tasks, use tags and dependencies, and filter searches for tasks to get the desired information. This allows you to categorise tasks even when they do not fall into any one obvious category.

---

## Installation

The easiest way to install is from [crates.io](https://crates.io/crates/toru) with cargo:

```
cargo install toru
```

Alternatively you can build from source:

```
git clone https://github.com/aaron-jack-manning/toru.git
cd toru
cargo build --release
```

which will create an executable at `/target/release/toru`.

---

## Getting Started

Simply type `toru` in terminal to display help information for each command:

```
toru 0.4.4
Aaron Manning <contact@aaronmanning.net>
A command line task manager.

USAGE:
    toru <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    complete      Mark a task as complete
    config        For making changes to global configuration
    delete        Delete a task (move file to trash)
    edit          Edit a task directly
    git           Run Git commands at the root of the vault
    gitignore     Adds the recommended .gitignore file to the vault
    list          Lists tasks according to the specified fields, ordering and filters
    new           Create a new task
    stats         For statistics about the state of your vault
    svn           Run Subversion commands at the root of the vault
    svn:ignore    Adds the recommended svn:ignore property to the top level of the vault
    switch        Switches to the specified vault
    track         For tracking time against a task
    vault         Commands for interacting with vaults
    view          Displays the specified task in detail
```

You can view any help screen by passing in the `-h` or `--help` flag, and the internal documentation is designed to make it obvious how to use Toru.

To start up you will need a vault to store tasks in, which you can create by running `toru vault new <NAME> <PATH>`.

If you ever want to view all vaults, along with which is the current one, run `toru vault list`.

Then you can run `toru new` to create your first task.

---

## Planned Changes

- Validate invariants at the point of saving, to create consistency across creating and editing notes.
- Convenient options with the edit command so that editing the raw file isn't the only option
