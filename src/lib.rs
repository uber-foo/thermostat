#![doc(html_root_url = "https://docs.rs/thermostat")]
#![doc(issue_tracker_base_url = "https://github.com/uber-foo/thermostat/issues/")]
#![deny(
    missing_docs, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code,
    unstable_features, unused_import_braces, unused_qualifications, unused_variables,
    unreachable_code, unused_comparisons, unused_imports, unused_must_use
)]
#![no_std]

//! This crate provides a finite state machine for a thermostat controlling a centralized HVAC
//! system or other heating and/or cooling apparatus.
//!
//! The goal of this component is to provide an abstracted thermostat that can be embedded in any
//! device where temperature and/or humidity must be controlled (e.g., homes, offices,
//! refigerators, kegerators). The library is starting out with a simple hysteretic control
//! algorithm using temperature and humidity measurements. Progressing from there, this library
//! will look at various stratgies to continually optimize in-situ for objectievs such as power
//! conservation, system lifespan, or predicted demand.
//!
//! This crate is not currently suitable for use with multi-stage or other controlled variable load
//! applications. It was designed on a model of simple on-or-off heating and cooling devices found
//! with in most HVAC systems and refigeration compressors.
//!
//! The thermostat uses double-precision floating-point format for representing both temperature in
//! degrees Celsius and percent relative humidity.
//!
//! # Usage Example
//!
//! ```
//! extern crate thermostat;
//!
//! use thermostat::{OperatingMode, Thermostat, Error as ThermostatError, ThermostatInterface};
//!
//! struct MyThermostatInterface {}
//! impl ThermostatInterface for MyThermostatInterface {
//!     fn call_for_heat(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//!     fn stop_call_for_heat(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//!
//!     fn call_for_cool(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//!     fn stop_call_for_cool(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//!
//!     fn call_for_fan(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//!     fn stop_call_for_fan(&self) -> Result<(), ThermostatError> {
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     // create a new physical interface for the thermostat
//!     let interface = MyThermostatInterface {};
//!     // create a new thermostat with our physical interface
//!     let mut thermostat = Thermostat::new(&interface);
//!
//!     // once the thermostat has been provided with a measure routine
//!     // it will begin polling for new measurements and calling for
//!     // heat, cool, and/or fan -- depending on which methods have
//!     // been registered.
//!
//!     // set max temp thermostat will allow before calling for cool
//!     thermostat.set_maximum_set_temperature(22.5).unwrap();
//!     // set min temp thermostat will allow before calling for heat
//!     thermostat.set_minimum_set_temperature(18.0).unwrap();
//!     // maintain temperatures between min and max set points
//!     thermostat.set_operating_mode(OperatingMode::MaintainRange).unwrap();
//! }
//! ```

use core::fmt;
use core::result::Result;

/// Thermostat errors
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Indicates a handler failed, intended to be used by thermostat handler implementations
    HandlerFailed,
    /// Indicates a measurement failed, indended to be used by thermostat measurement implementations
    MeasurementFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Thermostat Error: ")?;
        let label = match *self {
            Error::HandlerFailed => "handler failed",
            Error::MeasurementFailed => "measurement failed",
        };
        f.write_str(&label)
    }
}

// Safe temperatures control absolute limits that the thermostat logic will allow in any operating
// mode. No set temperature may exceed these bounds nor will normal operating mode constraints on
// the usage of the heating or cooling system be respected. The only way to override this behavior
// is to set the operating mode to DisabledUnsafe.
const DEFAULT_MAXIMUM_SAFE_TEMPERATURE: f64 = 30.0;
const DEFAULT_MINIMUM_SAFE_TEMPERATURE: f64 = 15.0; // degrees C
const DEFAULT_CURRENT_TEMPERATURE: f64 =
    (DEFAULT_MAXIMUM_SAFE_TEMPERATURE - DEFAULT_MINIMUM_SAFE_TEMPERATURE) / 2.0; // degrees C

const DEFAULT_OPERATING_MODE: OperatingMode = OperatingMode::Disabled;

/// Various thermostat operating modes
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum OperatingMode {
    /// Maintain temperature between min and max set points
    MaintainRange,
    /// Maintain temperature below the max set point
    CoolToSetPoint,
    /// Maintain temperature above the min set point
    HeatToSetPoint,
    /// Maintain only within the min and max safety set points
    Disabled,
    /// Ignore safety set points -- do nothing except measure
    DisabledUnsafe,
}

impl fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            OperatingMode::MaintainRange => "Maintain Range",
            OperatingMode::CoolToSetPoint => "Cool to Set Point",
            OperatingMode::HeatToSetPoint => "Heat to Set Point",
            OperatingMode::Disabled => "Disabled",
            OperatingMode::DisabledUnsafe => "Disabled (Unsafe)",
        })
    }
}

