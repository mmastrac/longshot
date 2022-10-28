use crate::protocol::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MonitorV2Response {
    pub state: MachineEnum<EcamMachineState>,
    pub accessory: MachineEnum<EcamAccessory>,
    pub progress: u8,
    pub percentage: u8,
    pub load0: u8,
    pub load1: u8,
}
