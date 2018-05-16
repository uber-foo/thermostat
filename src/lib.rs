#![doc(html_root_url = "https://docs.rs/uberhome-thermostat")]
#![doc(issue_tracker_base_url = "https://github.com/uber-foo/uberhome-thermostat/issues/")]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications, unused_variables, unreachable_code, unused_comparisons, unused_imports,
    unused_must_use
)]

//! This crate provides a finite state machine for a thermostat controlling a centralized HVAC
//! system or other heating and/or cooling apparatus. It is a component of the
//! [UberHome](https://labs.uberfoo.net/uberhome) home automation platform.
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
//! extern crate uberhome_thermostat;
//!
//! use uberhome_thermostat::{Measurement, OperatingMode, Thermostat};
//!
//! enum HvacStatus {
//!     HeatOn,
//!     CoolOn,
//!     Off,
//! }
//!
//! fn call_for_heat() -> Result<(), String> {
//!     println!("calling for heat...");
//!     Ok(())
//! }
//!
//! fn stop_call_for_heat() -> Result<(), String> {
//!     println!("stopping call for heat...");
//!     Ok(())
//! }
//!
//! fn call_for_cool() -> Result<(), String> {
//!     println!("calling for cool...");
//!     Ok(())
//! }
//!
//! fn stop_call_for_cool() -> Result<(), String> {
//!     println!("stopping call for cool...");
//!     Ok(())
//! }
//!
//! fn call_for_fan() -> Result<(), String> {
//!     println!("calling for fan...");
//!     Ok(())
//! }
//!
//! fn stop_call_for_fan() -> Result<(), String> {
//!     println!("stopping call for fan...");
//!     Ok(())
//! }
//!
//! fn measure_temp_and_humidity() -> Result<Measurement, String> {
//!     Ok(Measurement {
//!         temperature: 15.0,
//!         humidity: 40.0,
//!     })
//! }
//!
//! fn main() {
//!     // create a new thermostat with default settings
//!     let mut thermostat = Thermostat::default();
//!
//!     // register interfaces with device native implementation
//!     thermostat.use_heat(call_for_heat, stop_call_for_heat);
//!     thermostat.use_cool(call_for_cool, stop_call_for_cool);
//!     thermostat.use_fan(call_for_fan, stop_call_for_fan);
//!     thermostat.use_measure(measure_temp_and_humidity);
//!
//!     // once the thermostat has been provided with a measure routine
//!     // it will begin polling for new measurements and calling for
//!     // heat, cool, and/or fan -- depending on which methods have
//!     // been registered.
//!
//!     // set max temp thermostat will allow before calling for cool
//!     thermostat.change_maximum_set_temperature(22.5).unwrap();
//!     // set min temp thermostat will allow before calling for heat
//!     thermostat.change_minimum_set_temperature(18.0).unwrap();
//!     // maintain temperatures between min and max set points
//!     thermostat.change_operating_mode(OperatingMode::MaintainRange).unwrap();
//! }
//! ```

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

// Safe temperatures control absolute limits that the thermostat logic will allow in any operating
// mode. No set temperature may exceed these bounds nor will normal operating mode constraints on
// the usage of the heating or cooling system be respected. The only way to override this behavior
// is to set the operating mode to DisabledUnsafe.
const DEFAULT_MAXIMUM_SAFE_TEMPERATURE: f64 = 30.0;
const DEFAULT_MINIMUM_SAFE_TEMPERATURE: f64 = 10.0; // degrees C
const DEFAULT_CURRENT_TEMPERATURE: f64 =
    (DEFAULT_MAXIMUM_SAFE_TEMPERATURE - DEFAULT_MINIMUM_SAFE_TEMPERATURE) / 2.0; // degrees C

const DEFAULT_OPERATING_MODE: OperatingMode = OperatingMode::Disabled;

/// Various thermostat operating modes.
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

/// Temperature and humidity measurement.
#[derive(Copy, Clone, Debug)]
pub struct Measurement {
    /// current temperature in degrees Celsius
    pub temperature: f64,
    /// current percentage relative humidity
    pub humidity: f64,
}

/// Thermostat state machine.
#[derive(Copy, Clone, Debug)]
pub struct Thermostat {
    operating_mode: OperatingMode,
    minimum_safe_temperature: f64,
    maximum_safe_temperature: f64,
    minimum_set_temperature: f64,
    maximum_set_temperature: f64,
    current_temperature: f64,
    call_for_heat: fn() -> std::result::Result<(), String>,
    stop_call_for_heat: fn() -> std::result::Result<(), String>,
    heat_handlers_registered: bool,
    call_for_cool: fn() -> std::result::Result<(), String>,
    stop_call_for_cool: fn() -> std::result::Result<(), String>,
    cool_handlers_registered: bool,
    call_for_fan: fn() -> std::result::Result<(), String>,
    stop_call_for_fan: fn() -> std::result::Result<(), String>,
    fan_handlers_registered: bool,
    measure: fn() -> std::result::Result<Measurement, String>,
    measure_handler_registered: bool,
}

fn null_handler() -> std::result::Result<(), String> {
    let msg = "no call method registered";
    Err(msg.to_string())
}

fn null_measure_handler() -> std::result::Result<Measurement, String> {
    let msg = "no measurements available -- no measurement handler registered";
    Err(msg.to_string())
}

