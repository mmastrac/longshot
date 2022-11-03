//! Status display utilities.

use crate::ecam::EcamStatus;
use atty::Stream;
use colored::*;
use std::sync::Mutex;
use std::{io::Write};
use lazy_static::lazy_static;

lazy_static! {
    static ref DISPLAY: Mutex<Option<Box<dyn StatusDisplay>>> = Mutex::new(None);
}

/// Initializes the global display based on the TERM and COLORTERM environment variables. 
pub fn initialize_display() {
    let term = std::env::var("TERM").ok();
    let colorterm = std::env::var("COLORTERM").ok();

    if term.is_none() || !atty::is(Stream::Stdout) || !atty::is(Stream::Stderr) {
        *DISPLAY.lock().expect("Failed to lock display for initialization") = Some(Box::new(NoTtyStatusDisplay::default()));
    } else if colorterm.is_some() {
        *DISPLAY.lock().expect("Failed to lock display for initialization") = Some(Box::new(ColouredStatusDisplay::new(80)));
    } else {
        *DISPLAY.lock().expect("Failed to lock display for initialization") = Some(Box::new(BasicStatusDisplay::new(80)));
    }
}

/// Displays the [`EcamStatus`] according to the current mode.
pub fn display_status(state: EcamStatus) {
    if let Ok(mut display) = DISPLAY.lock() {
        if let Some(ref mut display) = *display {
            display.display(state);
            return;
        }
    }
    println!("[default] {:?}", state);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn prefix(&self) -> &'static str {
        match self {
            LogLevel::Trace => { "[TRACE] " }
            LogLevel::Warning => { "[WARNING] " }
            LogLevel::Error => { "[ERROR] " }
            LogLevel::Info => { "" }
        }
    }
}

/// Logs the [`EcamStatus`] according to the current mode.
pub fn log(level: LogLevel, s: &str) {
    if let Ok(mut display) = DISPLAY.lock() {
        if let Some(ref mut display) = *display {
            display.log(level, s);
            return;
        }
    }
    println!("[default] {:?}", s);
}

trait StatusDisplay: Send + Sync {
    fn display(&mut self, state: EcamStatus);
    fn log(&mut self, level: LogLevel, s: &str);
}

/// [`StatusDisplay`] for basic terminals, or non-TTY stdio.
#[derive(Default)]
struct NoTtyStatusDisplay {}

impl StatusDisplay for NoTtyStatusDisplay {
    fn display(&mut self, state: EcamStatus) {
        println!("{:?}", state);
    }

    fn log(&mut self, level: LogLevel, s: &str) {
        if level == LogLevel::Info {
            println!("{}", s);
        } else {
            eprintln!("{}{}", level.prefix(), s);
        }
    }
}

struct ColouredStatusDisplay {
    activity: usize,
    width: usize,
    last_was_status: bool,
}

impl ColouredStatusDisplay {
    pub fn new(width: usize) -> Self {
        Self { activity: 0, width, last_was_status: false }
    }
}

impl StatusDisplay for ColouredStatusDisplay {
    fn log(&mut self, level: LogLevel, s: &str) {
        if std::mem::take(&mut self.last_was_status) {
            println!();
        }
        if level == LogLevel::Info {
            println!("{}", s);
        } else {
            eprintln!("{}{}", level.prefix(), s);
        }
    }

