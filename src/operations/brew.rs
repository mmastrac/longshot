use crate::{prelude::*, display};
use crate::{
    ecam::{Ecam, EcamError, EcamStatus},
    operations::{
        check_ingredients, list_recipies_for, BrewIngredientInfo, IngredientCheckError,
        IngredientCheckMode,
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
            Err(IngredientCheckError {
                missing,
                extra,
                range_errors,
            }) => {
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
            Ok(result) => {
                info!(
                    "Brewing {:?} with {}...",
                    beverage,
                    result
                        .iter()
                        .collect_filter_map_join(" ", BrewIngredientInfo::to_arg_string)
                );
                Ok(result
                    .iter()
                    .map(BrewIngredientInfo::to_recipe_info)
                    .collect())
            }
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
    skip_brew: bool,
    beverage: EcamBeverageId,
    recipe: Vec<RecipeInfo<u16>>,
) -> Result<(), EcamError> {
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

    // Wait for not ready
    ecam.wait_for_not_state(EcamStatus::Ready, display::display_status).await?;

    // Wait for not busy
    ecam.wait_for(|m| match EcamStatus::extract(m) {
        EcamStatus::Busy(_) => { false },
        _ => { true } 
    }, display::display_status).await?;

    display::log(display::LogLevel::Info, "Completed");

    Ok(())
}
