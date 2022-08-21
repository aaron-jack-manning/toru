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
