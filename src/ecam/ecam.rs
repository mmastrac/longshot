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
}

struct EcamInternals {
    last_status: tokio::sync::watch::Receiver<Option<MonitorState>>,
    ready_lock: Arc<tokio::sync::Semaphore>,
}

impl Ecam {
    pub async fn new(driver: Box<dyn EcamDriver>) -> Self {
        let driver = Arc::new(driver);
        let (tx, rx) = tokio::sync::watch::channel(None);

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
            ready_lock,
        }));
        let ecam = Ecam { driver, internals };
        let driver = ecam.driver.clone();

        tokio::spawn(async move {
            loop {
                if tx.is_closed() {
                    break;
                }
                match driver.read().await? {
                    Some(EcamOutput::Packet(Response::State(x))) => {
                        // println!("{:?}", x);
                        if tx.send(Some(x)).is_err() {
                            break;
                        }
                        ready_lock_semaphore.take();
                    }
                    Some(EcamOutput::Done) => {
                        break;
                    }
                    x => {
                        println!("{:?}", x);
                    }
                }
            }
            println!("Closed");
            Result::<(), EcamError>::Ok(())
        });
        let driver = ecam.driver.clone();
        tokio::spawn(async move {
            loop {
                driver
                    .write(Request::Monitor(MonitorRequestVersion::V2).encode())
                    .await?;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Result::<(), EcamError>::Ok(())
        });
        ecam
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
}