/// Thermostat state machine
#[derive(Copy, Clone)]
pub struct Thermostat<'a> {
    operating_mode: OperatingMode,
    minimum_safe_temperature: f64,
    maximum_safe_temperature: f64,
    minimum_set_temperature: f64,
    maximum_set_temperature: f64,
    current_temperature: f64,
    interface: &'a ThermostatInterface,
}

/// Wrapper for physical interface controls
pub trait ThermostatInterface {
    /// calls for heat
    fn call_for_heat(&self) -> Result<(), Error>;
    /// stops call for heat
    fn stop_call_for_heat(&self) -> Result<(), Error>;
    /// calls for cool
    fn call_for_cool(&self) -> Result<(), Error>;
    /// stops call for cool
    fn stop_call_for_cool(&self) -> Result<(), Error>;
    /// calls for fan
    fn call_for_fan(&self) -> Result<(), Error>;
    /// stops call for fan
    fn stop_call_for_fan(&self) -> Result<(), Error>;
}

impl<'a> Thermostat<'a> {
    /// Create a new thermostat using the provided interface
    pub fn new(interface: &ThermostatInterface) -> Thermostat {
        Thermostat {
            operating_mode: DEFAULT_OPERATING_MODE,
            minimum_safe_temperature: DEFAULT_MINIMUM_SAFE_TEMPERATURE,
            maximum_safe_temperature: DEFAULT_MAXIMUM_SAFE_TEMPERATURE,
            minimum_set_temperature: DEFAULT_MINIMUM_SAFE_TEMPERATURE,
            maximum_set_temperature: DEFAULT_MAXIMUM_SAFE_TEMPERATURE,
            current_temperature: DEFAULT_CURRENT_TEMPERATURE,
            interface,
        }
    }

    /// Change the current operating mode.
    ///
    /// Will return an Err result if the specified operating mode is incompatible with the current
    /// configuration.
    pub fn set_operating_mode(&mut self, operating_mode: OperatingMode) -> Result<(), Error> {
        self.operating_mode = operating_mode;
        Ok(())
    }
    /// Get the current operating mode.
    pub fn get_operating_mode(&self) -> OperatingMode {
        self.operating_mode
    }