impl Default for Thermostat {
    fn default() -> Self {
        Thermostat {
            operating_mode: DEFAULT_OPERATING_MODE,
            minimum_safe_temperature: DEFAULT_MINIMUM_SAFE_TEMPERATURE,
            maximum_safe_temperature: DEFAULT_MAXIMUM_SAFE_TEMPERATURE,
            minimum_set_temperature: DEFAULT_MINIMUM_SAFE_TEMPERATURE,
            maximum_set_temperature: DEFAULT_MAXIMUM_SAFE_TEMPERATURE,
            current_temperature: DEFAULT_CURRENT_TEMPERATURE,
            call_for_heat: null_handler,
            stop_call_for_heat: null_handler,
            heat_handlers_registered: false,
            call_for_cool: null_handler,
            stop_call_for_cool: null_handler,
            cool_handlers_registered: false,
            call_for_fan: null_handler,
            stop_call_for_fan: null_handler,
            fan_handlers_registered: false,
            measure: null_measure_handler,
            measure_handler_registered: false,
        }
    }
}

impl Thermostat {
    /// Change the current operating mode.
    ///
    /// Will return an Err result if the specified operating mode is incompatible with the current
    /// configuration.
    pub fn change_operating_mode(&mut self, operating_mode: OperatingMode) -> Result<()> {
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
    pub fn change_maximum_safe_temperature(&mut self, temperature: f64) -> Result<()> {
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
    pub fn change_minimum_safe_temperature(&mut self, temperature: f64) -> Result<()> {
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
    pub fn change_maximum_set_temperature(&mut self, temperature: f64) -> Result<()> {
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
    pub fn change_minimum_set_temperature(&mut self, temperature: f64) -> Result<()> {
        self.minimum_set_temperature = temperature;
        Ok(())
    }
    /// Get the current minimum set temperature.
    pub fn get_minimum_set_temperature(&self) -> f64 {
        self.minimum_set_temperature
    }

    /// Register handlers to call for heat and cancel a call for heat.
    pub fn use_heat(
        &mut self,
        call_for_heat: fn() -> std::result::Result<(), String>,
        stop_call_for_heat: fn() -> std::result::Result<(), String>,
    ) {
        self.call_for_heat = call_for_heat;
        self.stop_call_for_heat = stop_call_for_heat;
        self.heat_handlers_registered = true;
    }

    /// Register handlers to call for cool and cancel a call for cool.
    pub fn use_cool(
        &mut self,
        call_for_cool: fn() -> std::result::Result<(), String>,
        stop_call_for_cool: fn() -> std::result::Result<(), String>,
    ) {
        self.call_for_cool = call_for_cool;
        self.stop_call_for_cool = stop_call_for_cool;
        self.cool_handlers_registered = true;
    }

    /// Register handlers to call for fan and cancel a call for fan.
    pub fn use_fan(
        &mut self,
        call_for_fan: fn() -> std::result::Result<(), String>,
        stop_call_for_fan: fn() -> std::result::Result<(), String>,
    ) {
        self.call_for_fan = call_for_fan;
        self.stop_call_for_fan = stop_call_for_fan;
        self.fan_handlers_registered = true;
    }

    /// Register handler to obtain current measurements
    pub fn use_measure(&mut self, measure: fn() -> std::result::Result<Measurement, String>) {
        self.measure = measure;
        self.measure_handler_registered = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermo_default_uses_default_values() {
        let thermostat = Thermostat::default();
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
        let mut thermostat = Thermostat::default();
        thermostat
            .change_operating_mode(OperatingMode::MaintainRange)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::MaintainRange
        );
        thermostat
            .change_operating_mode(OperatingMode::CoolToSetPoint)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::CoolToSetPoint
        );
        thermostat
            .change_operating_mode(OperatingMode::HeatToSetPoint)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::HeatToSetPoint
        );
        thermostat
            .change_operating_mode(OperatingMode::Disabled)
            .unwrap();
        assert_eq!(thermostat.get_operating_mode(), OperatingMode::Disabled);
        thermostat
            .change_operating_mode(OperatingMode::DisabledUnsafe)
            .unwrap();
        assert_eq!(
            thermostat.get_operating_mode(),
            OperatingMode::DisabledUnsafe
        );
    }

    #[test]
    fn thermo_changes_maximum_safe_temperature() {
        let mut thermostat = Thermostat::default();
        thermostat.change_maximum_safe_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), 5.0);
        thermostat.change_maximum_safe_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), 15.0);
        thermostat.change_maximum_safe_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -15.0);
        thermostat.change_maximum_safe_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -5.0);
        thermostat.change_maximum_safe_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_maximum_safe_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_minimum_safe_temperature() {
        let mut thermostat = Thermostat::default();
        thermostat.change_minimum_safe_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), 5.0);
        thermostat.change_minimum_safe_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), 15.0);
        thermostat.change_minimum_safe_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -15.0);
        thermostat.change_minimum_safe_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -5.0);
        thermostat.change_minimum_safe_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_minimum_safe_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_maximum_set_temperature() {
        let mut thermostat = Thermostat::default();
        thermostat.change_maximum_set_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), 5.0);
        thermostat.change_maximum_set_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), 15.0);
        thermostat.change_maximum_set_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -15.0);
        thermostat.change_maximum_set_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -5.0);
        thermostat.change_maximum_set_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_maximum_set_temperature(), -0.0);
    }

    #[test]
    fn thermo_changes_minimum_set_temperature() {
        let mut thermostat = Thermostat::default();
        thermostat.change_minimum_set_temperature(5.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), 5.0);
        thermostat.change_minimum_set_temperature(15.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), 15.0);
        thermostat.change_minimum_set_temperature(-15.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -15.0);
        thermostat.change_minimum_set_temperature(-5.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -5.0);
        thermostat.change_minimum_set_temperature(0.0).unwrap();
        assert_eq!(thermostat.get_minimum_set_temperature(), -0.0);
    }
}
