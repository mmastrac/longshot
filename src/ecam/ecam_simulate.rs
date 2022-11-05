use tokio::sync::Mutex;

use crate::ecam::{EcamDriver, EcamDriverOutput, EcamError};
use crate::prelude::*;
use crate::protocol::{
    hexdump, EcamAccessory, EcamDriverPacket, EcamMachineState, EcamMachineSwitch, EcamRequestId,
    MachineEnum, MonitorV2Response, PartialEncode, SwitchSet,
};

struct EcamSimulate {
    rx: Mutex<tokio::sync::mpsc::Receiver<EcamDriverOutput>>,
    tx: Mutex<tokio::sync::mpsc::Sender<EcamDriverOutput>>,
}

impl EcamDriver for EcamSimulate {
    fn read(&self) -> AsyncFuture<Option<EcamDriverOutput>> {
        Box::pin(async {
            let packet = self.rx.lock().await.recv().await;
            Ok(packet)
        })
    }

    fn write(&self, data: crate::protocol::EcamDriverPacket) -> AsyncFuture<()> {
        Box::pin(async move {
            if data.bytes[0] == EcamRequestId::RecipeQuantityRead as u8 {
                // TODO: How do we get rustfmt to format this better?
                let packet = &[
                    166,
                    240,
                    1,
                    data.bytes[3],
                    1,
                    0,
                    40,
                    2,
                    3,
                    8,
                    0,
                    27,
                    4,
                    25,
                    1,
                ];
                self.tx
                    .lock()
                    .await
                    .send(EcamDriverOutput::Packet(EcamDriverPacket::from_slice(
                        packet,
                    )))
                    .await
                    .map_err(eat_errors_with_warning)?;
                trace_packet!("response {:?} -> {:?}", data.bytes, packet);
            }
            if data.bytes[0] == EcamRequestId::RecipeMinMaxSync as u8 {
                // TODO: How do we get rustfmt to format this better?
                let packet = &[
                    176,
                    240,
                    data.bytes[2],
                    1,
                    0,
                    20,
                    0,
                    40,
                    0,
                    180,
                    2,
                    0,
                    3,
                    5,
                    8,
                    0,
                    0,
                    1,
                    24,
                    1,
                    1,
                    1,
                    25,
                    1,
                    1,
                    1,
                    27,
                    0,
                    4,
                    4,
                    28,
                    0,
                    0,
                    0,
                ];
                self.tx
                    .lock()
                    .await
                    .send(EcamDriverOutput::Packet(EcamDriverPacket::from_slice(
                        packet,
                    )))
                    .await
                    .map_err(eat_errors_with_warning)?;
                trace_packet!("response {:?} {:?}", data.bytes, packet);
            }
            Ok(())
        })
    }

    fn alive(&self) -> AsyncFuture<bool> {
        Box::pin(async { Ok(true) })
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
            ..Default::default()
        }
        .encode(),
    );
    v
}

fn eat_errors_with_warning<T: std::fmt::Debug>(e: T) -> EcamError {
    warning!("{:?}", e);
    EcamError::Unknown
}

async fn send_output(
    tx: &tokio::sync::mpsc::Sender<EcamDriverOutput>,
    packet: EcamDriverOutput,
) -> Result<(), EcamError> {
    tx.send(packet).await.map_err(eat_errors_with_warning)
}

async fn send(
    tx: &tokio::sync::mpsc::Sender<EcamDriverOutput>,
    v: Vec<u8>,
) -> Result<(), EcamError> {
    trace_packet!("{}", hexdump(&v));
    send_output(tx, EcamDriverOutput::Packet(EcamDriverPacket::from_vec(v))).await
}

pub async fn get_ecam_simulator() -> Result<impl EcamDriver, EcamError> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    const DELAY: Duration = Duration::from_millis(250);
    send_output(&tx, EcamDriverOutput::Ready).await?;
    let tx_out = tx.clone();
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
        for _ in 0..10 {
            send(
                &tx,
                make_simulated_response(EcamMachineState::ReadyOrDispensing, 0, 0),
            )
            .await?;
            tokio::time::sleep(DELAY).await;
        }

        send_output(&tx, EcamDriverOutput::Done).await?;

        trace_shutdown!("EcamSimulate");
        Result::<(), EcamError>::Ok(())
    });
    Ok(EcamSimulate {
        rx: Mutex::new(rx),
        tx: Mutex::new(tx_out),
    })
}
