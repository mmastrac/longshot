use std::vec::Vec;

pub enum Request {
    Brew(BrewRequest),
    Monitor(MonitorRequestVersion),
    State(StateRequest),
    Parameter(ParameterRequest),
    Raw(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum Response {
    State(MonitorState),
    Raw(Vec<u8>),
}

pub enum MonitorRequestVersion {
    V0,
    V1,
    V2,
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

#[derive(Debug, PartialEq)]
pub enum MachineState {
    StandBy,
    TurningOn,
    ShuttingDown,
    Descaling,
    SteamPreparation,
    Recovery,
    Ready,
    /// Working is ready w/a function progress
    Working,
    Rinsing,
    MilkPreparation,
    HotWaterDelivery,
    MilkCleaning,
    Unknown(u8),
}

pub enum Accessory {
    None,
    Water,
    Milk,
    Chocolate,
    MilkClean,
    Unknown(u8),
}

pub enum BeverageTasteType {
    Delete,                  // 0
    Save,                    // 1
    Prepare,                 // 2
    PrepareAndSave,          // 3
    SaveInversion,           // 5
    PrepareInversion,        // 6
    PrepareAndSaveInversion, // 7
}

// pub enum Ingredients {
//     TEMP,                   //(0),
//     COFFEE,                 //(1),
//     TASTE,                  //(2),
//     GRANULOMETRY,           //(3),
//     BLEND,                  //(4),
//     INFUSION_SPEED,         //(5),
//     PREINFUSIONE,           //(6),
//     CREMA,                  //(7),
//     DUExPER,                //(8),
//     MILK,                   //(9),
//     MILK_TEMP,              //(10),
//     MILK_FROTH,             //(11),
//     INVERSION,              //(12),
//     THE_TEMP,               //(13),
//     THE_PROFILE,            //(14),
//     HOT_WATER,              //(15),
//     MIX_VELOCITY,           //(16),
//     MIX_DURATION,           //(17),
//     DENSITY_MULTI_BEVERAGE, //(18),
//     TEMP_MULTI_BEVERAGE,    //(19),
//     DECALC_TYPE,            //(20),
//     TEMP_RISCIACQUO,        //(21),
//     WATER_RISCIACQUO,       //(22),
//     CLEAN_TYPE,             //(23),
//     PROGRAMABLE,            //(24),
//     VISIBLE,                //(25),
//     VISIBLE_IN_PROGRAMMING, //(26),
//     INDEX_LENGTH,           //(27),
//     ACCESSORIO,             //(28);
// }

#[derive(Debug, PartialEq)]
pub struct MonitorState {
    pub state: MachineState,
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

            Request::Raw(r) => r.clone(),
        }
    }
}

impl BrewRequest {
    pub fn encode(&self) -> Vec<u8> {
        // dispense request, 0xf0, beverage type, trigger, parameters*, taste type
        // parameter: coffee quantity, coffee aroma, water quantity, milk quantity, froth
        // COFFEE, 1
        // MILK, 2
        // WATER, 3
        // AROMA, 4
        // TEMPERATURE 5
        // FROTH, 6
        // COFFEE_TYPE, 7
        // COFFEE_GRINDING,

        match *self {
            BrewRequest::Coffee => {
                vec![
                    0x83, 0xf0, 0x02, 0x01, 0, 0x00, 0x67, 0x02, 0x02, 0x00, 0x00, 0x06,
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
            Response::State(MonitorState::decode(&data[2..]))
        } else {
            Response::Raw(data.to_vec())
        }
    }
}

impl MonitorState {
    pub fn decode(data: &[u8]) -> Self {
        /* accessory, sw0, sw1, sw2, sw3, function, function progress, percentage, ?, load0, load1, sw, water */

        // Handle ready/working overlap
        let mut state = MachineState::decode(data[5]);
        if state == MachineState::Ready && data[6] != 0 {
            state = MachineState::Working;
        }

        MonitorState {
            state: MachineState::decode(data[5]),
            progress: data[6],
            percentage: data[7],
            load0: data[8],
            load1: data[9],
            raw: data.to_vec(),
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

impl MachineState {
    pub fn decode(data: u8) -> Self {
        match data {
            0 => MachineState::StandBy,
            1 => MachineState::TurningOn,
            2 => MachineState::ShuttingDown,
            4 => MachineState::Descaling,
            5 => MachineState::SteamPreparation,
            6 => MachineState::Recovery,
            7 => MachineState::Ready,
            8 => MachineState::Rinsing,
            10 => MachineState::MilkPreparation,
            11 => MachineState::HotWaterDelivery,
            12 => MachineState::MilkCleaning,
            n => MachineState::Unknown(n),
        }
    }
}
