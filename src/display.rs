use crate::ecam::EcamStatus;
use std::io::Write;

pub struct BasicDisplay {
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

impl BasicDisplay {
    pub fn new(width: usize) -> Self {
        BasicDisplay { activity: 0, width }
    }

    pub fn display(&mut self, state: EcamStatus) {
        let (bar, percent) = match state {
            EcamStatus::Ready => ("Ready", None),
            EcamStatus::StandBy => ("Standby", None),
            EcamStatus::TurningOn(percent) => ("Turning on...", Some(percent)),
            EcamStatus::Busy(percent) => ("Dispensing...", Some(percent)),
        };

        self.activity = (self.activity + 1) % 8;
        print!(
            "\r{} {}",
            make_bar(bar, self.width - 2, percent),
            "/-\\|/-\\|"[self.activity as usize..self.activity as usize + 1].to_owned()
        );

        std::io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::make_bar;

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
}
