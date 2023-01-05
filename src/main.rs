use std::sync::Mutex;
use std::time::Duration;

use cpal::{SampleFormat, Sample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use dasp::signal;
use dasp::signal::{Signal,Sine,ConstHz};

static mut CARRIER: Mutex<Option<Box<Sine<ConstHz>>>> = Mutex::new(None);
static mut MODIFIER: Mutex<Option<Box<Sine<ConstHz>>>> = Mutex::new(None);

fn set_carrier_hz(hz: f64) {
    {
        let mut c_lock = unsafe { CARRIER.lock().unwrap() };
        *c_lock = Some(Box::new(signal::rate(44_100.0).const_hz(hz).sine()));
    }
}

fn set_modifier_hz(hz: f64) {
    {
        let mut m_lock = unsafe { MODIFIER.lock().unwrap() };
        *m_lock = Some(Box::new(signal::rate(44_100.0).const_hz(hz).sine()));
    }
}

fn main() {

    set_carrier_hz(440.0);
    set_modifier_hz(220.0);

    let host = cpal::default_host();
    let device = host.default_output_device().expect("Couldn't open output device");

    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    let data_callback = match sample_format {
        SampleFormat::I16 => |data: &mut [i16], _info: &cpal::OutputCallbackInfo| {

            let mut c_lock = unsafe { CARRIER.lock().unwrap() };
            let mut m_lock = unsafe { MODIFIER.lock().unwrap() };

            if let (Some(carrier), Some(modifier)) = (c_lock.as_deref_mut(), m_lock.as_deref_mut()) {
                for sample in data.iter_mut() {
                    let frame = (carrier.next() * modifier.next()) as f32;
                    *sample = frame.to_i16();
                }
            }

        },
        _ => panic!()
    };

    let stream = device.build_output_stream(&config, data_callback, err_fn).unwrap();
    stream.play().unwrap();

    loop {
        std::thread::sleep(Duration::from_millis(100));
    }
}
