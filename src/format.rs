use crate::tasks;
use crate::graph;
use crate::error;
use crate::tasks::Id;

use std::fmt;
use std::path;
use std::collections::{HashSet, HashMap};
use colored::Colorize;
use chrono::SubsecRound;

// Yellow
pub static VAULT : (u8, u8, u8) = (243, 156, 18);
// Blue
pub static ID : (u8, u8, u8) = (52, 152, 219);
// Red
pub static ERROR : (u8, u8, u8) = (192, 57, 43);
// Purple
pub static COMMAND : (u8, u8, u8) = (155, 89, 182);
// Green
pub static TASK : (u8, u8, u8) = (39, 174, 96);
// Beige
pub static FILE : (u8, u8, u8) = (255, 184, 184);
// Grey
pub static GREY : (u8, u8, u8) = (99, 110, 114);
// Pink
pub static PROFILE : (u8, u8, u8) = (253, 121, 168);

mod due {
    pub static OVERDUE : (u8, u8, u8) = (192, 57, 43);
    pub static VERY_CLOSE : (u8, u8, u8) = (231, 76, 60);
    pub static CLOSE : (u8, u8, u8) = (241, 196, 15);
    pub static PLENTY_OF_TIME : (u8, u8, u8) = (46, 204, 113);
}

pub mod priority {
    pub static LOW : (u8, u8, u8) = (46, 204, 113);
    pub static MEDIUM : (u8, u8, u8) = (241, 196, 15);
    pub static HIGH : (u8, u8, u8) = (231, 76, 60);
}

fn text(string : &str, colour : (u8, u8, u8)) -> colored::ColoredString {
    string.truecolor(colour.0, colour.1, colour.2)
}

pub fn vault(string : &str) -> colored::ColoredString {
    text(string, VAULT).bold()
}

pub fn id(id : Id) -> colored::ColoredString {
    text(&id.to_string(), ID)
}

pub fn error(string : &str) -> colored::ColoredString {
    text(string, ERROR).bold()
}

pub fn command(string : &str) -> colored::ColoredString {
    text(string, COMMAND).bold()
}

pub fn task(string : &str) -> colored::ColoredString {
    text(string, TASK).bold()
}

pub fn file(string : &str) -> colored::ColoredString {
    text(string, FILE).bold()
}

pub fn greyed_out(string : &str) -> colored::ColoredString {
    text(string, GREY)
}

pub fn profile(string : &str) -> colored::ColoredString {
    text(string, PROFILE)
}

pub fn priority(priority : &tasks::Priority) -> String {
    use tasks::Priority::*;
    let priority = match priority {
        Low => text("low", priority::LOW),
        Medium => text("medium", priority::MEDIUM),
        High => text("high", priority::HIGH),
    };
    format!("{}", priority)
}

pub fn hash_set<T : fmt::Display>(set : &HashSet<T>) -> Result<String, error::Error> {
    let mut output = String::new();

    for value in set.iter() {
        fmt::write(&mut output, format_args!("{}, ", value))?;
    }

    // Remove the trailing comma and space.
    if !output.is_empty() {
        output.pop();
        output.pop();
    }

    Ok(output)
}

pub fn due_date(due : &chrono::NaiveDateTime, include_fuzzy_period : bool) -> String {

    let remaining = *due - chrono::Local::now().naive_local();

    let fuzzy_period = if remaining.num_days() != 0 {
        let days = remaining.num_days().abs();
        format!("{} day{}", days, if days == 1 {""} else {"s"})
    }
    else if remaining.num_hours() != 0 {
        let hours = remaining.num_hours().abs();
        format!("{} hour{}", hours, if hours == 1 {""} else {"s"})
    }
    else if remaining.num_minutes() != 0 {
        let minutes = remaining.num_minutes().abs();
        format!("{} minute{}", minutes, if minutes == 1 {""} else {"s"})
    }
    else {
        let seconds = remaining.num_seconds().abs();
        format!("{} second{}", seconds, if seconds == 1 {""} else {"s"})
    };

    if include_fuzzy_period {
        if remaining < chrono::Duration::zero() {
            format!("{} {}", due.round_subsecs(0), text(&format!("({} overdue)", fuzzy_period), due::OVERDUE))
        }
        else if remaining < chrono::Duration::days(1) {
            format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), due::VERY_CLOSE))

        }
        else if remaining < chrono::Duration::days(5) {
            format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), due::CLOSE))

        }
        else {
            format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), due::PLENTY_OF_TIME))
        }
    }
    else {
        format!("{}", due.round_subsecs(0))
    }
}

