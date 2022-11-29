use crate::display;
use crate::ecam::{Ecam, EcamError, EcamStatus};
use crate::prelude::*;
use crate::protocol::*;

pub async fn power_on(ecam: Ecam, allow_off: bool, turn_on: bool) -> Result<bool, EcamError> {
    match ecam.current_state().await? {
        EcamStatus::Ready => {}
        EcamStatus::StandBy => {
            if allow_off {
                info!("Machine is off, but --allow-off will allow us to proceed");
                return Ok(true);
            } else if !turn_on {
                info!("Machine is not on, pass --turn-on to turn it on before operation");
            } else {
                info!("Waiting for the machine to turn on...");
                ecam.write_request(Request::AppControl(AppControl::TurnOn))
                    .await?;
                ecam.wait_for_state(EcamStatus::Ready, display::display_status)
                    .await?;
                    return Ok(true);
                }
        }
        s => {
            info!(
                "Machine is in state {:?}, so we will cowardly refuse to brew coffee",
                s
            );
        }
    }
    Ok(false)
}
