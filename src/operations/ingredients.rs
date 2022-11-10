//! Translation of recipe ingredients provided by the device, as well as validation of provided ingredients
//! for a brew request against the ingredients specified by the recipe.
//!
//! There's a lot of code here for some apparently simple things, but it allows us to keep the messy protocol stuff
//! separated from the semi-clean CLI interface. We also validate ingredients as much as we can to avoid sending anything
//! bad to the machine that might have unintended consequences (spilled milk, too little coffee, spectacular fire, etc).
use std::collections::HashMap;
use std::vec;

use crate::prelude::*;
use crate::protocol::*;

/// The requested ingredients to brew, generally provided by an API user or CLI input. A [`Vec<BrewIngredientInfo>`] will
/// be combined with the [`IngredientCheckMode`] and a [`Vec<IngredientRangeInfo`] to create the final brew recipe to send
/// to the machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BrewIngredientInfo {
    Coffee(u16),
    Milk(u16),
    HotWater(u16),
    Taste(EcamBeverageTaste),
    Temperature(EcamTemperature),
    Inversion(bool),
    Brew2(bool),
}

impl BrewIngredientInfo {
    pub fn to_arg_string(&self) -> Option<String> {
        let number_arg = |name: &str, value| format!("--{} {}", name, value);
        match self {
            Self::Coffee(value) => Some(number_arg("coffee", value)),
            Self::Milk(value) => Some(number_arg("milk", value)),
            Self::HotWater(value) => Some(number_arg("hotwater", value)),
            Self::Taste(value) => Some(format!("--taste {}", value.to_arg_string(),)),
            Self::Temperature(value) => Some(format!("--temp {}", value.to_arg_string(),)),
            // We don't support these for now
            Self::Inversion(..) | Self::Brew2(..) => None,
        }
    }

    pub fn ingredient(&self) -> EcamIngredients {
        match self {
            Self::Coffee(..) => EcamIngredients::Coffee,
            Self::Milk(..) => EcamIngredients::Milk,
            Self::HotWater(..) => EcamIngredients::HotWater,
            Self::Taste(..) => EcamIngredients::Taste,
            Self::Temperature(..) => EcamIngredients::Temp,
            Self::Inversion(..) => EcamIngredients::Inversion,
            Self::Brew2(..) => EcamIngredients::DueXPer,
        }
    }

    pub fn value_u16(&self) -> u16 {
        match self {
            Self::Coffee(x) => *x,
            Self::Milk(x) => *x,
            Self::HotWater(x) => *x,
            Self::Taste(x) => <u8>::from(*x) as u16,
            Self::Temperature(x) => <u8>::from(*x) as u16,
            Self::Inversion(x) => <u16>::from(*x),
            Self::Brew2(x) => <u16>::from(*x),
        }
    }

    pub fn to_recipe_info(&self) -> RecipeInfo<u16> {
        RecipeInfo::<u16>::new(self.ingredient(), self.value_u16())
    }
}

/// The processed ingredients from the raw ECAM responses. Some ingredients are omitted as they are not useful for brewing.
///
/// This could be done with the raw [`RecipeMinMaxInfo`], but an older attempt at this code tried that and it became a
/// fairly decent mess.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IngredientRangeInfo {
    Coffee(u16, u16, u16),
    Milk(u16, u16, u16),
    HotWater(u16, u16, u16),
    Taste(EcamBeverageTaste),
    Temperature(EcamTemperature),
    Accessory(EcamAccessory),
    Inversion(bool, bool),
    Brew2(bool, bool),
}

