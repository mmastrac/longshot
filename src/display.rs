//! Status display utilities.

use crate::ecam::EcamStatus;
use colored::*;
use std::io::Write;

pub trait StatusDisplay {
    fn display(&mut self, state: EcamStatus);
}

pub struct ColouredStatusDisplay {
    activity: usize,
    width: usize,
}

impl ColouredStatusDisplay {
    pub fn new(width: usize) -> Self {
        Self { activity: 0, width }
    }
}

impl StatusDisplay for ColouredStatusDisplay {
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
    }
}

pub struct BasicStatusDisplay {
    activity: u8,
    width: usize,
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
        Self { activity: 0, width }
    }
}

impl StatusDisplay for BasicStatusDisplay {
    fn display(&mut self, state: EcamStatus) {
        let (bar, percent) = match state {
            EcamStatus::Ready => ("Ready".to_owned(), None),
            EcamStatus::StandBy => ("Standby".to_owned(), None),
            EcamStatus::TurningOn(percent) => ("Turning on...".to_owned(), Some(percent)),
            EcamStatus::ShuttingDown(percent) => ("Shutting down...".to_owned(), Some(percent)),
            EcamStatus::Busy(percent) => ("Dispensing...".to_owned(), Some(percent)),
            EcamStatus::Alarm(alarm) => (format!("Alarm: {:?}", alarm), None),
        };

        self.activity = (self.activity + 1) % 8;
        print!(
            "\r{} {}",
            make_bar(&bar, self.width - 2, percent),
            "/-\\|/-\\|"[self.activity as usize..self.activity as usize + 1].to_owned()
        );

        std::io::stdout().flush().unwrap();
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