    /// Change the minimum safe temperature.
    ///
    /// If the maximum set temperature is higher than the specified maximum safe temperature, the
    /// maximum set temperature will be automatically adjusted to match.
    ///
    /// An Err Result is returned if the specified temperature is not within the bounds of the
    /// minimum and maximum safe temperatures.
    pub fn set_maximum_safe_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        self.maximum_safe_temperature = temperature;
        Ok(())
    }
    /// Get the current maximum safe temperature.
    pub fn get_maximum_safe_temperature(&self) -> f64 {
        self.maximum_safe_temperature
    }

    /// Change the minimum safe temperature.
    ///
    /// If the minimum set temperature is lower than the specified minimum safe temperature, the
    /// minimum set temperature will be automatically adjusted to match.
    ///
    /// An Err Result is returned if the specified temperature is not within the bounds of the
    /// minimum and maximum safe temperatures.
    pub fn set_minimum_safe_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        self.minimum_safe_temperature = temperature;
        Ok(())
    }
    /// Get the current minimum safe temperature
    pub fn get_minimum_safe_temperature(&self) -> f64 {
        self.minimum_safe_temperature
    }

    /// Change the maximum set temperature.
    ///
    /// If the minimum set temperature is higher than the specified maximum set temperature, the
    /// minimum set temperature will be automatically adjusted to match.
    ///
    /// An Err Result is returned if the specified temperature is not within the bounds of the
    /// minimum and maximum safe temperatures.
    pub fn set_maximum_set_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        self.maximum_set_temperature = temperature;
        Ok(())
    }
    /// Get the current maximum set temperature.
    pub fn get_maximum_set_temperature(&self) -> f64 {
        self.maximum_set_temperature
    }

    /// Change the minimum set temperature.
    ///
    /// If the minimum set temperature is higher than the specified maximum set temperature, the
    /// maximum set temperature will be automatically adjusted to match.
    ///
    /// An Err Result is returned if the specified temperature is not within the bounds of the
    /// minimum and maximum safe temperatures.
    pub fn set_minimum_set_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        self.minimum_set_temperature = temperature;
        Ok(())
    }
    /// Get the current minimum set temperature.
    pub fn get_minimum_set_temperature(&self) -> f64 {
        self.minimum_set_temperature
    }

    /// Get the current temperature as known to the thermostat
    pub fn get_current_temperature(&self) -> f64 {
        self.current_temperature
    }

    /// Update the thermostat with a new temperature reading
    pub fn set_current_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        if (temperature < self.minimum_safe_temperature
            && self.operating_mode != OperatingMode::DisabledUnsafe)
            || (temperature < self.minimum_set_temperature
                && self.operating_mode != OperatingMode::CoolToSetPoint)
        {
            self.interface.call_for_fan()?;
            self.interface.call_for_heat()?;
        } else if (temperature > self.maximum_safe_temperature
            && self.operating_mode != OperatingMode::DisabledUnsafe)
            || (temperature > self.maximum_set_temperature
                && self.operating_mode != OperatingMode::HeatToSetPoint)
        {
            self.interface.call_for_fan()?;
            self.interface.call_for_cool()?;
        } else {
            self.interface.stop_call_for_heat()?;
            self.interface.stop_call_for_cool()?;
            self.interface.stop_call_for_fan()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysWorksInterface {}

    impl ThermostatInterface for AlwaysWorksInterface {
        fn call_for_heat(&self) -> Result<(), Error> {
            Ok(())
        }
        fn call_for_cool(&self) -> Result<(), Error> {
            Ok(())
        }
        fn call_for_fan(&self) -> Result<(), Error> {
            Ok(())
        }
        fn stop_call_for_heat(&self) -> Result<(), Error> {
            Ok(())
        }
        fn stop_call_for_cool(&self) -> Result<(), Error> {
            Ok(())
        }
        fn stop_call_for_fan(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    const ALWAYS_WORKS: AlwaysWorksInterface = AlwaysWorksInterface {};

    #[test]
    fn thermo_default_uses_default_values() {
        let thermostat = Thermostat::new(&ALWAYS_WORKS);
        assert_eq!(thermostat.get_operating_mode(), DEFAULT_OPERATING_MODE);
        assert_eq!(
            thermostat.get_maximum_safe_temperature(),
            DEFAULT_MAXIMUM_SAFE_TEMPERATURE
        );
        assert_eq!(
            thermostat.get_minimum_safe_temperature(),
            DEFAULT_MINIMUM_SAFE_TEMPERATURE
        );
        assert_eq!(
            thermostat.get_maximum_set_temperature(),
            DEFAULT_MAXIMUM_SAFE_TEMPERATURE
        );
        assert_eq!(
            thermostat.get_minimum_set_temperature(),
            DEFAULT_MINIMUM_SAFE_TEMPERATURE
        );
    }

    #[test]
    fn thermo_changes_operating_mode() {
        let mut thermostat = Thermostat::new(&ALWAYS_WORKS);
        thermostat
            .set_operating_mode(OperatingMode::MaintainRange)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::MaintainRange
        );
        thermostat
            .set_operating_mode(OperatingMode::CoolToSetPoint)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::CoolToSetPoint
        );
        thermostat
            .set_operating_mode(OperatingMode::HeatToSetPoint)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::HeatToSetPoint
        );
        thermostat
            .set_operating_mode(OperatingMode::Disabled)
            .unwrap();
        assert_eq!(thermostat.get_operating_mode(), OperatingMode::Disabled);
        thermostat
            .set_operating_mode(OperatingMode::DisabledUnsafe)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::DisabledUnsafe
        );
    }

    #[test]
    fn thermo_changes_maximum_safe_temperature() {
        let mut thermostat = Thermostat::new(&ALWAYS_WORKS);
        thermostat.set_maximum_safe_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), 5.0);
        thermostat.set_maximum_safe_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), 15.0);
        thermostat.set_maximum_safe_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -15.0);
        thermostat.set_maximum_safe_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -5.0);
        thermostat.set_maximum_safe_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_minimum_safe_temperature() {
        let mut thermostat = Thermostat::new(&ALWAYS_WORKS);
        thermostat.set_minimum_safe_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), 5.0);
        thermostat.set_minimum_safe_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), 15.0);
        thermostat.set_minimum_safe_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -15.0);
        thermostat.set_minimum_safe_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -5.0);
        thermostat.set_minimum_safe_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_maximum_set_temperature() {
        let mut thermostat = Thermostat::new(&ALWAYS_WORKS);
        thermostat.set_maximum_set_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), 5.0);
        thermostat.set_maximum_set_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), 15.0);
        thermostat.set_maximum_set_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -15.0);
        thermostat.set_maximum_set_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -5.0);
        thermostat.set_maximum_set_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_minimum_set_temperature() {
        let mut thermostat = Thermostat::new(&ALWAYS_WORKS);
        thermostat.set_minimum_set_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), 5.0);
        thermostat.set_minimum_set_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), 15.0);
        thermostat.set_minimum_set_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -15.0);
        thermostat.set_minimum_set_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -5.0);
        thermostat.set_minimum_set_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -0.0);
    }
}
