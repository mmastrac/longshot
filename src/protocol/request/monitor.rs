use super::PartialDecode;
use crate::protocol::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorV2Response {
    pub state: MachineEnum<EcamMachineState>,
    pub accessory: MachineEnum<EcamAccessory>,
    pub switches: SwitchSet<EcamMachineSwitch>,
    pub alarms: SwitchSet<EcamAlarm>,
    pub progress: u8,
    pub percentage: u8,
    pub load0: u8,
    pub load1: u8,
}

impl<T: MachineEnumerable> PartialDecode<SwitchSet<T>> for SwitchSet<T> {
    fn partial_decode(input: &mut &[u8]) -> Option<SwitchSet<T>> {
        let a = <u8>::partial_decode(input)? as u16;
        let b = <u8>::partial_decode(input)? as u16;
        // Note that this is inverted from <u16>::partial_decode
        Some(SwitchSet::from_u16((b << 8) | a))
    }
}

impl<T: MachineEnumerable> PartialEncode for SwitchSet<T> {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        self.value.partial_encode(out)
    }
}

impl PartialDecode<MonitorV2Response> for MonitorV2Response {
    fn partial_decode(input: &mut &[u8]) -> Option<MonitorV2Response> {
        Some(MonitorV2Response {
            accessory: <MachineEnum<EcamAccessory>>::partial_decode(input)?,
            switches: <SwitchSet<EcamMachineSwitch>>::partial_decode(input)?,
            alarms: <SwitchSet<EcamAlarm>>::partial_decode(input)?,
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
        self.switches.partial_encode(out);
        self.alarms.partial_encode(out);
        out.push(self.state.into());
        out.push(self.progress);
        out.push(self.percentage);
        out.push(self.load0);
        out.push(self.load1);
    }
}

#[cfg(test)]
mod test {
    use crate::protocol::EcamMachineSwitch;

    use super::SwitchSet;

    #[test]
    fn switch_set_test() {
        let switches = SwitchSet::<EcamMachineSwitch>::of(&[]);
        assert_eq!("(empty)", format!("{:?}", switches));
        let switches =
            SwitchSet::of(&[EcamMachineSwitch::MotorDown, EcamMachineSwitch::WaterSpout]);
        assert_eq!("WaterSpout | MotorDown", format!("{:?}", switches));
    }
}
