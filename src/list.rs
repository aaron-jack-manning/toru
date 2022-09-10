use crate::args;
use crate::error;
use crate::state;
use crate::tasks;
use crate::format;
use crate::tasks::Id;

use std::cmp;
use std::path;
use std::collections::HashSet;
use chrono::SubsecRound;

impl args::ListOptions {
    /// Combines list options coming from a profile and from the additional arguments given. Order
    /// of the arguments provided matters, hence the argument names (because optional arguments
    /// from the profile are overwritten by the additional arguments).
    pub fn combine(profile : &Self, additional : &Self) -> Self {
        /// Joins two vectors together one after the other, creating a new allocation.
        fn concat<T : Clone>(a : &Vec<T>, b : &Vec<T>) -> Vec<T> {
            let mut a = a.clone();
            a.extend(b.iter().cloned());
            a
        }

        /// Takes two options, and prioritises the second if it is provided in the output, using
        /// the first as a fallback, and returning None if both are None.
        fn join_options<T : Clone>(a : &Option<T>, b : &Option<T>) -> Option<T> {
            match (a, b) {
                (Some(_), Some(b)) => Some(b.clone()),
                (Some(a), None) => Some(a.clone()),
                (None, Some(b)) => Some(b.clone()),
                (None, None) => None,
            }
        }

        Self {
            column : concat(&profile.column, &additional.column),
            order_by : join_options(&profile.order_by, &additional.order_by),
            order : join_options(&profile.order, &profile.order),
            tag : concat(&profile.tag, &additional.tag),
            exclude_tag : concat(&profile.exclude_tag, &additional.exclude_tag),
            priority : concat(&profile.priority, &additional.priority),
            due_before : join_options(&profile.due_before, &additional.due_before),
            due_after : join_options(&profile.due_after, &additional.due_after),
            created_before : join_options(&profile.created_before, &additional.created_before),
            created_after : join_options(&profile.created_after, &additional.created_after),
            include_completed : profile.include_completed || additional.include_completed,
            no_dependencies : profile.no_dependencies || additional.no_dependencies,
            no_dependents : profile.no_dependents || additional.no_dependents,
        }
    }
}

