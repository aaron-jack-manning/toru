# Toru

A (currently in development) to do app for the command line.

This program is at the state where I am regularly using it, and it definately is workable. That said, it does not include all of the core features required to make it an adequate alternative to people who have a to-do system they like, and as such I am regularly making breaking changes. I don't recommend it if you want a stable to do system (yet, although I will update this message when that's no longer true) but if you like some of the ideas here and want to try it out, feedback is welcomed.

## Introduction and Design

The general idea of Toru is to have a to-do app which uses distinct, mutually exclusive vaults of tasks and configuration which is in a human readable and easy to export and import format (to completely separate personal, work, study, etc), however within a vault, to use tags and dependencies as a means of organising notes, rather than mutually exclusive folders.

For example, in a given vault, one may have a big project they are working on. This project, and all of the subtasks are listed together on the top level (and not organised according to projects). In order to conveniently organise and view tasks, use tags and dependencies, and filter searches for tasks to get the desired information. This allows you to categorise tasks even when they do not fall into any one obvious category.

To get started install by running:
```
cargo install toru
```
and then run:
```
toru vault new <NAME> <PATH>
```
to create a new vault of tasks.

Then you can run `toru new` to create your first task.

Run `--help` alongside any command to get details on what it does.

## Planned Features and Changes:

- Options to configure and customise output of `list`
    - Options for which field to order by, and how to order (ascending or descending)
    - Options for which columns to include
    - Filters, to exclude notes of a certain type
    - If no values given, read a set of defaults from a `list.toml` file, which can be edited from a similar command
- Dependency tracker
    - Store dependencies in a file and correctly update them upon creation and removal of notes
    - Error if any circular dependencies are introduced
    - Make sure dependencies written to file are only those that could be successfully created
    - List dependencies as a tree on note view below info
- Automatically added recurring notes system
- Time tracking
    - `track` command to track time on a note
    - Include time entries and totals on note view
    - Command to give statistics on time tracking (by tag, and for the last x days)
- Due dates
    - Taken as input when creating notes
    - Displayed in list view by default (with number of days remaining)
- Git integration
    - Command to add default gitignore file
- `clean` command to delete discarded tasks
