use std::fmt::format;

use crate::prelude::*;
use crate::{
    display,
    ecam::{Ecam, EcamError, EcamStatus},
    operations::{
        check_ingredients, list_recipies_for, monitor, BrewIngredientInfo, IngredientCheckMode,
        IngredientCheckResult,
    },
    protocol::*,
};

/// Checks the arguments for the given beverage against the machine's recipes and returns a computed recipe.
pub async fn validate_brew(
    ecam: Ecam,
    beverage: EcamBeverageId,
    ingredients: Vec<BrewIngredientInfo>,
    mode: IngredientCheckMode,
) -> Result<Vec<RecipeInfo<u16>>, EcamError> {
    info!("Fetching recipe for {:?}...", beverage);
    let recipe_list = list_recipies_for(ecam.clone(), Some(vec![beverage])).await?;
    let recipe = recipe_list.find(beverage);
    if let Some(recipe) = recipe {
        let ranges = recipe.fetch_ingredients();
        match check_ingredients(mode, &ingredients, &ranges) {
            IngredientCheckResult::Error {
                missing,
                extra,
                range_errors,
            } => {
                for m in missing {
                    info!("{}", m.to_arg_string().unwrap_or(format!("{:?}", m)));
                }
                for e in extra {
                    info!("{}", e.to_arg_string());
                }
                for r in range_errors {
                    info!("{}", r.1);
                }
                Err(EcamError::Unknown)
            }
            IngredientCheckResult::Ok(result) => Ok(result),
        }
    } else {
        info!(
            "I wasn't able to fetch the recipe for {:?}. Perhaps this machine can't make it?",
            beverage
        );
        Err(EcamError::NotFound)
    }
}

pub async fn brew(
    ecam: Ecam,
    turn_on: bool,
    allow_off: bool,
    skip_brew: bool,
    dump_decoded_packets: bool,
    beverage: EcamBeverageId,
    recipe: Vec<RecipeInfo<u16>>,
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
                ecam.wait_for_state(EcamStatus::Ready, display::display_status)
                    .await?;
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

    info!("Brewing {:?}...", beverage);
    let req = Request::BeverageDispensingMode(
        beverage.into(),
        EcamOperationTrigger::Start.into(),
        recipe,
        EcamBeverageTasteType::Prepare.into(),
    );

    if skip_brew {
        info!("--skip-brew was passed, so we aren't going to brew anything");
    } else {
        ecam.write_request(req).await?;
    }
    monitor(ecam, false, dump_decoded_packets).await?;

    Ok(())
}