    fn display(&mut self, state: EcamStatus) {
        const BUBBLE_CHARS: [char; 5] = ['⋅', '∘', '°', 'º', '⚬'];

        self.activity += 1;

        let (percent, status_text) = match state {
            EcamStatus::Ready => (0, "✅ Ready".to_string()),
            EcamStatus::StandBy => (0, "💤 Standby".to_string()),
            EcamStatus::Busy(percent) => (percent, format!("☕ Dispensing... ({}%)", percent)),
            EcamStatus::TurningOn(percent) => (percent, format!("💡 Turning on... ({}%)", percent)),
            EcamStatus::ShuttingDown(percent) => {
                (percent, format!("🛏 Shutting down... ({}%)", percent))
            }
            EcamStatus::Alarm(alarm) => (0, format!("🔔 Alarm ({:?})", alarm)),
            EcamStatus::Fetching(percent) => {
                (percent, format!("👓 Fetching... ({}%)", percent))
            }
        };

        let mut status = " ".to_owned() + &status_text;
        let pad = " ".repeat(self.width - status.len());
        status = status + &pad;
        let temp_vec = vec![];
        if percent == 0 {
            print!(
                "\r▐{}▌ ",
                status.truecolor(153, 141, 109).on_truecolor(92, 69, 6)
            );
        } else {
            let status = status.chars().collect::<Vec<char>>();

            // This isn't super pretty but it's visually what we need and Good Enough™️
            let (left, right) = status.split_at((percent * status.len()) / 100);
            let (mid, right) = if right.len() <= 2 {
                (right, temp_vec.as_slice())
            } else {
                right.split_at(2)
            };
            let mut left = left.to_owned();
            if left.len() > 10 {
                for i in 0..2 {
                    let random = |n| (self.activity * 321 + 677 * i) % n;
                    // Pick a spot at random
                    let pos = random(left.len());
                    if pos < status_text.len() + 3 {
                        continue;
                    }
                    let (a, b) = left.split_at(pos);
                    if b[0] == ' ' {
                        let mut temp = a.to_owned();
                        temp.push(BUBBLE_CHARS[random(BUBBLE_CHARS.len())]);
                        temp.extend_from_slice(&b[1..]);
                        left = temp;
                    }
                }
            }

            print!(
                "\r▐{}{}{}▌ ",
                left.iter()
                    .collect::<String>()
                    .truecolor(183, 161, 129)
                    .on_truecolor(92, 69, 6),
                mid.iter().collect::<String>().black().on_white(),
                right.iter().collect::<String>().white().on_black()
            );
        }
        std::io::stdout().flush().unwrap();
        self.last_was_status = true;
    }
}

struct BasicStatusDisplay {
    activity: u8,
    width: usize,
    last_was_status: bool,
}

fn make_bar(s: &str, width: usize, percent: Option<usize>) -> String {
    let mut s = s.to_owned();
    if let Some(percent) = percent {
        let percent = percent.clamp(0, 100);
        s += " [";
        let remaining = width - s.len() - 1;
        let count = (remaining * percent) / 100;
        s += &"#".repeat(count);
        s += &"=".repeat(remaining - count);
        s += "]";
        s
    } else {
        // No bar, just pad w/spaces
        let pad = width - s.len();
        s + &" ".repeat(pad)
    }
}

impl BasicStatusDisplay {
    pub fn new(width: usize) -> Self {
        Self { activity: 0, width, last_was_status: false }
    }
}

impl StatusDisplay for BasicStatusDisplay {
    fn log(&mut self, level: LogLevel, s: &str) {
        if std::mem::take(&mut self.last_was_status) {
            println!();
        }
        if level == LogLevel::Info {
            println!("{}", s);
        } else {
            eprintln!("{}{}", level.prefix(), s);
        }
    }

    fn display(&mut self, state: EcamStatus) {
        let (bar, percent) = match state {
            EcamStatus::Ready => ("Ready".to_owned(), None),
            EcamStatus::StandBy => ("Standby".to_owned(), None),
            EcamStatus::TurningOn(percent) => ("Turning on...".to_owned(), Some(percent)),
            EcamStatus::ShuttingDown(percent) => ("Shutting down...".to_owned(), Some(percent)),
            EcamStatus::Busy(percent) => ("Dispensing...".to_owned(), Some(percent)),
            EcamStatus::Alarm(alarm) => (format!("Alarm: {:?}", alarm), None),
            EcamStatus::Fetching(percent) => ("Fetching...".to_owned(), Some(percent)),
        };

        self.activity = (self.activity + 1) % 8;
        print!(
            "\r{} {}",
            make_bar(&bar, self.width - 2, percent),
            "/-\\|/-\\|"[self.activity as usize..self.activity as usize + 1].to_owned()
        );

        std::io::stdout().flush().unwrap();
        self.last_was_status = true;
    }
}

#[cfg(test)]
mod test {
    use super::{make_bar, ColouredStatusDisplay, StatusDisplay};

    #[test]
    fn format_no_progress() {
        let none: Option<usize> = None;
        let test_cases = [
            // 123456789012345678901234567890123456789
            (
                "Description                             ",
                ("Description", none),
            ),
            (
                "Description [######====================]",
                ("Description", Some(25)),
            ),
            (
                "Description [#############=============]",
                ("Description", Some(50)),
            ),
            (
                "Description [###################=======]",
                ("Description", Some(75)),
            ),
            (
                "Description [##########################]",
                ("Description", Some(100)),
            ),
        ];

        for (expected, (description, progress)) in test_cases.into_iter() {
            assert_eq!(expected, make_bar(description, 40, progress));
        }
    }

    #[test]
    fn format_rich() {
        let mut display = ColouredStatusDisplay::new(60);
        for i in 0..=100 {
            display.display(crate::ecam::EcamStatus::Busy(i));
        }
    }
}