/// Lists all tasks in the specified vault.
pub fn list(mut options : args::ListOptions, vault_folder : &path::Path, state : &state::State) -> Result<(), error::Error> {

    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);


    let tasks = tasks::Task::load_all(vault_folder, true)?;

    // Collect the Ids of completed tasks for the sake of checking if a task has no incomplete dependencies.
    let completed_ids : HashSet<Id> = tasks.iter().filter_map(|t| if t.data.completed.is_some() { Some(t.data.id) } else { None }).collect();

    let mut tasks : Box<dyn Iterator<Item = tasks::Task>> = Box::new(tasks.into_iter());

    // Filter the tasks.
    if let Some(date) = options.created_before {
        tasks = Box::new(tasks.filter(move |t| t.data.created.date() <= date));
    }
    if let Some(date) = options.created_after {
        tasks = Box::new(tasks.filter(move |t| t.data.created.date() >= date));
    }

    if let Some(date) = options.due_before {
        tasks = Box::new(tasks.filter(move |t| {
            match tasks::compare_due_dates(&t.data.due.map(|d| d.date()), &Some(date)) {
                cmp::Ordering::Less | cmp::Ordering::Equal => true,
                cmp::Ordering::Greater => false,
            }
        }));
    }
    if let Some(date) = options.due_after {
        tasks = Box::new(tasks.filter(move |t| {
            match tasks::compare_due_dates(&t.data.due.map(|d| d.date()), &Some(date)) {
                cmp::Ordering::Greater | cmp::Ordering::Equal => true,
                cmp::Ordering::Less => false,
            }
        }));
    }

    if !options.include_completed {
        tasks = Box::new(tasks.filter(|t| t.data.completed.is_none()));
    }

    if !options.tag.is_empty() {
        let specified_tags : HashSet<_> = options.tag.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            let task_tags : HashSet<_> = t.data.tags.iter().collect();

            // Non empty intersection of tags means the task should be displayed
            specified_tags.intersection(&task_tags).next().is_some()
        }));
    }

    if !options.exclude_tag.is_empty() {
        let specified_tags : HashSet<_> = options.exclude_tag.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            let task_tags : HashSet<_> = t.data.tags.iter().collect();

            // If the task contains a tag which was supposed to be excluded, it should be filtered
            // out
            !specified_tags.intersection(&task_tags).next().is_some()
        }));
    }

    if !options.priority.is_empty() {
        let specified_priority_levels : HashSet<_> = options.priority.iter().collect();

        tasks = Box::new(tasks.filter(move |t| {
            specified_priority_levels.contains(&t.data.priority)
        }));
    }

    // Checks that a task has no incomplete dependencies.
    if options.no_dependencies {
        tasks = Box::new(tasks.filter(move |t| {
            // Get all dependencies (including indirect ones).
            let all_dependencies = state.data.deps.get_nested_deps(t.data.id);
            // Check that all of those dependencies are completed.
            all_dependencies.iter().all(|d| completed_ids.contains(&d))
        }));
    }

    if options.no_dependents {
        let tasks_with_dependents = state.data.deps.get_tasks_with_dependents();

        tasks = Box::new(tasks.filter(move |t| {
            !tasks_with_dependents.contains(&t.data.id)
        }));
    }

    let mut tasks : Vec<_> = tasks.collect();

    // Sort the tasks.
    use super::{OrderBy, Order};
    match options.order_by.unwrap_or_default() {
        OrderBy::Id => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.id.cmp(&t2.data.id));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.id.cmp(&t1.data.id));
                },
            }
        },
        OrderBy::Name => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.name.cmp(&t2.data.name));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.name.cmp(&t1.data.name));
                },
            }
        },
        OrderBy::Due => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| tasks::compare_due_dates(&t1.data.due, &t2.data.due));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| tasks::compare_due_dates(&t2.data.due, &t1.data.due));
                },
            }
        },
        OrderBy::Priority => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.priority.cmp(&t2.data.priority));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.priority.cmp(&t1.data.priority));
                },
            }
        },
        OrderBy::Created => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| t1.data.created.cmp(&t2.data.created));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| t2.data.created.cmp(&t1.data.created));
                },
            }
        },
        OrderBy::Tracked => {
            match options.order.unwrap_or_default() {
                Order::Asc => {
                    tasks.sort_by(|t1, t2| tasks::TimeEntry::total(&t1.data.time_entries).cmp(&tasks::TimeEntry::total(&t2.data.time_entries)));
                },
                Order::Desc => {
                    tasks.sort_by(|t1, t2| tasks::TimeEntry::total(&t2.data.time_entries).cmp(&tasks::TimeEntry::total(&t1.data.time_entries)));
                },
            }
        }
    }

    // Include the required columns
    let mut headers = vec!["Id", "Name"];

    // Remove duplicate columns.
    options.column = {
        let mut columns = HashSet::new();

        options.column.clone()
            .into_iter()
            .filter(|c| {
                if columns.contains(c) {
                    false
                }
                else {
                    columns.insert(c.clone());
                    true
                }
            })
            .collect()
    };
    
    use super::Column;
    for column in &options.column {
        match column {
            Column::Tracked => {
                headers.push("Tracked");
            },
            Column::Due => {
                headers.push("Due");
            },
            Column::Tags => {
                headers.push("Tags");
            },
            Column::Priority => {
                headers.push("Priority");
            },
            Column::Status => {
                headers.push("Status");
            },
            Column::Created => {
                headers.push("Created");
            },
        }
    }

    table.set_header(headers);

    for task in tasks {

        use comfy_table::Cell;
        let mut row = vec![Cell::from(task.data.id), Cell::from(task.data.name)];

        for column in &options.column {
            match column {
                Column::Tracked => {
                    let duration = tasks::TimeEntry::total(&task.data.time_entries);
                    row.push(
                        Cell::from(if duration == tasks::Duration::zero() { String::new() } else { duration.to_string() })
                    );
                },
                Column::Due => {
                    row.push(match task.data.due {
                        Some(due) => format::cell::due_date(&due, task.data.completed.is_none()),
                        None => Cell::from(String::new())
                    });
                },
                Column::Tags => {
                    row.push(Cell::new(format::hash_set(&task.data.tags)?));
                },
                Column::Priority => {
                    row.push(format::cell::priority(&task.data.priority));
                },
                Column::Status => {
                    row.push(
                        Cell::new(if task.data.completed.is_some() {
                            String::from("complete")
                        }
                        else {
                            String::from("incomplete")
                        })
                    );
                },
                Column::Created => {
                    row.push(Cell::new(task.data.created.round_subsecs(0).to_string()));
                },
            }
        }

        table.add_row(row);
    }

    println!("{}", table);

    Ok(())
}
