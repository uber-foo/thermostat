extern crate thermostat;

use thermostat::{Error, Measurement, Thermostat};

fn call_for_heat() -> Result<(), Error> {
    println!("heat on!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn stop_call_for_heat() -> Result<(), Error> {
    println!("heat off!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn call_for_cool() -> Result<(), Error> {
    println!("cool on!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn stop_call_for_cool() -> Result<(), Error> {
    println!("cool off!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn call_for_fan() -> Result<(), Error> {
    println!("fan on!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn stop_call_for_fan() -> Result<(), Error> {
    println!("fan off!");
    Ok(())
    // or, if failure Err(Error::HandlerFailed)
}

fn measure() -> Result<Measurement, Error> {
    println!("taking measurements...");
    Ok(Measurement {
        temperature: 20.0,
        humidity: 35.0,
    })
    // or, if failure Err(Error::MeasurementFailed)
}

fn main() {
    let mut thermostat = Thermostat::default();
    thermostat.use_heat(call_for_heat, stop_call_for_heat);
    thermostat.use_cool(call_for_cool, stop_call_for_cool);
    thermostat.use_fan(call_for_fan, stop_call_for_fan);
    thermostat.use_measure(measure);
    loop {
        thermostat.tick();
        ::std::thread::sleep(::std::time::Duration::from_secs(1));
    }
}
