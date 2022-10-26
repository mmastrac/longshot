use crate::prelude::*;

use tokio::sync::Mutex;

use crate::command::*;
use crate::ecam::{hardware_enums::EcamMachineState, EcamDriver, EcamError, EcamOutput};

#[derive(Debug, PartialEq)]
pub enum EcamStatus {
    Unknown,
    StandBy,
    Ready,
    Busy,
}

impl EcamStatus {
    fn extract(state: &MonitorState) -> EcamStatus {
        if state.state == EcamMachineState::StandBy {
            return EcamStatus::StandBy;
        }
        if state.state == EcamMachineState::ReadyOrDispensing && state.progress == 0 {
            return EcamStatus::Ready;
        }
        return EcamStatus::Busy;
    }

    fn matches(&self, state: &MonitorState) -> bool {
        *self == Self::extract(state)
    }
}

#[derive(Clone)]
pub struct Ecam {
    driver: Arc<Box<dyn EcamDriver>>,
    internals: Arc<Mutex<EcamInternals>>,
    alive: Arc<std::sync::Mutex<bool>>,
}

struct EcamInternals {
    last_status: tokio::sync::watch::Receiver<Option<MonitorState>>,
    packet_tap: Arc<tokio::sync::broadcast::Sender<EcamOutput>>,
    ready_lock: Arc<tokio::sync::Semaphore>,
}

impl Ecam {
    pub async fn new(driver: Box<dyn EcamDriver>) -> Self {
        let driver = Arc::new(driver);
        let (tx, rx) = tokio::sync::watch::channel(None);
        let (txb, _) = tokio::sync::broadcast::channel(100);

        // We want to lock the status until we've received at least one packet
        let ready_lock = Arc::new(tokio::sync::Semaphore::new(1));
        let mut ready_lock_semaphore = Some(
            ready_lock
                .clone()
                .acquire_owned()
                .await
                .expect("Failed to lock mutex"),
        );

        let internals = Arc::new(Mutex::new(EcamInternals {
            last_status: rx,
            packet_tap: Arc::new(txb),
            ready_lock,
        }));
        let ecam_result = Ecam {
            driver,
            internals,
            alive: Arc::new(true.into()),
        };

        let ecam = ecam_result.clone();
        tokio::spawn(async move {
            let txb = ecam.internals.lock().await.packet_tap.clone();
            while ecam.is_alive() && !tx.is_closed() {
                // Treat end-of-stream as EcamOutput::Done, but we might want to reconsider this in the future
                let packet = ecam.driver.read().await?.unwrap_or(EcamOutput::Done);
                let _ = txb.send(packet.clone());
                match packet {
                    EcamOutput::Done => {
                        break;
                    }
                    EcamOutput::Packet(Response::State(x)) => {
                        if tx.send(Some(x)).is_err() {
                            break;
                        }
                        ready_lock_semaphore.take();
                    }
                    _ => {}
                }
            }
            println!("Closed");
            ecam.deaden();
            Result::<(), EcamError>::Ok(())
        });

        let ecam = ecam_result.clone();
        tokio::spawn(async move {
            let status_request = Request::Monitor(MonitorRequestVersion::V2).encode();
            while ecam.is_alive() {
                match tokio::time::timeout(
                    Duration::from_millis(250),
                    ecam.driver.write(status_request.clone()),
                )
                .await
                {
                    Ok(Err(_)) => {
                        println!("Warning: failed to request status");
                    }
                    Err(_) => {
                        println!("Warning: status request send timeout");
                    }
                    _ => {
                        tokio::time::sleep(Duration::from_millis(250)).await;
                    }
                }
            }
            ecam.deaden();
            Result::<(), EcamError>::Ok(())
        });
        ecam_result
    }

    pub async fn wait_for_state(&self, state: EcamStatus) -> Result<(), EcamError> {
        let mut rx = self.internals.lock().await.last_status.clone();
        loop {
            if let Some(test) = rx.borrow().as_ref() {
                if state.matches(test) {
                    return Ok(());
                }
            }
            // TODO: timeout
            rx.changed().await.map_err(|_| EcamError::Unknown)?;
        }
    }

    pub async fn current_state(&self) -> Result<EcamStatus, EcamError> {
        let internals = self.internals.lock().await;
        let rx = internals.last_status.clone();
        drop(
            internals
                .ready_lock
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| EcamError::Unknown)?,
        );
        let ret = if let Some(test) = rx.borrow().as_ref() {
            Ok(EcamStatus::extract(test))
        } else {
            Err(EcamError::Unknown)
        };
        ret
    }

    pub async fn write(&self, request: Request) -> Result<(), EcamError> {
        self.driver.write(request.encode()).await
    }

    pub fn is_alive(&self) -> bool {
        if let Ok(alive) = self.alive.lock() {
            *alive
        } else {
            false
        }
    }

    fn deaden(&self) {
        if let Ok(mut alive) = self.alive.lock() {
            *alive = false;
        }
    }
}

impl Drop for Ecam {
    fn drop(&mut self) {
        self.deaden()
    }
}
