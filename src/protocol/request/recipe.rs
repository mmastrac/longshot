use crate::protocol::*;

/// Recipe information returned from [`Request::RecipeQuantityRead`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeInfo<T> {
    pub ingredient: MachineEnum<EcamIngredients>,
    pub value: T,
}

impl<T: Copy + Clone + Eq + PartialEq> RecipeInfo<T> {
    pub fn new(ingredient: EcamIngredients, value: T) -> Self {
        RecipeInfo {
            ingredient: ingredient.into(),
            value,
        }
    }
}

impl PartialDecode<RecipeInfo<u16>> for RecipeInfo<u16> {
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

impl PartialEncode for RecipeInfo<u16> {
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

/// Recipe information returned from [`Request::RecipeQuantityRead`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl PartialEncode for RecipeMinMaxInfo {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push(self.ingredient.into());
        let ingredient: Option<EcamIngredients> = self.ingredient.into();
        if ingredient
            .and_then(|x| x.is_wide_encoding())
            .expect("Unknown encoding")
        {
            self.min.partial_encode(out);
            self.value.partial_encode(out);
            self.max.partial_encode(out);
        } else {
            (self.min as u8).partial_encode(out);
            (self.value as u8).partial_encode(out);
            (self.max as u8).partial_encode(out);
        }
    }
}
