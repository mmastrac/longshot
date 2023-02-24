use crate::{display, prelude::*};
use crate::{
    ecam::{Ecam, EcamError},
    operations::IngredientRangeInfo,
    protocol::*,
};
use std::collections::HashMap;

/// Accumulates recipe responses, allowing us to fetch them one-at-a-time and account for which ones went missing in transit.
/// Note that this doesn't support profiles yet and currently requires the use of profile 1.
pub struct RecipeAccumulator {
    recipe: HashMap<EcamBeverageId, Vec<RecipeInfo<u16>>>,
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
        Self::limited_to(EcamBeverageId::all_values().to_vec())
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

    /// Returns the [`Request`] required to fetch this beverage.
    pub fn get_request_packets(&self, beverage: EcamBeverageId) -> Vec<Request> {
        vec![
            Request::RecipeMinMaxSync(beverage.into()),
            Request::RecipeQuantityRead(1, beverage.into()),
        ]
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

    pub fn get(
        &self,
        beverage: EcamBeverageId,
    ) -> (Option<Vec<RecipeInfo<u16>>>, Option<Vec<RecipeMinMaxInfo>>) {
        (
            self.recipe.get(&beverage).map(Clone::clone),
            self.recipe_min_max.get(&beverage).map(Clone::clone),
        )
    }
}

/// A completed list of [`RecipeDetails`], containing one [`RecipeDetails`] object for each valid [`EcamBeverageId`].
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

/// The details for a given [`EcamBeverageId`]'s recipe.
#[derive(Clone, Debug)]
pub struct RecipeDetails {
    pub beverage: EcamBeverageId,
    recipe: Vec<RecipeInfo<u16>>,
    recipe_min_max: Vec<RecipeMinMaxInfo>,
}

impl RecipeDetails {
    /// Formats this recipe as an argument string.
    pub fn to_arg_string(&self) -> String {
        let args = self
            .fetch_ingredients()
            .iter()
            .collect_filter_map_join(" ", IngredientRangeInfo::to_arg_string);
        format!("--beverage {} {}", self.beverage.to_arg_string(), args)
    }

