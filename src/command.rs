use crate::ecam::hardware_enums::*;
use crate::ecam::machine_enum::MachineEnum;
use std::{fmt::Debug, vec::Vec};

pub enum Request {
    Brew(BrewRequest),
    Monitor(MonitorRequestVersion),
    State(StateRequest),
    Parameter(ParameterRequest),
    Profile(ProfileRequest),
    Raw(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    State(MonitorState),
    Raw(Vec<u8>),
}

pub enum MonitorRequestVersion {
    V0,
    V1,
    V2,
}

pub enum ProfileRequest {
    GetProfileNames(u8, u8),
    GetRecipeNames(u8, u8),
    /// Retrieves the quantity of ingredients for a given profile/recipe.
    GetRecipeQuantities(u8, u8),
    /// Retrieve a list of recipes, in priority (most-used) order.
    GetRecipePriority(u8),
}

pub enum StateRequest {
    TurnOn,
}

pub enum BrewRequest {
    Coffee,
}

pub enum ParameterRequest {
    ReadParameter(ParameterId, u8),
    WriteParameter(ParameterId),
}

pub enum ParameterId {
    WaterHardness,
}

pub enum Strength {}

pub enum Size {}

#[derive(Clone, Debug, PartialEq)]
pub struct MonitorState {
    pub state: MachineEnum<EcamMachineState>,
    pub accessory: MachineEnum<EcamAccessory>,
    pub progress: u8,
    pub percentage: u8,
    pub load0: u8,
    pub load1: u8,
    pub raw: Vec<u8>,
}

impl Request {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Request::Brew(r) => r.encode(),
            Request::Monitor(r) => r.encode(),
            Request::State(r) => r.encode(),
            Request::Parameter(r) => r.encode(),
            Request::Profile(r) => r.encode(),
            Request::Raw(r) => r.clone(),
        }
    }
}

impl BrewRequest {
    pub fn encode(&self) -> Vec<u8> {
        // dispense request, 0xf0, beverage type, trigger, parameters*, taste type
        // parameter: coffee quantity, coffee aroma, water quantity, milk quantity, froth
        match *self {
            BrewRequest::Coffee => {
                vec![
                    0x83, 0xf0, 0x02, 0x01, 0x01, 0x00, 0x67, 0x02, 0x02, 0x00, 0x00, 0x06,
                ]
            }
        }
    }
}

impl MonitorRequestVersion {
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            MonitorRequestVersion::V0 => {
                vec![0x60, 0x0f]
            }
            MonitorRequestVersion::V1 => {
                vec![0x70, 0x0f]
            }
            MonitorRequestVersion::V2 => {
                vec![0x75, 0x0f]
            }
        }
    }
}

impl ParameterRequest {
    pub fn encode(&self) -> Vec<u8> {
        unimplemented!();
    }
}

impl ProfileRequest {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::GetProfileNames(a, b) => {
                vec![EcamRequestId::ProfileNameRead.into(), 0xf0, *a, *b]
            }
            Self::GetRecipeNames(a, b) => {
                vec![EcamRequestId::RecipeNameRead.into(), 0xf0, *a, *b]
            }
            Self::GetRecipeQuantities(a, b) => {
                vec![EcamRequestId::RecipeQtyRead.into(), 0xf0, *a, *b]
            }
            Self::GetRecipePriority(a) => {
                vec![EcamRequestId::RecipePriorityRead.into(), 0xf0, *a]
            }
        }
    }
}

impl StateRequest {
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            StateRequest::TurnOn => {
                vec![0x84, 0x0f, 0x02, 0x01]
            }
        }
    }
}

impl Response {
    pub fn decode(data: &[u8]) -> Self {
        if data[0] == 0x75 && data.len() > 10 {
            Response::State(MonitorState::decode(&data))
        } else {
            Response::Raw(data.to_vec())
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Response::State(s) => s.raw.clone(),
            Response::Raw(v) => v.clone(),
        }
    }
}

impl MonitorState {
    pub fn decode(data: &[u8]) -> Self {
        /* accessory, sw0, sw1, sw2, sw3, function, function progress, percentage, ?, load0, load1, sw, water */
        let raw = data.to_vec();
        let data = &data[2..];
        MonitorState {
            state: MachineEnum::decode(data[5]),
            accessory: MachineEnum::decode(data[0]),
            progress: data[6],
            percentage: data[7],
            load0: data[8],
            load1: data[9],
            raw,
        }

        // progress 5 = water 3 = hot wter

        /*

            <string name="COFFEE_DISPENSING_STATUS_0">Ready to use</string>
            <string name="COFFEE_DISPENSING_STATUS_1">Select beverage</string>
            <string name="COFFEE_DISPENSING_STATUS_11">Delivery</string>
            <string name="COFFEE_DISPENSING_STATUS_14">Brewing unit moving</string>
            <string name="COFFEE_DISPENSING_STATUS_16">End</string>
            <string name="COFFEE_DISPENSING_STATUS_3">Brewing unit moving</string>
            <string name="COFFEE_DISPENSING_STATUS_4">Grinding</string>
            <string name="COFFEE_DISPENSING_STATUS_6">Brewing unit moving</string>
            <string name="COFFEE_DISPENSING_STATUS_7">Water heating up</string>
            <string name="COFFEE_DISPENSING_STATUS_8">Pre-infusion</string>
        */
    }
}
