use tokio::sync::Mutex;

use crate::ecam::{EcamDriver, EcamDriverOutput, EcamError};
use crate::prelude::*;
use crate::protocol::{
    hexdump, EcamAccessory, EcamBeverageId, EcamDriverPacket, EcamMachineState, EcamMachineSwitch,
    EcamRequestId, MonitorV2Response, PartialEncode, SwitchSet,
};

struct EcamSimulate {
    rx: Mutex<tokio::sync::mpsc::Receiver<EcamDriverOutput>>,
    tx: Mutex<tokio::sync::mpsc::Sender<EcamDriverOutput>>,
}

/// These are the recipes the simulator will make
fn get_recipes(beverage: EcamBeverageId) -> Option<(Vec<u8>, Vec<u8>)> {
    use EcamBeverageId::*;

    let (recipe, minmax) = match beverage {
        EspressoCoffee => (
            "010028020308001b041901",
            "010014002800b4020003050800000118010101190101011b0004041c000000",
        ),
        RegularCoffee => (
            "0100b402031b041901",
            "01006400b400f00200030518010101190101011b0004041c000000",
        ),
        LongCoffee => (
            "0100fa02051b041901",
            "01007300a000fa0200030518010101190101011b0004041c000000",
        ),
        EspressoCoffee2X => (
            "010050020308001b041901",
            "01002800500168020003050801010118000000190101011b0004041c000000",
        ),
        DoppioPlus => (
            "01007802011b041901",
            "010050007800b40200010118010101190101011b0004041c000000",
        ),
        Cappuccino => (
            "0100410900be02030c001b0419011c02",
            "010014004100b409003c00be03840200030518010101190101010c0000001c0002001b000404",
        ),
        LatteMacchiato => (
            "01003c0900dc02030c001b0419011c02",
            "010014003c00b409003c00dc03840200030518010101190101010c0000001c0002001b000404",
        ),
        CaffeLatte => (
            "01003c0901f402030c001b0419011c02",
            "010014003c00b409003201f403840200030518010101190101010c0000001c0002001b000404",
        ),
        FlatWhite => (
            "01003c0901f402030c001b0419011c02",
            "010014003c00b409003c01f403840200030518010101190101010c0000001c0002001b000404",
        ),
        EspressoMacchiato => (
            "01001e09003c02030c001b0419011c02",
            "010014001e00b409003c003c03840200030518010101190101010c0000001c0002001b000404",
        ),
        HotMilk => (
            "0901c21c021b041901",
            "09003c01c2038418010101190101011c0002001b000404",
        ),
        CappuccinoDoppioPlus => (
            "0100780900be02010c001b0419011c02",
            "010050007800b409003c00be03840200010118010101190101010c0000001c0002001b000404",
        ),
        CappuccinoReverse => (
            "0100410900be02030c011b0419011c02",
            "010014004100b409003c00be03840200030518010101190101010c0101011c0002001b000404",
        ),
        HotWater => ("0f00fa19011c01", "0f001400fa01a418010101190101011c000100"),
        CoffeePot => (
            "0100fa02030f00001b041901",
            "0100fa00fa00fa18000000020003050f000000000000190101011b000404",
        ),
        Cortado => (
            "01006402000f00001b041901",
            "010028006400f018010101020003050f000000000000190101011b000404",
        ),
        Custom01 => (
            "0100b409000002050c001c001b041901",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        Custom02 => (
            "01002809000002050c001c001b041901",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        Custom03 => (
            "01000009000002030c001c001b041900",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        Custom04 => (
            "0100500900a002030c001c001b041900",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        Custom05 => (
            "0100500900a002030c001c001b041900",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        Custom06 => (
            "0100500900a002030c001c001b041900",
            "010014005000b409003200a003840200030518010101190000000c0000011c0000001b000404",
        ),
        _ => {
            return None;
        }
    };

    Some((
        hex::decode(recipe).expect("Failed to decode constant"),
        hex::decode(minmax).expect("Failed to decode constant"),
    ))
}

impl EcamDriver for EcamSimulate {
    fn read(&self) -> AsyncFuture<Option<EcamDriverOutput>> {
        Box::pin(async {
            let packet = self.rx.lock().await.recv().await;
            Ok(packet)
        })
    }

    fn write(&self, data: crate::protocol::EcamDriverPacket) -> AsyncFuture<()> {
        trace_packet!("{{host->device}} {}", hexdump(&data.bytes));
        Box::pin(async move {
            if data.bytes[0] == EcamRequestId::RecipeQuantityRead as u8 {
                let mut packet = vec![data.bytes[0], 0xf0, 1, data.bytes[3]];
                if let Ok(beverage) = data.bytes[3].try_into() {
                    if let Some((recipe, _)) = get_recipes(beverage) {
                        packet = [packet, recipe].concat();
                    }
                }
                send(&*self.tx.lock().await, packet).await?;
            }
            if data.bytes[0] == EcamRequestId::RecipeMinMaxSync as u8 {
                let mut packet = vec![data.bytes[0], 0xf0, data.bytes[2]];
                if let Ok(beverage) = data.bytes[2].try_into() {
                    if let Some((_, minmax)) = get_recipes(beverage) {
                        packet = [packet, minmax].concat();
                    }
                }
                send(&*self.tx.lock().await, packet).await?;
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
            state: state.into(),
            accessory: EcamAccessory::None.into(),
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

pub async fn get_ecam_simulator(simulator: &str) -> Result<impl EcamDriver, EcamError> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    const DELAY: Duration = Duration::from_millis(250);
    send_output(&tx, EcamDriverOutput::Ready).await?;
    let tx_out = tx.clone();
    let on = simulator.ends_with("[on]");
    trace_packet!("Initializing simulator: {}", simulator);
    tokio::spawn(async move {
        if !on {
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
