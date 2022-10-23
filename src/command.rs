use std::vec::Vec;

pub enum Request {
    Brew(BrewRequest),
    Monitor(MonitorRequestVersion),
    State(StateRequest),
    Parameter(ParameterRequest),
    RawRequest(Vec<u8>),
}

pub enum Response {
    State(),
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
    Coffee(),
}

pub enum ParameterRequest {
    ReadParameter(ParameterId, u8),
    WriteParameter(ParameterId),
}

pub enum ParameterId {
    WATER_HARDNESS,
}

pub enum Strength {}

pub enum Size {}

impl Request {
    pub fn encode(self: &Self) -> Vec<u8> {
        match self {
            Request::Brew(r) => r.encode(),
            Request::Monitor(r) => r.encode(),
            Request::State(r) => r.encode(),
            Request::Parameter(r) => r.encode(),

            Request::RawRequest(r) => r.clone(),
        }
    }
}

impl BrewRequest {
    pub fn encode(self: &Self) -> Vec<u8> {
        match *self {
            BrewRequest::Coffee() => {
                vec![
                    0x83, 0xf0, 0x02, 0x01, 0x01, 0x00, 0x67, 0x02, 0x02, 0x00, 0x00, 0x06,
                ]
            }
        }
    }
}

impl MonitorRequestVersion {
    pub fn encode(self: &Self) -> Vec<u8> {
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
    pub fn encode(self: &Self) -> Vec<u8> {
        unimplemented!();
    }
}

impl StateRequest {
    pub fn encode(self: &Self) -> Vec<u8> {
        match *self {
            StateRequest::TurnOn => {
                vec![0x84, 0x0f, 0x02, 0x01]
            }
        }
    }
}
