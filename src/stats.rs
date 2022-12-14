use crate::tasks;
use crate::error;

use std::path;
use std::collections::BTreeMap;
use chrono::SubsecRound;

pub fn completed_tasks(days : u16, vault_folder : &path::Path) -> Result<(), error::Error> {
    let tasks = tasks::Task::load_all(vault_folder, true)?;
    
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_header(vec!["Task", "Completed"]);

    for task in tasks {
        if let Some(completed_date) = task.data.completed {
            let time_diff = chrono::Local::now().naive_local() - completed_date;
            if time_diff < chrono::Duration::days(i64::from(days)) && time_diff > chrono::Duration::zero() {
                table.add_row(vec![
                    task.data.name.clone(),
                    completed_date.round_subsecs(0).to_string()
                ]);
            }
        }
    }

    println!("{}", table);

    Ok(())
}

pub fn time_per_tag(days : u16, vault_folder : &path::Path) -> Result<(), error::Error> {

    let tasks = tasks::Task::load_all(vault_folder, true)?;

    let mut times = BTreeMap::<String, tasks::Duration>::new();

    for task in &tasks {
        let mut time = tasks::Duration::zero();

        for entry in &task.data.time_entries {
            if chrono::Local::now().naive_local().date() - entry.logged_date < chrono::Duration::days(i64::from(days)) {
                time = time + entry.duration;
            }
        }

        let tag_count = task.data.tags.len();
        let time_per_tag = time / tag_count;

        for tag in &task.data.tags {
            match times.get_mut(tag) {
                Some(time) => {
                    *time = *time + time_per_tag;
                },
                None => {
                    times.insert(tag.clone(), time_per_tag);
                }
            }
        }
    }

    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_header(vec!["Tag", "Time"]);


    let mut total_duration = tasks::Duration::zero();
    for (tag, duration) in times {
        table.add_row(
            vec![
                tag.clone(),
                duration.to_string(),
            ]
        );

        total_duration = total_duration + duration;
    }

    table.add_row(
        vec![
            String::from("Total"),
            total_duration.to_string(),
        ]
    );

    println!("{}", table);

    Ok(())
}
