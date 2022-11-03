use crate::{display, prelude::*};
use std::collections::HashMap;

use crate::{
    ecam::{Ecam, EcamError},
    protocol::*,
};

/// Accumulates recipe responses, allowing us to fetch them one-at-a-time and account for which ones went missing in transit.
pub struct RecipeAccumulator {
    recipe: HashMap<EcamBeverageId, Vec<RecipeInfo>>,
    recipe_min_max: HashMap<EcamBeverageId, Vec<RecipeMinMaxInfo>>,
    list: Vec<EcamBeverageId>,
}

impl Default for RecipeAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeAccumulator {
    /// Creates a new accumulator for all recipes.
    pub fn new() -> Self {
        Self::limited_to(EcamBeverageId::all().collect())
    }

    /// Creates a new accumulator limited to a smaller subset of [`EcamBeverageId`]s (potentially just one).
    pub fn limited_to(recipes: Vec<EcamBeverageId>) -> Self {
        RecipeAccumulator {
            list: recipes,
            recipe: HashMap::new(),
            recipe_min_max: HashMap::new(),
        }
    }

    /// Lists the [`EcamBeverageId`]s which we still need to fetch information for.
    pub fn get_remaining_beverages(&self) -> Vec<EcamBeverageId> {
        let mut remaining = vec![];
        for beverage in self.list.iter() {
            if self.recipe.contains_key(beverage) && self.recipe_min_max.contains_key(beverage) {
                continue;
            }
            if self.is_empty(*beverage) {
                continue;
            }
            remaining.push(*beverage);
        }
        remaining
    }

    /// Is our fetch complete for this [`EcamBeverageId`].
    pub fn is_complete(&self, beverage: EcamBeverageId) -> bool {
        let recipe = self.recipe.get(&beverage);
        let recipe_min_max = self.recipe_min_max.get(&beverage);

        // If the recipe has empty ingredients, we're going to ignore it and say it's complete
        if let Some(recipe) = recipe {
            if recipe.is_empty() {
                return true;
            }
        }
        if let Some(recipe_min_max) = recipe_min_max {
            if recipe_min_max.is_empty() {
                return true;
            }
        }

        // Otherwise recipes are only complete if we have both recipe and min/max
        recipe.is_some() && recipe_min_max.is_some()
    }

    /// Is this [`EcamBeverageId`] empty, ie: is it unavailable for dispensing?
    pub fn is_empty(&self, beverage: EcamBeverageId) -> bool {
        let recipe = self.recipe.get(&beverage);
        let recipe_min_max = self.recipe_min_max.get(&beverage);

        // If the recipe has empty ingredients, we're going to ignore it and say it's complete
        if let Some(recipe) = recipe {
            if recipe.len() == 0 {
                return true;
            }
        }
        if let Some(recipe_min_max) = recipe_min_max {
            if recipe_min_max.len() == 0 {
                return true;
            }
        }

        false
    }

    /// Accumulate a [`Response`] for the given [`EcamBeverageId`].
    pub fn accumulate_packet(&mut self, expected_beverage: EcamBeverageId, packet: Response) {
        match packet {
            Response::RecipeQuantityRead(_, beverage, ingredients) => {
                if beverage == expected_beverage {
                    self.recipe.insert(expected_beverage, ingredients);
                }
            }
            Response::RecipeMinMaxSync(beverage, min_max) => {
                if beverage == expected_beverage {
                    self.recipe_min_max.insert(expected_beverage, min_max);
                }
            }
            _ => {
                warning!("Spurious packet received? {:?}", packet);
            }
        }
    }

    /// Take the contents of this instance as a [`RecipeList`].
    pub fn take(mut self) -> RecipeList {
        let mut list = RecipeList { recipes: vec![] };
        for beverage in self.list.iter() {
            if self.is_empty(*beverage) {
                continue;
            }
            let recipe = self.recipe.remove(beverage);
            let recipe_min_max = self.recipe_min_max.remove(beverage);
            if let (Some(recipe), Some(recipe_min_max)) = (recipe, recipe_min_max) {
                list.recipes.push(RecipeDetails {
                    beverage: *beverage,
                    recipe,
                    recipe_min_max,
                });
            } else {
                warning!(
                    "Recipe data seems to be out of sync, ignoring beverage {:?}",
                    beverage
                );
            }
        }
        list
    }
}

/// A completed list of [`RecipeDetails`].
#[derive(Clone, Debug)]
pub struct RecipeList {
    pub recipes: Vec<RecipeDetails>,
}

impl RecipeList {
    /// Find the recipe for the given [`EcamBeverageId`], returning it as a [`RecipeDetails`].
    pub fn find(&self, beverage: EcamBeverageId) -> Option<&RecipeDetails> {
        self.recipes.iter().find(|&r| r.beverage == beverage)
    }
}

/// The processed ingredients from the raw ECAM responses. Some ingredients are omitted as they are not useful for brewing.
#[derive(Clone, Debug)]
pub enum IngredientInfo {
    Coffee(u16, u16, u16),
    Milk(u16, u16, u16),
    HotWater(u16, u16, u16),
    Taste(EcamBeverageTaste),
    Temperature(EcamTemperature),
    Accessory(EcamAccessory),
    Inversion(bool, bool),
    Brew2(bool, bool),
}

/// The details for a given [`EcamBeverageId`]'s recipe.
#[derive(Clone, Debug)]
pub struct RecipeDetails {
    pub beverage: EcamBeverageId,
    recipe: Vec<RecipeInfo>,
    recipe_min_max: Vec<RecipeMinMaxInfo>,
}

