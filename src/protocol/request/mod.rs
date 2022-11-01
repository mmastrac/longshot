mod app_control;
mod monitor;
mod profile;
mod recipe;

use super::{hardware_enums::*, MachineEnum};
pub use app_control::*;
pub use monitor::*;
pub use profile::*;
pub use recipe::*;

/// Implements an encode/decode pair for a request or response.
pub trait PartialEncode {
    fn partial_encode(&self, out: &mut Vec<u8>);

    fn encode(&self) -> Vec<u8> {
        let mut v = vec![];
        self.partial_encode(&mut v);
        v
    }
}

impl PartialEncode for u8 {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push(*self);
    }
}

impl PartialEncode for u16 {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push((*self >> 8) as u8);
        out.push(*self as u8);
    }
}

impl<T: PartialEncode> PartialEncode for Vec<T> {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        for t in self.iter() {
            t.partial_encode(out);
        }
    }
}

impl<T> PartialEncode for &MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push((**self).into())
    }
}

pub trait PartialDecode<T> {
    fn partial_decode(input: &mut &[u8]) -> Option<T>;
}

impl<T: PartialDecode<T>> PartialDecode<Vec<T>> for Vec<T> {
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let mut v = vec![];
        while !input.is_empty() {
            v.push(<T>::partial_decode(input)?);
        }
        Some(v)
    }
}

impl<T> PartialDecode<MachineEnum<T>> for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn partial_decode(input: &mut &[u8]) -> Option<Self> {
        let (head, tail) = input.split_first()?;
        *input = tail;
        Some(MachineEnum::decode(*head))
    }
}

impl PartialDecode<u8> for u8 {
    fn partial_decode(input: &mut &[u8]) -> Option<u8> {
        let (head, tail) = input.split_first()?;
        *input = tail;
        Some(*head)
    }
}

impl PartialDecode<u16> for u16 {
    fn partial_decode(input: &mut &[u8]) -> Option<u16> {
        let a = <u8>::partial_decode(input)? as u16;
        let b = <u8>::partial_decode(input)? as u16;
        Some((a << 8) | b)
    }
}

macro_rules! packet_definition {
    (
        $(
            $name:ident
            ( $( $req_name:tt $req_type:ty ),* $(,)? )
            =>
            ( $( $resp_name:tt $resp_type:ty ),* $(,)? )
        ),* $(,)? ) => {

        #[allow(dead_code)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum Request {
            $(
                $name( $($req_type),* ),
            )*
        }

        impl PartialEncode for Request {
            fn partial_encode(&self, mut out: &mut Vec<u8>) {
                match self {
                    $(
                        Self::$name(
                            $(
                                $req_name
                            ),*
                        ) => {
                            out.push(EcamRequestId::$name as u8);
                            if self.is_response_required() {
                                out.push(0xf0);
                            } else {
                                out.push(0x0f);
                            }
                            $($req_name.partial_encode(&mut out); )*
                        }
                    )*
                }
            }
        }

        impl Request {
            pub fn ecam_request_id(&self) -> EcamRequestId {
                match self {
                    $( Self::$name(..) => { EcamRequestId::$name } )*
                }
            }
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum Response {
            $(
                $name ( $($resp_type),* ),
            )*
        }

        impl Response {
            pub fn ecam_request_id(&self) -> EcamRequestId {
                match self {
                    $( Self::$name(..) => { EcamRequestId::$name } )*
                }
            }
        }

        impl PartialDecode<Response> for Response {
            fn partial_decode(input: &mut &[u8]) -> Option<Self> {
                if input.len() < 2 {
                    return None;
                }
                let id = EcamRequestId::try_from(input[0]);
                if let Ok(id) = id {
                    let _ = input[1];
                    *input = &input[2..];
                    match id {
                        $(
                            EcamRequestId::$name => {
                                $(
                                    let $resp_name = <$resp_type>::partial_decode(input)?;
                                )*
                                return Some(Self::$name(
                                    $( $resp_name ),*
                                ));
                            }
                        )*
                    }
                }
                None
            }
        }
    };
}

