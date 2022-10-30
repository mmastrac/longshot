use std::collections::HashMap;

use crate::{
    protocol::{
        EcamAccessory, EcamBeverageId, EcamBeverageTaste, EcamIngredients, EcamTemperature,
        MachineEnum, RecipeInfo, RecipeMinMaxInfo, Response,
    },
    warning,
};

pub struct RecipeAccumulator {
    recipe: HashMap<EcamBeverageId, Vec<RecipeInfo>>,
    recipe_min_max: HashMap<EcamBeverageId, Vec<RecipeMinMaxInfo>>,
    list: Vec<EcamBeverageId>,
}

impl RecipeAccumulator {
    pub fn new() -> Self {
        RecipeAccumulator {
            list: enum_iterator::all().collect(),
            recipe: HashMap::new(),
            recipe_min_max: HashMap::new(),
        }
    }

    pub fn limited_to(recipes: Vec<EcamBeverageId>) -> Self {
        RecipeAccumulator {
            list: recipes,
            recipe: HashMap::new(),
            recipe_min_max: HashMap::new(),
        }
    }

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

    pub fn is_complete(&self, beverage: EcamBeverageId) -> bool {
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

        // Otherwise recipes are only complete if we have both recipe and min/max
        recipe.is_some() && recipe_min_max.is_some()
    }

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

#[derive(Clone, Debug)]
pub struct RecipeList {
    pub recipes: Vec<RecipeDetails>,
}

impl RecipeList {
    pub fn find(&self, beverage: EcamBeverageId) -> Option<&RecipeDetails> {
        for r in self.recipes.iter() {
            if r.beverage == beverage {
                return Some(r);
            }
        }
        None
    }
}

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

#[derive(Clone, Debug)]
pub struct RecipeDetails {
    pub beverage: EcamBeverageId,
    recipe: Vec<RecipeInfo>,
    recipe_min_max: Vec<RecipeMinMaxInfo>,
}

impl RecipeDetails {
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

        for ingredient in enum_iterator::all() {
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
                        println!("Unknown ingredient {:?}", ingredient)
                    }
                }
            } else {
                if m1.contains_key(key) ^ m2.contains_key(key) {
                    println!(
                        "Mismatch for ingredient {:?} (recipe={:?} min_max={:?})",
                        ingredient, r1, r2
                    );
                }
            }
        }
        v
    }
}
