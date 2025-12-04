#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celcius,
    Kelvin,
    Fahrenheit,
    Other(&'static str),
}

impl TemperatureUnit {
    pub fn as_str(&self) -> &'static str {
        // TODO: improve
        match self {
            TemperatureUnit::Celcius => "C",
            TemperatureUnit::Kelvin => "k",
            TemperatureUnit::Fahrenheit => "F",
            TemperatureUnit::Other(other) => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumidityUnit {
    Percentage,
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryUnit {
    Percentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightUnit {
    Lux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureUnit {
    HectoPascal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyUnit {
    KiloWattHour,
}
