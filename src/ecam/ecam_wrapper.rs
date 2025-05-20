use crate::prelude::*;

use tokio::sync::{Mutex, OwnedSemaphorePermit};
use tokio_stream::wrappers::BroadcastStream;

use crate::ecam::{EcamDriver, EcamDriverOutput, EcamError};
use crate::protocol::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EcamStatus {
    StandBy,
    TurningOn(usize),
    ShuttingDown(usize),
    Ready,
    Busy(usize),
    Cleaning(usize),
    Descaling,
    Alarm(MachineEnum<EcamMachineAlarm>),
    Fetching(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EcamOutput {
    Ready,
    Packet(EcamPacket<Response>),
    Done,
}

impl EcamOutput {
    /// Takes the underlying packet, if it exists.
    pub fn take_packet(self) -> Option<Response> {
        if let Self::Packet(EcamPacket {
            representation: r, ..
        }) = self
        {
            r
        } else {
            None
        }
    }
}

impl From<EcamDriverOutput> for EcamOutput {
    fn from(other: EcamDriverOutput) -> Self {
        match other {
            EcamDriverOutput::Done => EcamOutput::Done,
            EcamDriverOutput::Ready => EcamOutput::Ready,
            EcamDriverOutput::Packet(p) => EcamOutput::Packet(p.into()),
        }
    }
}

impl From<EcamOutput> for EcamDriverOutput {
    fn from(other: EcamOutput) -> EcamDriverOutput {
        match other {
            EcamOutput::Done => EcamDriverOutput::Done,
            EcamOutput::Ready => EcamDriverOutput::Ready,
            EcamOutput::Packet(p) => EcamDriverOutput::Packet(p.into()),
        }
    }
}

impl EcamStatus {
    pub fn extract(state: &MonitorV2Response) -> EcamStatus {
        if state.state == EcamMachineState::TurningOn {
            return EcamStatus::TurningOn(state.percentage as usize);
        }
        if state.state == EcamMachineState::ShuttingDown {
            if state.percentage < 100 {
                return EcamStatus::ShuttingDown(state.percentage as usize);
            }
            // Emulate status % using progress
            return EcamStatus::ShuttingDown((state.progress as usize * 10).clamp(0, 100));
        }
        if state.state == EcamMachineState::MilkCleaning || state.state == EcamMachineState::Rinsing
        {
            return EcamStatus::Cleaning(state.percentage as usize);
        }
        if state.state == EcamMachineState::MilkPreparation
            || state.state == EcamMachineState::HotWaterDelivery
            || (state.state == EcamMachineState::ReadyOrDispensing && state.progress != 0)
        {
            return EcamStatus::Busy(state.percentage as usize);
        }
        if state.state == EcamMachineState::Descaling {
            return EcamStatus::Descaling;
        }
        #[allow(clippy::never_loop)]
        for alarm in state.alarms.set() {
            if alarm != MachineEnum::Value(EcamMachineAlarm::CleanKnob) {
                return EcamStatus::Alarm(alarm);
            }
        }
        if state.state == EcamMachineState::StandBy {
            return EcamStatus::StandBy;
        }
        EcamStatus::Ready
    }

    fn matches(&self, state: &MonitorV2Response) -> bool {
        *self == Self::extract(state)
    }
}

struct StatusInterest {
    count: Arc<std::sync::Mutex<usize>>,
}

struct StatusInterestHandle {
    count: Arc<std::sync::Mutex<usize>>,
}

/// Internal flag indicating there is interest in the status of the machine.
impl StatusInterest {
    fn new() -> Self {
        StatusInterest {
            count: Arc::new(std::sync::Mutex::new(0)),
        }
    }

    fn lock(&mut self) -> StatusInterestHandle {
        *self.count.lock().unwrap() += 1;
        StatusInterestHandle {
            count: self.count.clone(),
        }
    }

    fn count(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

impl Drop for StatusInterestHandle {
    fn drop(&mut self) {
        *self.count.lock().unwrap() -= 1;
    }
}

/// Internal struct determining if the interface is still alive.
#[derive(Clone)]
struct Alive(Arc<std::sync::Mutex<bool>>);

impl Alive {
    fn new() -> Self {
        Self(Arc::new(std::sync::Mutex::new(true)))
    }

    fn is_alive(&self) -> bool {
        if let Ok(alive) = self.0.lock() {
            *alive
        } else {
            false
        }
    }

    fn deaden(&self) {
        if let Ok(mut alive) = self.0.lock() {
            if *alive {
                trace_shutdown!("Alive::deaden");
            }
            *alive = false;
        }
    }
}

struct EcamDropHandle {
    alive: Alive,
}

impl Drop for EcamDropHandle {
    fn drop(&mut self) {
        trace_shutdown!("Ecam (dropped)");
        self.alive.deaden()
    }
}

/// Handle that gives a user access to a machine. When all clones are dropped, the connection is closed.
#[derive(Clone)]
pub struct Ecam {
    driver: Arc<Box<dyn EcamDriver>>,
    internals: Arc<Mutex<EcamInternals>>,
    alive: Alive,
    #[allow(unused)]
    drop_handle: Arc<EcamDropHandle>,
}

struct EcamInternals {
    last_status: tokio::sync::watch::Receiver<Option<MonitorV2Response>>,
    packet_tap: Arc<tokio::sync::broadcast::Sender<EcamOutput>>,
    ready_lock: Arc<tokio::sync::Semaphore>,
    status_interest: StatusInterest,
    dump_packets: bool,
    started: bool,
}

impl Ecam {
    pub async fn new(driver: Box<dyn EcamDriver>, dump_packets: bool) -> Self {
        let driver = Arc::new(driver);
        let (tx, rx) = tokio::sync::watch::channel(None);
        let (txb, _) = tokio::sync::broadcast::channel(100);

        // We want to lock the status until we've received at least one packet
        let ready_lock = Arc::new(tokio::sync::Semaphore::new(1));
        let ready_lock_semaphore = Some(
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
            status_interest: StatusInterest::new(),
            started: false,
            dump_packets,
        }));
        let alive = Alive::new();
        let ecam_result = Ecam {
            driver,
            internals,
            drop_handle: Arc::new(EcamDropHandle {
                alive: alive.clone(),
            }),
            alive,
        };

        tokio::spawn(Self::operation_loop(
            ready_lock_semaphore,
            tx,
            ecam_result.driver.clone(),
            ecam_result.internals.clone(),
            ecam_result.alive.clone(),
        ));
        let (driver, alive) = (ecam_result.driver.clone(), ecam_result.alive.clone());
        tokio::spawn(Self::alive_watch(driver, alive));
        ecam_result
    }

    async fn alive_watch(driver: Arc<Box<dyn EcamDriver>>, alive: Alive) -> Result<(), EcamError> {
        while let Ok(b) = driver.alive().await {
            if !alive.is_alive() || !b {
                break;
            }
            // Don't spin on this if the alive check is cheap (ie: EcamSimulator)
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        trace_shutdown!("Ecam::alive_watch()");
        alive.deaden();
        Ok(())
    }

    async fn operation_loop(
        mut ready_lock_semaphore: Option<OwnedSemaphorePermit>,
        tx: tokio::sync::watch::Sender<Option<MonitorV2Response>>,
        driver: Arc<Box<dyn EcamDriver>>,
        internals: Arc<Mutex<EcamInternals>>,
        alive: Alive,
    ) -> Result<(), EcamError> {
        let packet_tap_sender = internals.lock().await.packet_tap.clone();
        let dump_packets = internals.lock().await.dump_packets;
        let mut started = false;
        while alive.is_alive() {
            // Treat end-of-stream as EcamOutput::Done, but we might want to reconsider this in the future
            let packet: EcamOutput = driver
                .read()
                .await?
                .unwrap_or(EcamDriverOutput::Done)
                .into();
            let _ = packet_tap_sender.send(packet.clone());
            if dump_packets {
                trace_packet!("{:?}", packet);
            }
            match packet {
                EcamOutput::Ready => {
                    if started {
                        warning!("Got multiple start requests");
                    } else {
                        tokio::spawn(Self::write_monitor_loop(
                            driver.clone(),
                            internals.clone(),
                            alive.clone(),
                        ));
                        started = true;
                        internals.lock().await.started = true;
                    }
                }
                EcamOutput::Done => {
                    trace_shutdown!("Ecam::operation_loop (Done)");
                    break;
                }
                EcamOutput::Packet(EcamPacket {
                    representation: Some(Response::MonitorV2(x)),
                    ..
                }) => {
                    if tx.send(Some(x)).is_err() {
                        warning!("Failed to send a monitor response");
                        break;
                    }
                    ready_lock_semaphore.take();
                }
                _ => {}
            }
        }
        trace_shutdown!("Ecam::operation_loop");
        alive.deaden();
        Ok(())
    }

    /// Is this ECAM still alive?
    pub fn is_alive(&self) -> bool {
        self.alive.is_alive()
    }

    /// Blocks until the device state reaches our desired state.
    pub async fn wait_for_state(
        &self,
        state: EcamStatus,
        monitor: fn(EcamStatus) -> (),
    ) -> Result<(), EcamError> {
        self.wait_for(|status| state.matches(status), monitor).await
    }

    /// Blocks until the device state is not in the undesired state.
    pub async fn wait_for_not_state(
        &self,
        state: EcamStatus,
        monitor: fn(EcamStatus) -> (),
    ) -> Result<(), EcamError> {
        self.wait_for(|status| !state.matches(status), monitor)
            .await
    }

    /// Blocks until the state test function returns true.
    pub async fn wait_for<F>(&self, f: F, monitor: fn(EcamStatus) -> ()) -> Result<(), EcamError>
    where
        F: Fn(&MonitorV2Response) -> bool,
    {
        let alive = self.alive.clone();
        let mut internals = self.internals.lock().await;
        let mut rx = internals.last_status.clone();
        let status_interest = internals.status_interest.lock();
        drop(internals);
        while alive.is_alive() {
            if let Some(test) = rx.borrow().as_ref() {
                monitor(EcamStatus::extract(test));
                if f(test) {
                    drop(status_interest);
                    return Ok(());
                }
            }
            // TODO: timeout
            rx.changed().await.map_err(|_| EcamError::Unknown)?;
        }
        Err(EcamError::Unknown)
    }

    /// Wait for the connection to establish, but not any particular state.
    pub async fn wait_for_connection(&self) -> Result<(), EcamError> {
        let _ = self.current_state().await?;
        Ok(())
    }

    /// Returns the current state, or blocks if we don't know what the current state is yet.
    pub async fn current_state(&self) -> Result<EcamStatus, EcamError> {
        let mut internals = self.internals.lock().await;
        let status_interest = internals.status_interest.lock();
        let rx = internals.last_status.clone();
        let ready_lock = internals.ready_lock.clone();
        drop(internals);
        drop(
            ready_lock
                .acquire_owned()
                .await
                .map_err(|_| EcamError::Unknown)?,
        );
        let ret = if let Some(test) = rx.borrow().as_ref() {
            Ok(EcamStatus::extract(test))
        } else {
            Err(EcamError::Unknown)
        };
        drop(status_interest);
        ret
    }

    pub async fn write(&self, packet: EcamPacket<Request>) -> Result<(), EcamError> {
        let internals = self.internals.lock().await;
        if !internals.started {
            warning!("Packet sent before device was ready!");
        }
        drop(internals);
        self.driver.write(packet.into()).await
    }

    /// Convenience method to skip the EcamPacket.
    pub async fn write_request(&self, r: Request) -> Result<(), EcamError> {
        self.write(EcamPacket::from_represenation(r)).await
    }

    pub async fn packet_tap(&self) -> Result<impl Stream<Item = EcamOutput> + use<>, EcamError> {
        let internals = self.internals.lock().await;
        Ok(BroadcastStream::new(internals.packet_tap.subscribe())
            .map(|x| x.expect("Unexpected receive error")))
    }

    /// The monitor loop is booted when the underlying driver reports that it is ready.
    async fn write_monitor_loop(
        driver: Arc<Box<dyn EcamDriver>>,
        internals: Arc<Mutex<EcamInternals>>,
        alive: Alive,
    ) -> Result<(), EcamError> {
        let status_request = EcamDriverPacket::from_vec(Request::MonitorV2().encode());
        while alive.is_alive() {
            // Only send status update packets while there is status interest
            if internals.lock().await.status_interest.count() == 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            match tokio::time::timeout(
                Duration::from_millis(250),
                driver.write(status_request.clone()),
            )
            .await
            {
                Ok(Err(_)) => {
                    warning!("Failed to request status");
                }
                Err(_) => {
                    warning!("Status request send timeout");
                }
                _ => {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
        }
        trace_shutdown!("Ecam::write_monitor_loop()");
        alive.deaden();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(EcamStatus::Busy(0), &crate::protocol::test::RESPONSE_STATUS_CAPPUCCINO_MILK)]
    #[case(EcamStatus::Cleaning(9), &crate::protocol::test::RESPONSE_STATUS_CLEANING_AFTER_CAPPUCCINO)]
    // We removed the need to test the CleanKnob alarm since it's technically a warning - should handle this better
    // #[case(EcamStatus::Alarm(EcamMachineAlarm::CleanKnob.into()), &crate::protocol::test::RESPONSE_STATUS_READY_AFTER_CAPPUCCINO)]
    #[case(EcamStatus::StandBy, &crate::protocol::test::RESPONSE_STATUS_STANDBY_NO_ALARMS)]
    #[case(EcamStatus::StandBy, &crate::protocol::test::RESPONSE_STATUS_STANDBY_NO_WATER_TANK)]
    #[case(EcamStatus::StandBy, &crate::protocol::test::RESPONSE_STATUS_STANDBY_WATER_SPOUT)]
    #[case(EcamStatus::StandBy, &crate::protocol::test::RESPONSE_STATUS_STANDBY_NO_COFFEE_CONTAINER)]
    #[case(EcamStatus::ShuttingDown(10), &crate::protocol::test::RESPONSE_STATUS_SHUTTING_DOWN_1)]
    #[case(EcamStatus::ShuttingDown(30), &crate::protocol::test::RESPONSE_STATUS_SHUTTING_DOWN_2)]
    #[case(EcamStatus::ShuttingDown(60), &crate::protocol::test::RESPONSE_STATUS_SHUTTING_DOWN_3)]
    fn decode_ecam_status(#[case] expected_status: EcamStatus, #[case] bytes: &[u8]) {
        let response = Response::decode(unwrap_packet(bytes))
            .0
            .expect("Expected to decode a response");
        if let Response::MonitorV2(response) = response {
            let status = EcamStatus::extract(&response);
            assert_eq!(status, expected_status);
        }
    }
}
