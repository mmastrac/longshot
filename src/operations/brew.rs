use super::{IngredientInfo, RecipeDetails};
use crate::protocol::*;

pub struct BrewIngredients {
    pub beverage: EcamBeverageId,
    pub coffee: Option<u16>,
    pub milk: Option<u16>,
    pub hotwater: Option<u16>,
    pub taste: Option<EcamBeverageTaste>,
    pub temp: Option<EcamTemperature>,
    pub allow_defaults: bool,
}

fn get_u16_arg(
    ingredient: &str,
    allow_defaults: bool,
    arg: Option<u16>,
    min: u16,
    value: u16,
    max: u16,
) -> Result<u16, String> {
    if let Some(arg) = arg {
        if arg.clamp(min, max) != arg {
            return Err(format!("{} out of valid range", ingredient));
        }
        Ok(arg)
    } else {
        if allow_defaults {
            Ok(value)
        } else {
            Err(format!("{} required for this beverage", ingredient))
        }
    }
}

fn get_enum_arg<T>(
    ingredient: &str,
    allow_defaults: bool,
    arg: Option<T>,
    default: T,
) -> Result<T, String> {
    if let Some(arg) = arg {
        Ok(arg)
    } else {
        if allow_defaults {
            Ok(default)
        } else {
            Err(format!("{} required for this beverage", ingredient))
        }
    }
}

pub fn check_ingredients(
    ingredients: &BrewIngredients,
    details: &RecipeDetails,
) -> Result<Vec<RecipeInfo>, String> {
    let mut v = vec![];
    for ingredient in details.fetch_ingredients() {
        match ingredient {
            IngredientInfo::Coffee(min, value, max) => {
                let coffee = get_u16_arg(
                    "Coffee",
                    ingredients.allow_defaults,
                    ingredients.coffee,
                    min,
                    value,
                    max,
                )?;
                v.push(RecipeInfo::new(EcamIngredients::Coffee, coffee))
            }
            IngredientInfo::HotWater(min, value, max) => {
                let hotwater = get_u16_arg(
                    "Hot water",
                    ingredients.allow_defaults,
                    ingredients.hotwater,
                    min,
                    value,
                    max,
                )?;
                v.push(RecipeInfo::new(EcamIngredients::HotWater, hotwater))
            }
            IngredientInfo::Taste(default) => {
                let taste = get_enum_arg(
                    "Taste",
                    ingredients.allow_defaults,
                    ingredients.taste,
                    default,
                )?;
                v.push(RecipeInfo::new(EcamIngredients::Taste, taste as u8 as u16));
            }
            _ => {}
        }
    }
    Ok(v)
}
