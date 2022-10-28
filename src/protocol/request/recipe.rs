use crate::protocol::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RecipeInfo {
    pub ingredient: MachineEnum<EcamIngredients>,
    pub value: u16,
}

impl PartialDecode<RecipeInfo> for RecipeInfo {
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let ingredient = <MachineEnum<EcamIngredients>>::partial_decode(input)?;
        if let MachineEnum::Value(known) = ingredient {
            match known {
                EcamIngredients::Temp
                | EcamIngredients::Taste
                | EcamIngredients::Inversion
                | EcamIngredients::DueXPer
                | EcamIngredients::IndexLength
                | EcamIngredients::Visible
                | EcamIngredients::Accessorio => {
                    return Some(RecipeInfo {
                        ingredient,
                        value: <u8>::partial_decode(input)? as u16,
                    });
                }
                EcamIngredients::Coffee | EcamIngredients::Milk | EcamIngredients::HotWater => {
                    let a = <u8>::partial_decode(input)? as u16;
                    let b = <u8>::partial_decode(input)? as u16;
                    return Some(RecipeInfo {
                        ingredient,
                        value: (a << 8) | b,
                    });
                }
                _ => {}
            }
        }
        panic!("Unhandled ingredient {:?}", ingredient);
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
