use tokio::sync::Mutex;

use crate::ecam::{EcamDriver, EcamDriverOutput, EcamError};
use crate::prelude::*;
use crate::protocol::{
    EcamAccessory, EcamDriverPacket, EcamMachineState, EcamMachineSwitch, EcamRequestId,
    MachineEnum, MonitorV2Response, PartialEncode, SwitchSet,
};

struct EcamSimulate {
    rx: Mutex<tokio::sync::mpsc::Receiver<Vec<u8>>>,
}

impl EcamDriver for EcamSimulate {
    fn read(&self) -> AsyncFuture<Option<EcamDriverOutput>> {
        Box::pin(async {
            let packet = self.rx.lock().await.recv().await;
            Ok(packet.map(|x| EcamDriverOutput::Packet(EcamDriverPacket::from_vec(x))))
        })
    }

    fn write(&self, data: crate::protocol::EcamDriverPacket) -> AsyncFuture<()> {
        Box::pin(async move {
            if data.bytes[0] == EcamRequestId::RecipeQuantityRead as u8 {
                println!("{:?}", data.bytes);
            }
            if data.bytes[0] == EcamRequestId::RecipeMinMaxSync as u8 {
                println!("{:?}", data.bytes);
            }
            Ok(())
        })
    }

    fn scan<'a>() -> AsyncFuture<'a, (String, uuid::Uuid)>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

/// Create a Vec<u8> that mocks a machine response.
fn make_simulated_response(state: EcamMachineState, progress: u8, percentage: u8) -> Vec<u8> {
    let mut v = vec![EcamRequestId::MonitorV2.into(), 0xf0];
    v.extend_from_slice(
        &MonitorV2Response {
            state: MachineEnum::Value(state),
            accessory: MachineEnum::Value(EcamAccessory::None),
            switches: SwitchSet::of(&[EcamMachineSwitch::WaterSpout]),
            alarms: SwitchSet::empty(),
            progress,
            percentage,
            load0: 0,
            load1: 0,
        }
        .encode(),
    );
    v
}

async fn send(tx: &tokio::sync::mpsc::Sender<Vec<u8>>, v: Vec<u8>) -> Result<(), EcamError> {
    tx.send(v).await.map_err(|_| EcamError::Unknown)
}

pub async fn get_ecam_simulator() -> Result<impl EcamDriver, EcamError> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    const DELAY: Duration = Duration::from_millis(250);
    tokio::spawn(async move {
        // Start in standby
        for _ in 0..5 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::StandBy, 0, 0),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        // Turning on
        for i in 0..5 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::TurningOn, 0, i * 20),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        // Ready
        for _ in 0..3 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::ReadyOrDispensing, 0, 0),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        // Dispensing
        for i in 0..25 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::ReadyOrDispensing, i, i * 4),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        // Ready forever
        for _ in 0..100 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::ReadyOrDispensing, 0, 0),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        Result::<(), EcamError>::Ok(())
    });
    Ok(EcamSimulate { rx: Mutex::new(rx) })
}
