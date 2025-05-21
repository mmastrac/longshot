use std::collections::BTreeMap;

use crate::{
    ecam::{Ecam, EcamError, EcamOutput},
    prelude::*,
    protocol::{Request, Response},
};

pub async fn read_parameter_memory(ecam: Ecam) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;
    let mut last_all_zero = false;
    for i in 0..0x1000 {
        let param = i * 4;
        ecam.write_request(Request::ParameterReadExt(param, 4))
            .await?;
        let now = std::time::Instant::now();
        while now.elapsed() < Duration::from_millis(500) {
            match tokio::time::timeout(Duration::from_millis(50), tap.next()).await {
                Err(_) => {}
                Ok(None) => {
                    eprintln!("No packet received for {:04x}", param);
                    return Err(EcamError::Unknown);
                }
                Ok(Some(x)) => {
                    if let Some(packet) = x.take_packet() {
                        if let Response::ParameterReadExt(param, data) = packet {
                            let all_zero = data.iter().all(|d| *d == 0);
                            if all_zero {
                                if last_all_zero {
                                    break;
                                }
                                last_all_zero = all_zero;
                                println!("...");
                                break;
                            }
                            last_all_zero = all_zero;
                            print!("{:04x}: ", param);
                            for d in &data {
                                print!("{:02x}", d);
                            }
                            print!("  ");
                            for d in &data {
                                if *d >= 32 && *d < 127 {
                                    print!("{}", *d as char);
                                } else {
                                    print!(".");
                                }
                            }
                            println!();
                            break;
                        } else {
                            eprintln!("Unexpected packet: {:?}", packet);
                            return Err(EcamError::Unknown);
                        }
                    } else {
                        eprintln!("No packet received for {:04x}", param);
                        return Err(EcamError::Unknown);
                    }
                }
            }
        }
    }

    Ok(())
}

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

pub async fn read_statistic(ecam: Ecam, param: u16, len: u8) -> Result<(), EcamError> {
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

    ecam.write_request(Request::StatisticsRead(param, len))
        .await?;

    while ecam.is_alive() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Read all statistics from the device. The machine behaves strangely:
///
///  - It will never return invalid statistics, so if you ask for statistic "1"
///    and it doesn't exist, it'll jump to the next valid statistic.
///  - It always behaves as if it read N statistics (N appears to be 9?) and
///    then truncates the list to the length specified. To read all statistics,
///    you need to use the maximum length because asking for the last statistic
///    will result in the machine returning MAX - STAT_BATCH_SIZE as the first one
///    and then truncating to the length specified.
///
/// So what we need to do is:
///
/// Ask for stat 1, length 16. This returns the first statistic clamped to the internal length (9).
/// We then ask for the _last_ statistic in that batch, length 16, which gets the next batch. Continue until
/// we get a response of zero length.
pub async fn read_statistics(ecam: Ecam) -> Result<(), EcamError> {
    let mut tap = ecam.packet_tap().await?;

    // let mut last_stat = 0;
    let mut current_stat = 1;
    const BATCH_SIZE: u8 = 16;

    let mut all_stats = BTreeMap::new();

    loop {
        ecam.write_request(Request::StatisticsRead(current_stat, BATCH_SIZE))
            .await?;
        let now = std::time::Instant::now();
        while now.elapsed() < Duration::from_millis(500) {
            match tokio::time::timeout(Duration::from_millis(50), tap.next()).await {
                Err(_) => {}
                Ok(None) => {
                    eprintln!("No packet received for {:04x}", current_stat);
                    return Err(EcamError::Unknown);
                }
                Ok(Some(x)) => {
                    if let Some(packet) = x.take_packet() {
                        if let Response::StatisticsRead(stats) = packet {
                            if stats.is_empty() {
                                return Ok(());
                            }
                            for stat in &stats {
                                if all_stats.insert(stat.stat, *stat).is_none() {
                                    println!(
                                        "{:>5}: {:08x} ({})",
                                        stat.stat, stat.value, stat.value
                                    );
                                }
                            }
                            current_stat = stats.last().unwrap().stat;
                        }
                    }
                }
            }
        }
    }
}
