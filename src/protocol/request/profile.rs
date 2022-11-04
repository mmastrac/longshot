use super::PartialDecode;

/// Represents a recipe or profile name with an associate icon tucked into the last byte.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WideStringWithIcon {
    name: String,
    icon: u8,
}

impl WideStringWithIcon {
    #[cfg(test)]
    pub fn new(name: &str, icon: u8) -> Self {
        WideStringWithIcon {
            name: name.to_owned(),
            icon,
        }
    }
}

impl PartialDecode<WideStringWithIcon> for WideStringWithIcon {
    fn partial_decode(input: &mut &[u8]) -> Option<WideStringWithIcon> {
        let mut s = vec![];
        for _ in 0..10 {
            let b1 = <u8>::partial_decode(input)? as u16;
            let b2 = <u8>::partial_decode(input)? as u16;
            let char = char::from_u32(((b1 << 8) | b2) as u32).expect("Invalid character");
            s.push(char);
        }
        Some(WideStringWithIcon {
            name: s
                .iter()
                .collect::<String>()
                .trim_end_matches(&['\0'])
                .to_owned(),
            icon: <u8>::partial_decode(input)?,
        })
    }
}
