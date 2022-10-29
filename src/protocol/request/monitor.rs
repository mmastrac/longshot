use super::PartialDecode;
use crate::protocol::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorV2Response {
    pub state: MachineEnum<EcamMachineState>,
    pub accessory: MachineEnum<EcamAccessory>,
    pub akey0: u8,
    pub akey1: u8,
    pub akey2: u8,
    pub akey3: u8,
    pub progress: u8,
    pub percentage: u8,
    pub load0: u8,
    pub load1: u8,
}

impl PartialDecode<MonitorV2Response> for MonitorV2Response {
    fn partial_decode(input: &mut &[u8]) -> Option<MonitorV2Response> {
        Some(MonitorV2Response {
            accessory: <MachineEnum<EcamAccessory>>::partial_decode(input)?,
            akey0: <u8>::partial_decode(input)?,
            akey1: <u8>::partial_decode(input)?,
            akey2: <u8>::partial_decode(input)?,
            akey3: <u8>::partial_decode(input)?,
            state: <MachineEnum<EcamMachineState>>::partial_decode(input)?,
            progress: <u8>::partial_decode(input)?,
            percentage: <u8>::partial_decode(input)?,
            load0: <u8>::partial_decode(input)?,
            load1: <u8>::partial_decode(input)?,
        })
    }
}

impl PartialEncode for MonitorV2Response {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push(self.accessory.into());
        out.push(self.akey0);
        out.push(self.akey1);
        out.push(self.akey2);
        out.push(self.akey3);
        out.push(self.state.into());
        out.push(self.progress.into());
        out.push(self.percentage.into());
        out.push(self.load0);
        out.push(self.load1);
    }
}
