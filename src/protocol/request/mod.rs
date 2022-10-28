

mod app_control;
mod monitor;

use app_control::*;
use monitor::*;
use super::hardware_enums::*;

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

/// Implements an encode/decode pair for a request or response.
trait Decode {
    fn try_decode<'a> (bytes: &'a [u8]) -> Option<Self> where Self: Sized;
}

macro_rules! as_item {
    ($i:item) => { $i };
}

macro_rules! packet_definition {
    ( 
        $(
            $name:ident 
            ( $( $req_name:tt $req_type:ty ),* $(,)? ) 
            => 
            ( $( $resp_name:tt $resp_type:ty ),* $(,)? ) 
        ),* $(,)? ) => {

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

        pub enum Response {
            $(
                $name ( $($resp_type),* ),
            )*
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
    SetTime() => (),
);