impl IngredientRangeInfo {
    /// Attempts to parse a [`RecipeInfo`] and [`RecipeMinMaxInfo`] into an [`IngredientRangeInfo`]. If this fails, it returns a string with
    /// a human-readable error.
    pub fn new(
        ingredient: EcamIngredients,
        r1: Option<RecipeInfo<u16>>,
        r2: Option<RecipeMinMaxInfo>,
    ) -> Result<Option<Self>, String> {
        // Ignore these types of ingredient
        if matches!(
            ingredient,
            EcamIngredients::Visible | EcamIngredients::IndexLength | EcamIngredients::Programmable
        ) {
            return Ok(None);
        }

        // Handle accessory separately, as it appears to differ between recipe and min/max
        if ingredient == EcamIngredients::Accessorio {
            return if let Some(r1) = r1 {
                match r1.value {
                    0 => Ok(None),
                    1 => Ok(Some(IngredientRangeInfo::Accessory(EcamAccessory::Water))),
                    2 => Ok(Some(IngredientRangeInfo::Accessory(EcamAccessory::Milk))),
                    _ => Err(format!("Unknown accessory value {}", r1.value)),
                }
            } else {
                Ok(None)
            };
        }

        macro_rules! error {
            ($msg:literal, $ingredient:expr, $r1:expr, $r2:expr) => {
                Err(format!(
                    "Specified ingredient {:?} {} ({}<={}<={}, value={})",
                    $ingredient, $msg, $r2.min, $r2.value, $r2.max, $r1.value
                ))
            };
        }

        return if let (Some(r1), Some(r2)) = (&r1, &r2) {
            if matches!(
                ingredient,
                EcamIngredients::Coffee | EcamIngredients::Milk | EcamIngredients::HotWater
            ) {
                // This appears to be the case for invalid ingredients in custom recipes
                if r1.value == 0 && r2.min > 0 {
                    return error!("with invalid ranges", ingredient, r1, r2);
                }
                // This shows up on the Cortado recipe on the Dinamica Plus
                if r2.min == r2.value && r2.value == r2.max && r2.value == 0 {
                    return error!("with zero ranges", ingredient, r1, r2);
                }
            }
            match ingredient {
                EcamIngredients::Coffee => {
                    Ok(Some(IngredientRangeInfo::Coffee(r2.min, r1.value, r2.max)))
                }
                EcamIngredients::Milk => {
                    Ok(Some(IngredientRangeInfo::Milk(r2.min, r1.value, r2.max)))
                }
                EcamIngredients::HotWater => Ok(Some(IngredientRangeInfo::HotWater(
                    r2.min, r1.value, r2.max,
                ))),
                EcamIngredients::Taste => {
                    if r2.min == 0 && r2.max == 5 {
                        if let Ok(taste) = EcamBeverageTaste::try_from(r1.value as u8) {
                            Ok(Some(IngredientRangeInfo::Taste(taste)))
                        } else {
                            error!("unknown", ingredient, r1, r2)
                        }
                    } else {
                        error!("unknown range", ingredient, r1, r2)
                    }
                }
                EcamIngredients::Temp => {
                    Ok(Some(IngredientRangeInfo::Temperature(EcamTemperature::Low)))
                }
                EcamIngredients::Inversion => Ok(Some(IngredientRangeInfo::Inversion(
                    r2.value == 1,
                    r2.min == r2.max,
                ))),
                EcamIngredients::DueXPer => Ok(Some(IngredientRangeInfo::Brew2(
                    r2.value == 1,
                    r2.min == r2.max,
                ))),
                _ => error!("is unknown", ingredient, r1, r2),
            }
        } else {
            if r1.is_some() ^ r2.is_some() {
                // If only one of min/max or recipe quantity comes back, that's bad
                Err(format!(
                    "Mismatch for ingredient {:?} (recipe={:?} min_max={:?})",
                    ingredient, r1, r2
                ))
            } else {
                // Otherwise it's just missing
                Ok(None)
            }
        };
    }

    pub fn to_arg_string(&self) -> Option<String> {
        let number_arg = |name: &str, min, value, max| {
            format!("--{} <{}-{}, default {}>", name, min, max, value)
        };

        match self {
            Self::Coffee(min, value, max) => Some(number_arg("coffee", min, value, max)),
            Self::Milk(min, value, max) => Some(number_arg("milk", min, value, max)),
            Self::HotWater(min, value, max) => Some(number_arg("hotwater", min, value, max)),
            Self::Taste(value) => Some(format!(
                "--taste <{}, default={}>",
                EcamBeverageTaste::all()
                    .map(|e| e.to_arg_string())
                    .collect::<Vec<_>>()
                    .join("|"),
                value.to_arg_string(),
            )),
            Self::Temperature(value) => Some(format!(
                "--temp <{}, default={}>",
                EcamTemperature::all()
                    .map(|e| e.to_arg_string())
                    .collect::<Vec<_>>()
                    .join("|"),
                value.to_arg_string(),
            )),
            // We don't support these for now
            Self::Accessory(..) | Self::Inversion(..) | Self::Brew2(..) => None,
        }
    }

    pub fn ingredient(&self) -> EcamIngredients {
        match self {
            Self::Coffee(..) => EcamIngredients::Coffee,
            Self::Milk(..) => EcamIngredients::Milk,
            Self::HotWater(..) => EcamIngredients::HotWater,
            Self::Taste(..) => EcamIngredients::Taste,
            Self::Temperature(..) => EcamIngredients::Temp,
            Self::Inversion(..) => EcamIngredients::Inversion,
            Self::Brew2(..) => EcamIngredients::DueXPer,
            Self::Accessory(..) => EcamIngredients::Accessorio,
        }
    }
}

/// Determines how ingredients are checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngredientCheckMode {
    /// Each ingredient is required, and must match one provided by the recipe.
    Strict,
    /// Ingredients are all optional and will be provided by the recipe. All ingredients must be present in the recipe.
    AllowDefaults,
    /// Disable all ingredient checking and process the ingredients as-is. CAUTION: this may have unintended results
    /// or cause damage to the machine.
    Force,
}

/// Result of the [`check_ingredients`] call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IngredientCheckResult {
    /// The ingredients are valid.
    Ok(Vec<BrewIngredientInfo>),
    /// One or more ingredients failed to validate.
    Error {
        missing: Vec<IngredientRangeInfo>,
        extra: Vec<EcamIngredients>,
        range_errors: Vec<(EcamIngredients, String)>,
    },
}

