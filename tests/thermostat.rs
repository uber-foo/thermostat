// #[cfg(test)]
extern crate thermostat;

use std::time::SystemTime;
use thermostat::*;

struct AlwaysWorksInterface {
    heat: bool,
    cool: bool,
    fan: bool,
    start: SystemTime,
}

impl Default for AlwaysWorksInterface {
    fn default() -> Self {
        Self {
            heat: false,
            cool: false,
            fan: false,
            start: SystemTime::now(),
        }
    }
}

impl ThermostatInterface for AlwaysWorksInterface {
    fn calling_for_heat(&self) -> Result<bool, Error> {
        Ok(self.heat)
    }
    fn calling_for_cool(&self) -> Result<bool, Error> {
        Ok(self.cool)
    }
    fn calling_for_fan(&self) -> Result<bool, Error> {
        Ok(self.fan)
    }
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
    fn get_seconds(&self) -> Result<u64, Error> {
        Ok(self.start.elapsed().unwrap().as_secs())
    }
}

#[test]
fn thermo_default_uses_default_values() {
    let interface = AlwaysWorksInterface::default();
    let thermostat = Thermostat::new(&interface);
    assert_eq!(thermostat.get_operating_mode(), OperatingMode::Disabled);
    assert_eq!(thermostat.get_maximum_safe_temperature(), 30.0);
    assert_eq!(thermostat.get_minimum_safe_temperature(), 15.0);
    assert_eq!(thermostat.get_maximum_set_temperature(), 30.0);
    assert_eq!(thermostat.get_minimum_set_temperature(), 15.0);
}

#[test]
fn thermo_changes_operating_mode() {
    let interface = AlwaysWorksInterface::default();
    let mut thermostat = Thermostat::new(&interface);
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
    let interface = AlwaysWorksInterface::default();
    let mut thermostat = Thermostat::new(&interface);
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
    let interface = AlwaysWorksInterface::default();
    let mut thermostat = Thermostat::new(&interface);
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
    let interface = AlwaysWorksInterface::default();
    let mut thermostat = Thermostat::new(&interface);
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
    let interface = AlwaysWorksInterface::default();
    let mut thermostat = Thermostat::new(&interface);
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
