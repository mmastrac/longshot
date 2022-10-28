mod app_control;
mod monitor;

use super::{hardware_enums::*, MachineEnum};
use app_control::*;
use monitor::*;

/// Implements an encode/decode pair for a request or response.
trait PartialEncode {
    fn partial_encode(&self, out: &mut Vec<u8>);
}

impl PartialEncode for &u8 {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.push(**self);
    }
}

impl PartialEncode for &Vec<u8> {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}

trait PartialDecode<T> {
    fn partial_decode(input: &mut &[u8]) -> Option<T>;
}

impl PartialDecode<Vec<u8>> for Vec<u8> {
    fn partial_decode(input: &mut &[u8]) -> Option<Vec<u8>> {
        let v = input.to_vec();
        *input = &[];
        Some(v)
    }
}

impl<T: TryFrom<u8>> PartialDecode<MachineEnum<T>> for MachineEnum<T> {
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

/// Implements an encode/decode pair for a request or response.
trait Decode {
    fn try_decode<'a>(bytes: &'a [u8]) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! as_item {
    ($i:item) => {
        $i
    };
}

macro_rules! packet_definition {
    (
        $(
            $name:ident
            ( $( $req_name:tt $req_type:ty ),* $(,)? )
            =>
            ( $( $resp_name:tt $resp_type:ty ),* $(,)? )
        ),* $(,)? ) => {

        #[derive(Clone, Debug, PartialEq)]
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
                            out.push(0x0f);
                            $($req_name .partial_encode(&mut out); )*
                            unimplemented!()
                        }
                    )*
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum Response {
            $(
                $name ( $($resp_type),* ),
            )*
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
    BeverageDispensingMode() => (),
    AppControl(request AppControl) => (),
    ParameterRead() => (),
    ParameterWrite() => (),
    ParameterReadExt() => (),
    StatisticsRead() => (),
    Checksum() => (),
    ProfileNameRead(start u8, end u8) => (),
    ProfileNameWrite() => (),
    RecipeQuantityRead(recipe u8) => (),
    RecipePriorityRead() => (priorities Vec<u8>),
    ProfileSelection() => (),
    RecipeNameRead(start u8, end u8) => (),
    RecipeNameWrite() => (),
    SetFavoriteBeverages(profile u8, recipies Vec<u8>) => (),
    RecipeMinMaxSync(recipe u8) => (),
    PinSet() => (),
    BeanSystemSelect() => (),
    BeanSystemRead() => (),
    BeanSystemWrite() => (),
    PinRead() => (),
    SetTime() => (),
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_monitor_packet() {
        let buf = [117_u8, 15, 1, 5, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        let input = &mut buf.as_slice();
        assert_eq!(
            <Response>::partial_decode(input).expect("Failed to decode"),
            Response::MonitorV2(MonitorV2Response {
                state: EcamMachineState::ReadyOrDispensing.into(),
                accessory: EcamAccessory::Water.into(),
                progress: 0,
                percentage: 0,
                akey0: 5,
                akey1: 0,
                akey2: 0,
                akey3: 0,
                load0: 0,
                load1: 0,
            })
        );
    }
}
