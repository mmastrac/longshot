use crate::prelude::*;
use std::time::Instant;

use crate::display::*;
use crate::{
    ecam::{Ecam, EcamError, EcamOutput, EcamStatus},
    protocol::*,
};

pub async fn monitor(ecam: Ecam, turn_on: bool) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;
    let ecam = ecam.clone();
    let _handle = tokio::spawn(async move {
        while let Some(packet) = tap.next().await {
            // println!("{:?}", packet);
            if packet == EcamOutput::Done {
                break;
            }
        }
    });

    let mut display: Box<dyn StatusDisplay> = Box::new(ColouredStatusDisplay::new(80));
    let mut state = ecam.current_state().await?;
    display.display(state);
    if turn_on && state == EcamStatus::StandBy {
        ecam.write_request(Request::AppControl(AppControl::TurnOn))
            .await?;
    }

    let mut debounce = Instant::now();
    loop {
        // Poll for current state
        let next_state = ecam.current_state().await?;
        if next_state != state || debounce.elapsed() > Duration::from_millis(250) {
            // println!("{:?}", next_state);
            display.display(next_state);
            state = next_state;
            debounce = Instant::now();
        }
    }

    Ok(())
}