impl RecipeDetails {
    /// Processes this [`RecipeDetails`] into a [`Vec<IngredientInfo>`], suitable for dispensing.
    pub fn fetch_ingredients(&self) -> Vec<IngredientInfo> {
        let mut v = vec![];
        let mut m1 = HashMap::new();
        let mut m2 = HashMap::new();
        for r in self.recipe.iter() {
            m1.insert(r.ingredient, r);
        }
        for r in self.recipe_min_max.iter() {
            m2.insert(r.ingredient, r);
        }

        for ingredient in EcamIngredients::all() {
            let key = &MachineEnum::Value(ingredient);
            if matches!(
                ingredient,
                EcamIngredients::Visible
                    | EcamIngredients::IndexLength
                    | EcamIngredients::Programmable
            ) {
                continue;
            }

            let r1 = m1.get(key);
            let r2 = m2.get(key);

            // Handle accessory separately, as it appears to differ between recipe and min/max
            if ingredient == EcamIngredients::Accessorio {
                if let Some(r1) = r1 {
                    match r1.value {
                        0 => {
                            continue;
                        }
                        1 => v.push(IngredientInfo::Accessory(EcamAccessory::Water)),
                        2 => v.push(IngredientInfo::Accessory(EcamAccessory::Milk)),
                        _ => {
                            warning!("Unknown accessory value {}", r1.value)
                        }
                    }
                }
                continue;
            }

            if let (Some(r1), Some(r2)) = (r1, r2) {
                match ingredient {
                    EcamIngredients::Coffee => {
                        v.push(IngredientInfo::Coffee(r2.min, r1.value, r2.max))
                    }
                    EcamIngredients::Milk => v.push(IngredientInfo::Milk(r2.min, r1.value, r2.max)),
                    EcamIngredients::HotWater => {
                        v.push(IngredientInfo::HotWater(r2.min, r1.value, r2.max))
                    }
                    EcamIngredients::Taste => {
                        if r2.min == 0 && r2.max == 5 {
                            if let Ok(taste) = EcamBeverageTaste::try_from(r1.value as u8) {
                                v.push(IngredientInfo::Taste(taste));
                            } else {
                                warning!("Unknown beverage taste {}", r1.value);
                            }
                        }
                    }
                    EcamIngredients::Temp => {
                        v.push(IngredientInfo::Temperature(EcamTemperature::Low))
                    }
                    EcamIngredients::Inversion => {
                        v.push(IngredientInfo::Inversion(r2.value == 1, r2.min == r2.max))
                    }
                    EcamIngredients::DueXPer => {
                        v.push(IngredientInfo::Brew2(r2.value == 1, r2.min == r2.max))
                    }
                    _ => {
                        warning!("Unknown ingredient {:?}", ingredient)
                    }
                }
            } else if m1.contains_key(key) ^ m2.contains_key(key) {
                warning!(
                    "Mismatch for ingredient {:?} (recipe={:?} min_max={:?})",
                    ingredient,
                    r1,
                    r2
                );
            }
        }
        v
    }
}

/// Lists recipes for either all recipes, or just the given ones.
pub async fn list_recipies_for(
    ecam: Ecam,
    recipes: Option<Vec<EcamBeverageId>>,
) -> Result<RecipeList, EcamError> {
    // Get the tap we'll use for reading responses
    let mut tap = ecam.packet_tap().await?;
    let mut recipes = if let Some(recipes) = recipes {
        RecipeAccumulator::limited_to(recipes)
    } else {
        RecipeAccumulator::new()
    };
    let total = recipes.get_remaining_beverages().len();
    for i in 0..3 {
        if i == 0 {
            info!("Fetching recipes...");
        } else if !recipes.get_remaining_beverages().is_empty() {
            info!(
                "Fetching potentially missing recipes... {:?}",
                recipes.get_remaining_beverages()
            );
        }
        'outer: for beverage in recipes.get_remaining_beverages() {
            'inner: for packet in vec![
                Request::RecipeMinMaxSync(MachineEnum::Value(beverage)),
                Request::RecipeQuantityRead(1, MachineEnum::Value(beverage)),
            ] {
                crate::display::display_status(crate::ecam::EcamStatus::Fetching(
                    (total - recipes.get_remaining_beverages().len()) * 100 / total,
                ));
                let request_id = packet.ecam_request_id();
                ecam.write_request(packet).await?;
                let now = std::time::Instant::now();
                while now.elapsed() < Duration::from_millis(500) {
                    match tokio::time::timeout(Duration::from_millis(50), tap.next()).await {
                        Err(_) => {}
                        Ok(None) => {}
                        Ok(Some(x)) => {
                            if let Some(packet) = x.take_packet() {
                                let response_id = packet.ecam_request_id();
                                recipes.accumulate_packet(beverage, packet);
                                // If this recipe is totally complete, move to the next one
                                if recipes.is_complete(beverage) {
                                    continue 'outer;
                                }
                                // If we got a response for the given request, move to the next packet/beverage
                                if response_id == request_id {
                                    continue 'inner;
                                }
                            }
                        }
                    }
                }
            }
        }
        crate::display::display_status(crate::ecam::EcamStatus::Fetching(100));
        crate::display::display_status(ecam.current_state().await?);
    }
    Ok(recipes.take())
}

pub async fn list_recipes(ecam: Ecam) -> Result<(), EcamError> {
    // Wait for device to settle
    ecam.wait_for_connection().await?;
    let list = list_recipies_for(ecam, None).await?;
    display::clear_status();
    for recipe in list.recipes {
        info!("{:?} {:?}", recipe.beverage, recipe.fetch_ingredients());
    }

    Ok(())
}
