use crate::prelude::*;
use std::time::Instant;

use crate::display::*;
use crate::ecam::{Ecam, EcamError};

pub async fn monitor(ecam: Ecam) -> Result<(), EcamError> {
    let mut state = ecam.current_state().await?;
    display_status(state);
    let mut debounce = Instant::now();
    while ecam.is_alive() {
        // Poll for current state
        let next_state = ecam.current_state().await?;
        if next_state != state || debounce.elapsed() > Duration::from_millis(250) {
            // println!("{:?}", next_state);
            display_status(next_state);
            state = next_state;
            debounce = Instant::now();
        }
    }

    Ok(())
}
