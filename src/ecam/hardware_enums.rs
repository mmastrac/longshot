#![allow(dead_code)]

use num_enum::{IntoPrimitive, TryFromPrimitive};

///! This file contains validated hardware enumerations and associated values.

/// Ingredients used for brew operations.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamIngredients {
    Temp = 0,                  // TEMP
    Coffee = 1,                // COFFEE
    Taste = 2,                 // TASTE
    Granulometry = 3,          // GRANULOMETRY
    Blend = 4,                 // BLEND
    InfusionSpeed = 5,         // INFUSION_SPEED
    Preinfusion = 6,           // PREINFUSIONE
    Crema = 7,                 // CREMA
    DueXPer = 8,               // DUExPER
    Milk = 9,                  // MILK
    MilkTemp = 10,             // MILK_TEMP
    MilkFroth = 11,            // MILK_FROTH
    Inversion = 12,            // INVERSION
    TheTemp = 13,              // THE_TEMP
    TheProfile = 14,           // THE_PROFILE
    HotWater = 15,             // HOT_WATER
    MixVelocity = 16,          // MIX_VELOCITY
    MixDuration = 17,          // MIX_DURATION
    DensityMultiBeverage = 18, // DENSITY_MULTI_BEVERAGE
    TempMultiBeverage = 19,    // TEMP_MULTI_BEVERAGE
    DecalcType = 20,           // DECALC_TYPE
    TempRisciaquo = 21,        // TEMP_RISCIACQUO
    WaterRisciaquo = 22,       // WATER_RISCIACQUO
    CleanType = 23,            // CLEAN_TYPE
    Programmable = 24,         // PROGRAMABLE
    Visible = 25,              // VISIBLE
    VisibleInProgramming = 26, // VISIBLE_IN_PROGRAMMING
    IndexLength = 27,          // INDEX_LENGTH
    Accessorio = 28,           // ACCESSORIO
}

/// Beverage preparation mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamBeverageTasteType {
    Delete = 0,
    Save = 1,
    Prepare = 2,
    PrepareAndSave = 3,
    SaveInversion = 5,
    PrepareInversion = 6,
    PrepareAndSaveInversion = 7,
}

/// Operation mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamOperationTrigger {
    DontCare = 0,
    Start = 1,
    /// This is START_PROGRAM and STOPV2, but only STOPV2 appears to be used.
    StartProgramOrStopV2 = 2,
    NextStep = 3,
    Stop = 4,
    StopProgram = 5,
    ExitProgramOk = 6,
    AdvancedMode = 7,
}

/// Answer and request IDs for packets send to/from the machine.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamRequestId {
    SetBtMode = 17,
    Data0 = 96,
    Data1 = 112,
    Data2 = 117,
    BeverageDispensingMode = 130,
    ParameterRead = 149,
    ParameterWrite = 144,
    ParameterReadExt = 161,
    StatisticsRead = 162,
    Checksum = 163,
    ProfileNameRead = 164,
    ProfileNameWrite = 165,
    RecipeQtyRead = 166,
    RecipePriorityRead = 168,
    ProfileSelection = 169,
    RecipeNameRead = 170,
    RecipeNameWrite = 171,
    PinActivation = 176,
    PinSet = 177,
    SetTime = 226,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamTemperature {
    Low = 0,
    Mid = 1,
    High = 2,
    VeryHigh = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamMachineState {
    StandBy = 0,
    TurningOn = 1,
    ShuttingDown = 2,
    Descaling = 4,
    SteamPreparation = 5,
    Recovery = 6,
    ReadyOrDispensing = 7,
    Rinsing = 8,
    MilkPreparation = 10,
    HotWaterDelivery = 11,
    MilkCleaning = 12,
    ChocolatePreparation = 16,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
pub enum EcamAccessory {
    None = 0,
    Water = 1,
    Milk = 2,
    Chocolate = 3,
    MilkClean = 4,
}
