#![allow(dead_code)]

use enum_iterator::Sequence;
use num_enum::{IntoPrimitive, TryFromPrimitive};

///! This file contains validated hardware enumerations and associated values.

/// Ingredients used for brew operations.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamIngredients {
    Temp = 0,                  // TEMP
    Coffee = 1,                // COFFEE
    Taste = 2,                 // TASTE
    Granulometry = 3,          // GRANULOMETRY
    Blend = 4,                 // BLEND
    InfusionSpeed = 5,         // INFUSIONSPEED
    Preinfusion = 6,           // PREINFUSIONE
    Crema = 7,                 // CREMA
    DueXPer = 8,               // DUExPER
    Milk = 9,                  // MILK
    MilkTemp = 10,             // MILKTEMP
    MilkFroth = 11,            // MILKFROTH
    Inversion = 12,            // INVERSION
    TheTemp = 13,              // THETEMP
    TheProfile = 14,           // THEPROFILE
    HotWater = 15,             // HOTWATER
    MixVelocity = 16,          // MIXVELOCITY
    MixDuration = 17,          // MIXDURATION
    DensityMultiBeverage = 18, // DENSITYMULTIBEVERAGE
    TempMultiBeverage = 19,    // TEMPMULTIBEVERAGE
    DecalcType = 20,           // DECALCTYPE
    TempRisciaquo = 21,        // TEMPRISCIACQUO
    WaterRisciaquo = 22,       // WATERRISCIACQUO
    CleanType = 23,            // CLEANTYPE
    Programmable = 24,         // PROGRAMABLE
    Visible = 25,              // VISIBLE
    VisibleInProgramming = 26, // VISIBLEINPROGRAMMING
    IndexLength = 27,          // INDEXLENGTH
    Accessorio = 28,           // ACCESSORIO
}

impl EcamIngredients {
    /// Is this ingredient encoded as two bytes? Unknown encodings return None.
    pub fn is_wide_encoding(&self) -> Option<bool> {
        match self {
            EcamIngredients::Temp
            | EcamIngredients::Taste
            | EcamIngredients::Inversion
            | EcamIngredients::DueXPer
            | EcamIngredients::IndexLength
            | EcamIngredients::Visible
            | EcamIngredients::Programmable
            | EcamIngredients::Accessorio => Some(false),
            EcamIngredients::Coffee | EcamIngredients::Milk | EcamIngredients::HotWater => {
                Some(true)
            }
            _ => None,
        }
    }
}

/// Beverage preparation mode.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
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
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamOperationTrigger {
    DontCare = 0,
    Start = 1,
    /// This is STARTPROGRAM and STOPV2, but only STOPV2 appears to be used.
    StartProgramOrStopV2 = 2,
    NextStep = 3,
    Stop = 4,
    StopProgram = 5,
    ExitProgramOk = 6,
    AdvancedMode = 7,
}

/// Answer and request IDs for packets send to/from the machine.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamRequestId {
    SetBtMode = 17,
    MonitorV0 = 96,
    MonitorV1 = 112,
    MonitorV2 = 117,
    BeverageDispensingMode = 131,
    /// (2, 1) for turn on, (3, 2) for refresh app ID.
    AppControl = 132,
    ParameterRead = 149,
    ParameterWrite = 144,
    ParameterReadExt = 161,
    StatisticsRead = 162,
    Checksum = 163,
    ProfileNameRead = 164,
    ProfileNameWrite = 165,
    RecipeQuantityRead = 166,
    RecipePriorityRead = 168,
    ProfileSelection = 169,
    RecipeNameRead = 170,
    RecipeNameWrite = 171,
    SetFavoriteBeverages = 173,
    /// This may be a PIN operation in some other versions of the protocol.
    RecipeMinMaxSync = 176,
    PinSet = 177,
    BeanSystemSelect = 185,
    BeanSystemRead = 186,
    BeanSystemWrite = 187,
    PinRead = 210,
    SetTime = 226,
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamTemperature {
    Low = 0,
    Mid = 1,
    High = 2,
    VeryHigh = 3,
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamBeverageTaste {
    Preground = 0,
    ExtraMild = 1,
    Mild = 2,
    Normal = 3,
    Strong = 4,
    ExtraStrong = 5,
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
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
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamAccessory {
    None = 0,
    Water = 1,
    Milk = 2,
    Chocolate = 3,
    MilkClean = 4,
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamBeverageId {
    EspressoCoffee = 1,
    RegularCoffee = 2,
    LongCoffee = 3,
    EspressoCoffee2X = 4,
    DoppioPlus = 5,
    Americano = 6,
    Cappuccino = 7,
    LatteMacchiato = 8,
    CaffeLatte = 9,
    FlatWhite = 10,
    EspressoMacchiato = 11,
    HotMilk = 12,
    CappuccinoDoppioPlus = 13,
    ColdMilk = 14,
    CappuccinoReverse = 15,
    HotWater = 16,
    Steam = 17,
    Ciocco = 18,
    Ristretto = 19,
    LongEspresso = 20,
    CoffeeCream = 21,
    Tea = 22,
    CoffeePot = 23,
    Cortado = 24,
    LongBlack = 25,
    TravelMug = 26,
    BrewOverIce = 27,
    Custom01 = 230,
    Custom02 = 231,
    Custom03 = 232,
    Custom04 = 233,
    Custom05 = 234,
    Custom06 = 235,
    Custom07 = 236,
    Custom08 = 237,
    Custom09 = 238,
    Custom10 = 239,
}

/// Represents bits set in the alarm fields.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamAlarm {
    EmptyWaterTank = 0,
    CoffeeWasteContainerFull = 1,
    DescaleAlarm = 2,
    ReplaceWaterFilter = 3,
    CoffeGroundTooFine = 4,
    CoffeeBeansEmpty = 5,
    MachineToService = 6,
    CoffeeHeaterProbeFailure = 7,
    TooMuchCoffee = 8,
    CoffeeInfuserMotorNotWorking = 9,
    EmptyDripTray = 11,
    SteamerProbeFailure = 10,
    TankIsInPosition = 13,
    HydraulicCircuitProblem = 12,
    CoffeeBeansEmptyTwo = 15,
    CleanKnob = 14,
    BeanHopperAbsent = 17,
    TankTooFull = 16,
    InfuserSense = 19,
    GridPresence = 18,
    ExpansionCommProb = 21,
    NotEnoughCoffee = 20,
    GrindingUnit1Problem = 23,
    ExpansionSubmodulesProb = 22,
    CondenseFanProblem = 25,
    GrindingUnit2Problem = 24,
    SpiCommProblem = 27,
    ClockBtCommProblem = 26,
}

/// Represents bits set in the machine switch fields.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive, Eq, Hash, Sequence,
)]
pub enum EcamMachineSwitch {
    WaterSpout = 0,
    MotorUp = 1,
    MotorDown = 2,
    CoffeeWasteContainer = 3,
    WaterTankAbsent = 4,
    Knob = 5,
    WaterLevelLow = 6,
    CoffeeJug = 7,
    IfdCaraffe = 8,
    CioccoTank = 9,
    CleanKnob = 10,
    DoorOpened = 13,
    PregroundDoorOpened = 14,
}
