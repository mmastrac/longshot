#![allow(dead_code)]
use super::MachineEnumerable;
use enum_iterator::Sequence;
use num_enum::{IntoPrimitive, TryFromPrimitive};

///! This file contains validated hardware enumerations and associated values.

macro_rules! hardware_enum {
    ($comment:literal, $name:ident { $($(# [ doc = $item_comment:literal ])? $x:ident = $v:literal,)* } ) => {
        #[doc=$comment]
        #[repr(u8)]
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Ord,
            IntoPrimitive,
            TryFromPrimitive,
            Eq,
            Hash,
            Sequence,
        )]
        pub enum $name { $($(#[doc=$item_comment])? $x = $v),* }

        impl $name {
            pub fn all() -> impl Iterator<Item=$name> {
                enum_iterator::all()
            }

            pub fn to_arg_string(&self) -> String {
                format!("{:?}", self).to_ascii_lowercase()
            }

            pub fn lookup_by_name_case_insensitive(s: &str) -> Option<$name> {
                // TODO: Can use one of the static ToString crates to improve this
                enum_iterator::all().find(|e| format!("{:?}", e).to_ascii_lowercase() == s.to_ascii_lowercase())
            }

            pub fn lookup_by_name(s: &str) -> Option<$name> {
                // TODO: Can use one of the static ToString crates to improve this
                enum_iterator::all().find(|e| format!("{:?}", e) == s)
            }
        }

        impl MachineEnumerable for $name {
            fn to_arg_string(&self) -> String {
                match *self {
                    $(Self::$x => {
                        stringify!($x).to_ascii_lowercase()
                    })*
                }
            }
        }
    };
}

hardware_enum! {"Ingredients used for brew operations.", EcamIngredients {
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
}}

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

hardware_enum! {"Beverage preparation mode.", EcamBeverageTasteType {
    Delete = 0,
    Save = 1,
    /// Prepare a beverage. This is the most likely enumeration value you'll want to use.
    Prepare = 2,
    PrepareAndSave = 3,
    SaveInversion = 5,
    PrepareInversion = 6,
    PrepareAndSaveInversion = 7,
}}

hardware_enum! {"Operation mode/trigger.", EcamOperationTrigger {
    DontCare = 0,
    /// Start preparing a beverage. This is the most likely enumeration value you'll want to use.
    Start = 1,
    /// This is STARTPROGRAM and STOPV2, but only STOPV2 appears to be used.
    StartProgramOrStopV2 = 2,
    NextStep = 3,
    Stop = 4,
    StopProgram = 5,
    ExitProgramOk = 6,
    AdvancedMode = 7,
}}

hardware_enum! {"Identifier determining the type of request and response (also referred to as the 'answer ID').", EcamRequestId {
    SetBtMode = 17,
    MonitorV0 = 96,
    MonitorV1 = 112,
    /// Send a monitor V2 packet to the machine. This is the only tested and working monitor functionality.
    MonitorV2 = 117,
    /// Brew a beverage, or interact with the profile saving functionality.
    BeverageDispensingMode = 131,
    /// (2, 1) for turn on, (3, 2) for refresh app ID.
    AppControl = 132,
    /// Read a parameter from the device. Used for reads less than or equal to 4 blocks, less than or equal to 10 blocks (each block is 2 bytes).
    ParameterRead = 149,
    ParameterWrite = 144,
    /// Read a parameter from the device. Used for reads longer than 4 blocks, less than or equal to 10 blocks (each block is 2 bytes).
    ParameterReadExt = 161,
    StatisticsRead = 162,
    Checksum = 163,
    ProfileNameRead = 164,
    ProfileNameWrite = 165,
    /// Read the default recipe for a beverage from the machine.
    RecipeQuantityRead = 166,
    /// Read the priority order of beverages from the machine.
    RecipePriorityRead = 168,
    ProfileSelection = 169,
    RecipeNameRead = 170,
    RecipeNameWrite = 171,
    SetFavoriteBeverages = 173,
    /// Request the min/max values for a given beverage. This may be a PIN operation in some other versions of the protocol.
    RecipeMinMaxSync = 176,
    PinSet = 177,
    BeanSystemSelect = 185,
    BeanSystemRead = 186,
    BeanSystemWrite = 187,
    PinRead = 210,
    SetTime = 226,
}}

hardware_enum! {"The temperature of the dispensed beverage.", EcamTemperature {
    Low = 0,
    Mid = 1,
    High = 2,
    VeryHigh = 3,
}}

hardware_enum! {"The strength of the dispensed beverage.", EcamBeverageTaste {
    Preground = 0,
    ExtraMild = 1,
    Mild = 2,
    Normal = 3,
    Strong = 4,
    ExtraStrong = 5,
}}

hardware_enum! {"The current state of the machine.", EcamMachineState {
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
}}

hardware_enum! {"The accessory that is connected to the accessory port.", EcamAccessory {
    None = 0,
    Water = 1,
    Milk = 2,
    Chocolate = 3,
    MilkClean = 4,
}}

hardware_enum! {"The type of beverage to prepare.", EcamBeverageId {
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
}}

hardware_enum! {"The set of alarms the machine can produce.", EcamMachineAlarm {
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
}}

hardware_enum! {"The various switches that the machine reads.", EcamMachineSwitch {
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
}}
