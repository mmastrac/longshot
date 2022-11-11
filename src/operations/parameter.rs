use crate::{
    ecam::{Ecam, EcamError, EcamOutput},
    prelude::*,
    protocol::Request,
};

pub async fn read_parameter(ecam: Ecam, param: u16, len: u8) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;
    let ecam = ecam.clone();
    let _handle = tokio::spawn(async move {
        while let Some(packet) = tap.next().await {
            // if dump_decoded_packets {
            trace_packet!("{:?}", packet);
            // }
            if packet == EcamOutput::Done {
                break;
            }
        }
    });

    if len > 4 {
        ecam.write_request(Request::ParameterReadExt(param, len))
            .await?;
    } else {
        ecam.write_request(Request::ParameterRead(param, len))
            .await?;
    }

    while ecam.is_alive() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