/// Checks this [`BrewIngredientInfo`] against an [`IngredientRangeInfo`] and returns [`Ok(RecipeInfo)`] if valid.
pub fn check_ingredients(
    mode: IngredientCheckMode,
    brew: &Vec<BrewIngredientInfo>,
    ranges: &Vec<IngredientRangeInfo>,
) -> IngredientCheckResult {
    let mut v = vec![];
    let mut extra = vec![];
    let mut range_errors = vec![];
    let mut ranges_map = HashMap::new();
    for ingredient in ranges.iter() {
        if matches!(
            ingredient,
            IngredientRangeInfo::Accessory(..)
                | IngredientRangeInfo::Brew2(..)
                | IngredientRangeInfo::Inversion(..)
        ) {
            continue;
        }
        ranges_map.insert(ingredient.ingredient(), ingredient);
    }
    for ingredient in brew.iter() {
        let key = ingredient.ingredient();
        if let Some(range) = ranges_map.remove(&key) {
            match check_ingredient(ingredient, range) {
                Err(s) => range_errors.push((key, s)),
                Ok(r) => v.push(r),
            }
        } else {
            extra.push(ingredient.ingredient());
        }
    }
    let missing: Vec<_> = ranges_map.values().map(|y| **y).collect();
    if extra.len() == 0 && missing.len() == 0 && range_errors.len() == 0 {
        IngredientCheckResult::Ok(v)
    } else {
        IngredientCheckResult::Error {
            extra,
            missing,
            range_errors,
        }
    }
}

pub fn check_ingredient(
    brew: &BrewIngredientInfo,
    range: &IngredientRangeInfo,
) -> Result<BrewIngredientInfo, String> {
    let validate_u16 = |ingredient: fn(u16) -> BrewIngredientInfo, min, value: u16, max| {
        if value.clamp(min, max) == value {
            Ok(ingredient(value))
        } else {
            Err(format!(
                "{:?} value out of range ({}<={}<={})",
                ingredient, min, value, max
            ))
        }
    };

    match (*brew, *range) {
        (BrewIngredientInfo::Coffee(value), IngredientRangeInfo::Coffee(min, _, max)) => {
            validate_u16(BrewIngredientInfo::Coffee, min, value, max)
        }
        (BrewIngredientInfo::Milk(value), IngredientRangeInfo::Milk(min, _, max)) => {
            validate_u16(BrewIngredientInfo::Milk, min, value, max)
        }
        (BrewIngredientInfo::HotWater(value), IngredientRangeInfo::HotWater(min, _, max)) => {
            validate_u16(BrewIngredientInfo::HotWater, min, value, max)
        }
        (brew, range) => {
            panic!(
                "Incorrect pairing, likely an internal error: {:?} {:?}",
                brew, range
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Basic espresso, just coffee.
    const ESPRESSO_RECIPE: [IngredientRangeInfo; 1] = [IngredientRangeInfo::Coffee(0, 100, 250)];
    /// Cappucino with coffee and milk.
    const CAPPUCINO_RECIPE: [IngredientRangeInfo; 2] = [
        IngredientRangeInfo::Coffee(0, 100, 250),
        IngredientRangeInfo::Milk(0, 50, 750),
    ];

    #[test]
    fn test_strict() {
        assert_eq!(
            IngredientCheckResult::Ok(vec![BrewIngredientInfo::Coffee(100)]),
            check_ingredients(
                IngredientCheckMode::Strict,
                &vec![BrewIngredientInfo::Coffee(100)],
                &ESPRESSO_RECIPE.to_vec()
            )
        );
        assert_eq!(
            IngredientCheckResult::Error {
                missing: vec![],
                extra: vec![],
                range_errors: vec![(
                    EcamIngredients::Coffee,
                    "Coffee value out of range (0<=1000<=250)".to_owned()
                )]
            },
            check_ingredients(
                IngredientCheckMode::Strict,
                &vec![BrewIngredientInfo::Coffee(1000)],
                &ESPRESSO_RECIPE.to_vec()
            )
        );
        assert_eq!(
            IngredientCheckResult::Error {
                missing: vec![IngredientRangeInfo::Coffee(0, 100, 250)],
                extra: vec![],
                range_errors: vec![]
            },
            check_ingredients(
                IngredientCheckMode::Strict,
                &vec![],
                &ESPRESSO_RECIPE.to_vec()
            )
        );
        assert_eq!(
            IngredientCheckResult::Error {
                extra: vec![EcamIngredients::Milk],
                missing: vec![IngredientRangeInfo::Coffee(0, 100, 250)],
                range_errors: vec![]
            },
            check_ingredients(
                IngredientCheckMode::Strict,
                &vec![BrewIngredientInfo::Milk(100)],
                &ESPRESSO_RECIPE.to_vec()
            )
        );
    }
}
