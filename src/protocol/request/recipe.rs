use crate::protocol::*;

#[derive(Clone, Debug, Eq, PartialEq)]
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
                return Some(RecipeInfo {
                    ingredient,
                    value: <u16>::partial_decode(input)? as u16,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeMinMaxInfo {
    pub ingredient: MachineEnum<EcamIngredients>,
    pub min: u16,
    pub value: u16,
    pub max: u16,
}

impl PartialDecode<RecipeMinMaxInfo> for RecipeMinMaxInfo {
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let ingredient = <MachineEnum<EcamIngredients>>::partial_decode(input)?;
        if let MachineEnum::Value(known) = ingredient {
            if known
                .is_wide_encoding()
                .unwrap_or_else(|| panic!("Unknown encoding for {:?}", known))
            {
                return Some(RecipeMinMaxInfo {
                    ingredient,
                    min: <u16>::partial_decode(input)?,
                    value: <u16>::partial_decode(input)?,
                    max: <u16>::partial_decode(input)?,
                });
            } else {
                return Some(RecipeMinMaxInfo {
                    ingredient,
                    min: <u8>::partial_decode(input)? as u16,
                    value: <u8>::partial_decode(input)? as u16,
                    max: <u8>::partial_decode(input)? as u16,
                });
            }
        }
        panic!("Unhandled ingredient {:?}", ingredient);
    }
}