packet_definition!(
    SetBtMode() => (),
    MonitorV0() => (),
    MonitorV1() => (),
    MonitorV2() => (response MonitorV2Response),
    BeverageDispensingMode(
        recipe MachineEnum<EcamBeverageId>,
        trigger MachineEnum<EcamOperationTrigger>,
        ingredients Vec<RecipeInfo>,
        mode MachineEnum<EcamBeverageTasteType>) => (),
    AppControl(request AppControl) => (),
    ParameterRead() => (),
    ParameterWrite() => (),
    ParameterReadExt() => (),
    StatisticsRead() => (),
    Checksum() => (),
    ProfileNameRead(start u8, end u8) => (names Vec<WideStringWithIcon>),
    ProfileNameWrite() => (),
    RecipeQuantityRead(profile u8, recipe MachineEnum<EcamBeverageId>)
        => (profile u8, recipe MachineEnum<EcamBeverageId>, ingredients Vec<RecipeInfo>),
    RecipePriorityRead() => (priorities Vec<u8>),
    ProfileSelection() => (),
    RecipeNameRead(start u8, end u8) => (names Vec<WideStringWithIcon>),
    RecipeNameWrite() => (),
    SetFavoriteBeverages(profile u8, recipies Vec<u8>) => (),
    RecipeMinMaxSync(recipe MachineEnum<EcamBeverageId>) => (recipe MachineEnum<EcamBeverageId>, bounds Vec<RecipeMinMaxInfo>),
    PinSet() => (),
    BeanSystemSelect() => (),
    BeanSystemRead() => (),
    BeanSystemWrite() => (),
    PinRead() => (),
    SetTime() => (),
);

impl Request {
    fn is_response_required(&self) -> bool {
        !matches!(
            self,
            Request::AppControl(..)
                | Request::MonitorV0()
                | Request::MonitorV1()
                | Request::MonitorV2()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode_monitor_packet() {
        let buf = [117_u8, 15, 1, 5, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        let input = &mut buf.as_slice();
        assert_eq!(
            <Response>::partial_decode(input).expect("Failed to decode"),
            Response::MonitorV2(MonitorV2Response {
                state: EcamMachineState::ReadyOrDispensing.into(),
                accessory: EcamAccessory::Water.into(),
                progress: 0,
                percentage: 0,
                switches: SwitchSet::of(&[
                    EcamMachineSwitch::WaterSpout,
                    EcamMachineSwitch::MotorDown
                ]),
                alarms: SwitchSet::empty(),
                load0: 0,
                load1: 0,
            })
        );
    }

    #[test]
    fn test_decode_monitor_packet_alarm() {
        let buf = [117_u8, 15, 1, 69, 0, 1, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        let input = &mut buf.as_slice();
        assert_eq!(
            <Response>::partial_decode(input).expect("Failed to decode"),
            Response::MonitorV2(MonitorV2Response {
                state: EcamMachineState::ReadyOrDispensing.into(),
                accessory: EcamAccessory::Water.into(),
                progress: 0,
                percentage: 0,
                switches: SwitchSet::of(&[
                    EcamMachineSwitch::WaterSpout,
                    EcamMachineSwitch::MotorDown,
                    EcamMachineSwitch::WaterLevelLow,
                ]),
                alarms: SwitchSet::of(&[EcamAlarm::EmptyWaterTank]),
                load0: 0,
                load1: 0,
            })
        );
    }

    #[test]
    fn test_decode_profile_packet() {
        let buf = [
            164_u8, 240, 0, 77, 0, 97, 0, 116, 0, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0,
            77, 0, 105, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 80, 0, 82, 0, 79, 0,
            70, 0, 73, 0, 76, 0, 69, 0, 32, 0, 51, 0, 0, 3,
        ];
        let input = &mut buf.as_slice();
        assert_eq!(
            <Response>::partial_decode(input).expect("Failed to decode"),
            Response::ProfileNameRead(vec![
                WideStringWithIcon::new("Matt", 3),
                WideStringWithIcon::new("Mia", 8),
                WideStringWithIcon::new("PROFILE 3", 3)
            ])
        )
    }

    #[test]
    fn test_brew_coffee() {
        let recipe = vec![
            RecipeInfo::new(EcamIngredients::Coffee, 103),
            RecipeInfo::new(EcamIngredients::Taste, 2),
            RecipeInfo::new(EcamIngredients::Temp, 0),
        ];
        assert_eq!(
            Request::BeverageDispensingMode(
                EcamBeverageId::RegularCoffee.into(),
                EcamOperationTrigger::Start.into(),
                recipe,
                EcamBeverageTasteType::PrepareInversion.into()
            )
            .encode(),
            vec![0x83, 0xf0, 0x02, 0x01, 0x01, 0x00, 0x67, 0x02, 0x02, 0x00, 0x00, 0x06]
        );
    }
}