pub fn dependencies(start : Id, vault_folder : &path::Path, graph : &graph::Graph) -> Result<(), error::Error> {

    pub fn helper(curr : Id, prefix : &String, is_last_item : bool, graph : &graph::Graph, tasks : &HashMap<Id, tasks::Task>) -> Result<(), error::Error> {

        let next = graph.edges.get(&curr).unwrap();

        {
            let task = tasks.get(&curr).unwrap();

            let name = if task.data.completed.is_some() {
                self::greyed_out(&task.data.name)
            }
            else {
                self::task(&task.data.name)
            };

            if is_last_item {
                println!("{}└──{} (ID: {})", prefix, name, self::id(curr))
            }
            else {
                println!("{}├──{} (ID: {})", prefix, name, self::id(curr))
            }
        }

        let count = next.len();

        for (i, node) in next.iter().enumerate() {
            let new_is_last_item = i == count - 1;

            let new_prefix = if is_last_item {
                format!("{}   ", prefix)
            }
            else {
                format!("{}│  ", prefix)
            };

            helper(*node, &new_prefix, new_is_last_item, graph, tasks)?;
        }

        Ok(())
    }

    let tasks = tasks::Task::load_all_as_map(vault_folder, true)?;

    helper(start, &String::new(), true, graph, &tasks)
}





pub mod cell {
    use crate::tasks;

    use chrono::SubsecRound;

    fn cell<T : Into<comfy_table::Cell>>(text : T, colour : (u8, u8, u8)) -> comfy_table::Cell {
        text.into().fg(comfy_table::Color::from(colour))
    }

    pub fn priority(priority : &tasks::Priority) -> comfy_table::Cell {
        use tasks::Priority::*;
        match priority {
            Low => comfy_table::Cell::new("low").fg(comfy_table::Color::from(super::priority::LOW)),
            Medium => comfy_table::Cell::new("medium").fg(comfy_table::Color::from(super::priority::MEDIUM)),
            High => comfy_table::Cell::new("high").fg(comfy_table::Color::from(super::priority::HIGH)),
        }
    }

    pub fn due_date(due : &chrono::NaiveDateTime, include_fuzzy_period : bool) -> comfy_table::Cell {

        let remaining = *due - chrono::Local::now().naive_local();

        let fuzzy_period = if remaining.num_days() != 0 {
            let days = remaining.num_days().abs();
            format!("{} day{}", days, if days == 1 {""} else {"s"})
        }
        else if remaining.num_hours() != 0 {
            let hours = remaining.num_hours().abs();
            format!("{} hour{}", hours, if hours == 1 {""} else {"s"})
        }
        else if remaining.num_minutes() != 0 {
            let minutes = remaining.num_minutes().abs();
            format!("{} minute{}", minutes, if minutes == 1 {""} else {"s"})
        }
        else {
            let seconds = remaining.num_seconds().abs();
            format!("{} second{}", seconds, if seconds == 1 {""} else {"s"})
        };

        if include_fuzzy_period {
            if remaining < chrono::Duration::zero() {
                cell(format!("{} {}", due.round_subsecs(0), format!("({} overdue)", fuzzy_period)), super::due::OVERDUE)
            }
            else if remaining < chrono::Duration::days(1) {
                cell(format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period)), super::due::VERY_CLOSE)

            }
            else if remaining < chrono::Duration::days(5) {
                cell(format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period)), super::due::CLOSE)

            }
            else {
                cell(format!("{} {}", due.round_subsecs(0), format!("({} remaining)", fuzzy_period)), super::due::PLENTY_OF_TIME)
            }
        }
        else {
            comfy_table::Cell::new(format!("{}", due.round_subsecs(0)))
        }

    }
}

