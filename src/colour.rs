use crate::tasks::Id;

use colored::Colorize;

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

    pub fn due_date(due : &chrono::NaiveDateTime, include_fuzzy_period : bool, colour : bool) -> comfy_table::Cell {

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
            if colour {
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
                if remaining < chrono::Duration::zero() {
                    comfy_table::Cell::new(format!("{} ({} overdue)", due.round_subsecs(0), fuzzy_period))
                }
                else {
                    comfy_table::Cell::new(format!("{} ({} remaining)", due.round_subsecs(0), fuzzy_period))
                }
            }
        }
        else {
            comfy_table::Cell::new(format!("{}", due.round_subsecs(0)))
        }

    }
}

pub mod text {
    use super::*;
    use crate::tasks;

    use chrono::SubsecRound;

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

    pub fn priority(priority : &tasks::Priority) -> String {
        use tasks::Priority::*;
        let priority = match priority {
            Low => text("low", super::priority::LOW),
            Medium => text("medium", super::priority::MEDIUM),
            High => text("high", super::priority::HIGH),
        };
        format!("{}", priority)
    }


    pub fn due_date(due : &chrono::NaiveDateTime, include_fuzzy_period : bool, colour : bool) -> String {

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
            if colour {
                if remaining < chrono::Duration::zero() {
                    format!("{} {}", due.round_subsecs(0), text(&format!("({} overdue)", fuzzy_period), super::due::OVERDUE))
                }
                else if remaining < chrono::Duration::days(1) {
                    format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), super::due::VERY_CLOSE))

                }
                else if remaining < chrono::Duration::days(5) {
                    format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), super::due::CLOSE))

                }
                else {
                    format!("{} {}", due.round_subsecs(0), text(&format!("({} remaining)", fuzzy_period), super::due::PLENTY_OF_TIME))
                }
            }
            else {
                if remaining < chrono::Duration::zero() {
                    format!("{} ({} overdue)", due.round_subsecs(0), fuzzy_period)
                }
                else {
                    format!("{} ({} remaining)", due.round_subsecs(0), fuzzy_period)
                }
            }
        }
        else {
            format!("{}", due.round_subsecs(0))
        }
    }
}

