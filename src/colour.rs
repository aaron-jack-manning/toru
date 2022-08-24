use colored::Colorize;

// Yellow
pub fn vault(text : &str) -> colored::ColoredString {
    text.truecolor(243, 156, 18).bold()
}

// Red
pub fn error(text : &str) -> colored::ColoredString {
    text.truecolor(192, 57, 43).bold()
}

// Purple
pub fn command(text : &str) -> colored::ColoredString {
    text.truecolor(155, 89, 182).bold()
}

// Green
pub fn task_name(text : &str) -> colored::ColoredString {
    text.truecolor(39, 174, 96).bold()
}

// Beige
pub fn file(text : &str) -> colored::ColoredString {
    text.truecolor(255, 184, 184).bold()
}

// Blue
pub fn id(text : &str) -> colored::ColoredString {
    text.truecolor(52, 152, 219)
}

// Grey
pub fn greyed_out(text : &str) -> colored::ColoredString {
    text.truecolor(99, 110, 114)
}


pub mod due_date {
    use colored::Colorize;

    pub fn overdue(text : &str) -> colored::ColoredString {
        text.truecolor(192, 57, 43)
    }

    pub fn very_close(text : &str) -> colored::ColoredString {
        text.truecolor(231, 76, 60)

    }

    pub fn close(text : &str) -> colored::ColoredString {
        text.truecolor(230, 126, 34)

    }

    pub fn plenty_of_time(text : &str) -> colored::ColoredString {
        text.truecolor(46, 204, 113)
    }
}