    /// Processes this [`RecipeDetails`] into a [`Vec<IngredientInfo>`], suitable for dispensing.
    pub fn fetch_ingredients(&self) -> Vec<IngredientRangeInfo> {
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
            let key = ingredient.into();
            match IngredientRangeInfo::new(
                ingredient,
                m1.get(&key).map(|x| **x),
                m2.get(&key).map(|x| **x),
            ) {
                Err(s) => warning!("{}", s),
                Ok(Some(x)) => v.push(x),
                Ok(None) => {}
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
    Ok(accumulate_recipies_for(ecam, recipes).await?.take())
}

/// Accumulates recipe min/max and ingredient info for either all recipes, or just the given ones.
pub async fn accumulate_recipies_for(
    ecam: Ecam,
    recipes: Option<Vec<EcamBeverageId>>,
) -> Result<RecipeAccumulator, EcamError> {
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
            'inner: for packet in recipes.get_request_packets(beverage) {
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
        display::display_status(crate::ecam::EcamStatus::Fetching(100));
        display::clear_status();
    }
    Ok(recipes)
}

pub async fn list_recipes(ecam: Ecam) -> Result<(), EcamError> {
    // Wait for device to settle
    ecam.wait_for_connection().await?;
    let list = list_recipies_for(ecam, None).await?;
    info!("Beverages supported:");
    for recipe in list.recipes {
        info!("  {}", recipe.to_arg_string());
    }

    Ok(())
}

fn enspacen(b: &[u8]) -> String {
    let mut s = "".to_owned();
    let space = "·";
    if let Some((head, tail)) = b.split_first() {
        s += &format!("{:02x}{}", head, space);
        s += &match tail.len() {
            1 => format!("{:02x}", tail[0]),
            2 => format!("{:02x}{:02x}", tail[0], tail[1]),
            3 => format!(
                "{:02x}{}{:02x}{}{:02x}",
                tail[0], space, tail[1], space, tail[2]
            ),
            6 => format!(
                "{:02x}{:02x}{}{:02x}{:02x}{}{:02x}{:02x}",
                tail[0], tail[1], space, tail[2], tail[3], space, tail[4], tail[5]
            ),
            _ => hex::encode(tail),
        };
    }
    s
}

pub async fn list_recipes_detailed(ecam: Ecam) -> Result<(), EcamError> {
    use ariadne::{Color, Config, Label, Report, ReportBuilder, ReportKind, Source};
    const LINE_LIMIT: usize = 100;

    // Wait for device to settle
    ecam.wait_for_connection().await?;
    let list = accumulate_recipies_for(ecam, None).await?;
    for beverage in EcamBeverageId::all() {
        let name = &format!("{:?}", beverage);
        let (recipe, minmax) = list.get(beverage);
        let mut s = "".to_owned();
        let mut builder = Report::build(ReportKind::Custom("Beverage", Color::Cyan), name, 0)
            .with_message(format!("{:?} (id 0x{:02x})", beverage, beverage as u8));
        let len = |s: &str| s.chars().count();

        // Add a chunk of labelled text
        let add_labelled_text =
            |builder: ReportBuilder<_>, i: usize, s: &mut String, t: &str, msg: &str| {
                if i > 0 {
                    if s.rfind('\n').unwrap_or_default() + LINE_LIMIT < s.len() {
                        *s += "•\n↳ ";
                    } else {
                        *s += "•";
                    }
                }
                let start_len = len(s);
                *s += t;
                let end_len = len(s);
                let label = Label::new((name, start_len..end_len))
                    .with_message(msg)
                    .with_order(-(i as i32))
                    .with_color([Color::Unset, Color::Cyan][i % 2]);
                builder.with_label(label)
            };

        // Add a note about missing data
        let add_missing_note = |mut builder: ReportBuilder<_>, s: &mut String, msg| {
            builder = add_labelled_text(builder, 0, s, "", msg);
            builder.with_note("The recipe or min/max info is not correct, which means this recipe is likely not supported")
        };

        // Print the recipe
        if let Some(recipe) = recipe {
            if recipe.is_empty() {
                builder = add_missing_note(builder, &mut s, "Empty recipe");
            }
            for (i, recipe_info) in recipe.iter().enumerate() {
                builder = add_labelled_text(
                    builder,
                    i,
                    &mut s,
                    &enspacen(&recipe_info.encode()),
                    &format!("{:?}={}", recipe_info.ingredient, recipe_info.value),
                );
            }
        } else {
            builder = add_missing_note(builder, &mut s, "Missing recipe");
        }
        s += "\n";

        // Print the min/max info
        if let Some(minmax) = minmax {
            if minmax.is_empty() {
                builder = add_missing_note(builder, &mut s, "Empty min/max");
            }
            for (i, minmax_info) in minmax.iter().enumerate() {
                builder = add_labelled_text(
                    builder,
                    i,
                    &mut s,
                    &enspacen(&minmax_info.encode()),
                    &format!(
                        "{:?}: {}<={}<={}",
                        minmax_info.ingredient, minmax_info.min, minmax_info.value, minmax_info.max
                    ),
                );
            }
        } else {
            builder = add_missing_note(builder, &mut s, "Missing min/max");
        }
        s += "\n";

        builder
            .with_config(Config::default().with_underlines(true))
            .finish()
            .print((name, Source::from(s)))?;
    }

    Ok(())
}

pub async fn list_recipes_raw(ecam: Ecam) -> Result<(), EcamError> {
    // Wait for device to settle
    ecam.wait_for_connection().await?;
    let list = accumulate_recipies_for(ecam, None).await?;
    let mut s = "".to_owned();

    for beverage in EcamBeverageId::all() {
        if !list.is_complete(beverage) {
            continue;
        }

        s += &format!("# {:?} (id=0x{:02x})\n", beverage, beverage as u8);
        let (recipe, minmax) = list.get(beverage);

        // Print the recipe
        if let Some(recipe) = recipe {
            for recipe_info in recipe.iter() {
                s += &hex::encode(recipe_info.encode());
            }
        }
        s += "\n";

        // Print the min/max info
        if let Some(minmax) = minmax {
            for minmax_info in minmax.iter() {
                s += &hex::encode(minmax_info.encode());
            }
        }
        s += "\n";
    }

    println!("{}", s);

    Ok(())
}
