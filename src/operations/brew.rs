use crate::prelude::*;
use super::{IngredientInfo, RecipeDetails};
use crate::{
    ecam::{Ecam, EcamError, EcamStatus},
    operations::{list_recipies_for, monitor},
    protocol::*,
};

/// The complete set of brewing ingredients that can be specified to dispense a beverage.
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
    } else if allow_defaults {
        Ok(value)
    } else {
        Err(format!("{} required for this beverage", ingredient))
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
    } else if allow_defaults {
        Ok(default)
    } else {
        Err(format!("{} required for this beverage", ingredient))
    }
}

fn check_ingredients(
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

pub async fn brew(
    ecam: Ecam,
    turn_on: bool,
    allow_off: bool,
    skip_brew: bool,
    ingredients: BrewIngredients,
) -> Result<(), EcamError> {
    match ecam.current_state().await? {
        EcamStatus::Ready => {}
        EcamStatus::StandBy => {
            if allow_off {
                info!("Machine is off, but --allow-off will allow us to proceed")
            } else {
                if !turn_on {
                    info!("Machine is not on, pass --turn-on to turn it on before operation");
                    return Ok(());
                }
                info!("Waiting for the machine to turn on...");
                ecam.write_request(Request::AppControl(AppControl::TurnOn))
                    .await?;
                ecam.wait_for_state(EcamStatus::Ready).await?;
            }
        }
        s => {
            info!(
                "Machine is in state {:?}, so we will cowardly refuse to brew coffee",
                s
            );
            return Ok(());
        }
    }

    info!("Fetching recipe for {:?}...", ingredients.beverage);
    let recipe_list = list_recipies_for(ecam.clone(), Some(vec![ingredients.beverage])).await?;
    let recipe = recipe_list.find(ingredients.beverage);
    if let Some(details) = recipe {
        match check_ingredients(&ingredients, details) {
            Err(s) => {
                warning!("{}", s)
            }
            Ok(recipe) => {
                info!(
                    "Brewing {:?} with {}",
                    ingredients.beverage,
                    recipe
                        .iter()
                        .map(|x| format!("--{:?}={}", x.ingredient, x.value))
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                let req = Request::BeverageDispensingMode(
                    MachineEnum::Value(ingredients.beverage),
                    MachineEnum::Value(EcamOperationTrigger::Start),
                    recipe,
                    MachineEnum::Value(EcamBeverageTasteType::Prepare),
                );

                if skip_brew {
                    info!("--skip-brew was passed, so we aren't going to brew anything");
                } else {
                    ecam.write_request(req).await?;
                }
                monitor(ecam, false).await?;
            }
        }
    } else {
        info!(
            "I wasn't able to fetch the recipe for {:?}. Perhaps this machine can't make it?",
            ingredients.beverage
        );
    }

    Ok(())
}
