use crate::protocol::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RecipeInfo {
    pub ingredient: MachineEnum<EcamIngredients>,
    pub value: u16,
}

impl RecipeInfo {
    pub fn new(ingredient: EcamIngredients, value: u16) -> Self {
        RecipeInfo {
            ingredient: ingredient.into(),
            value,
        }
    }
}

impl PartialDecode<RecipeInfo> for RecipeInfo {
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let ingredient = <MachineEnum<EcamIngredients>>::partial_decode(input)?;
        if let MachineEnum::Value(known) = ingredient {
            if known.is_wide_encoding().expect("Unknown encoding") {
                let a = <u8>::partial_decode(input)? as u16;
                let b = <u8>::partial_decode(input)? as u16;
                return Some(RecipeInfo {
                    ingredient,
                    value: (a << 8) | b,
                });
            } else {
                return Some(RecipeInfo {
                    ingredient,
                    value: <u8>::partial_decode(input)? as u16,
                });
            }
        }
        panic!("Unhandled ingredient {:?}", ingredient);
    }
}

impl PartialEncode for RecipeInfo {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push(self.ingredient.into());
        let ingredient: Option<EcamIngredients> = self.ingredient.into();
        if ingredient
            .and_then(|x| x.is_wide_encoding())
            .expect("Unknown encoding")
        {
            out.push((self.value >> 8) as u8);
        }
        out.push(self.value as u8);
    }
}

impl PartialDecode<Vec<RecipeInfo>> for Vec<RecipeInfo> {
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let mut v = vec![];
        while !input.is_empty() {
            v.push(<RecipeInfo>::partial_decode(input)?);
        }
        Some(v)
    }
}
