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
    let handle = tokio::spawn(async move {
        while let Some(packet) = tap.next().await {
            // println!("{:?}", packet);
            if packet == EcamOutput::Done {
                break;
            }
        }
    });

    let mut state = ecam.current_state().await?;
    display_status(state);
    if turn_on && state == EcamStatus::StandBy {
        ecam.write_request(Request::AppControl(AppControl::TurnOn))
            .await?;
    }

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

    let _ = handle.await;

    Ok(())
}
