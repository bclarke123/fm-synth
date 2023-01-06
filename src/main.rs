use std::sync::Mutex;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, Stream, StreamConfig};

use dasp::signal::{self, Signal};

static mut CARRIER: Mutex<Option<Box<dyn Signal<Frame = f64>>>> = Mutex::new(None);
static mut MODULATOR: Mutex<Option<Box<dyn Signal<Frame = f64>>>> = Mutex::new(None);

use fm_synth::octave;

struct Note {
    freq: Option<f64>,
    duration: u64
}

impl Note {
    fn new(freq: Option<f64>, duration: u64) -> Self {
        Self {
            freq, duration
        }
    }
    fn some(freq: octave::Note, octave: i32, duration: u64) -> Self {
        Self::new(Some(octave::freq(freq, octave)), duration)
    }
    fn silence(duration: u64) -> Self {
        Self::new(None, duration)
    }
}

fn song() -> Vec<Note> {
    include_str!("song.txt")
        .lines()
        .map(|l| l.split(' ').collect::<Vec<_>>())
        .map(|x| match x[0] {
            "S" => Note::silence(x[1].parse::<u64>().unwrap()),
            "A" => Note::some(octave::A, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "B" => Note::some(octave::B, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "C" => Note::some(octave::C, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "Cs" => Note::some(octave::C_SHARP, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "D" => Note::some(octave::D, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "Ds" => Note::some(octave::D_SHARP, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "E" => Note::some(octave::E, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "F" => Note::some(octave::F, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "Fs" => Note::some(octave::F_SHARP, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "G" => Note::some(octave::G, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            "Af" => Note::some(octave::A_FLAT, x[1].parse::<i32>().unwrap(), x[2].parse::<u64>().unwrap()),
            _ => panic!()
        })
        .collect::<Vec<_>>()
}

fn set_harmonic(hz: f64, mul: f64) {
    set_carrier_hz(hz);
    set_modifier_hz(hz * mul);
}

fn set_carrier_hz(hz: f64) {
    let mut c_lock = unsafe { CARRIER.lock().unwrap() };
    *c_lock = Some(Box::new(signal::rate(44_100.0).const_hz(hz).sine()));
}

fn set_carrier_silent() {
    let mut c_lock = unsafe { CARRIER.lock().unwrap() };
    *c_lock = None;
}

fn set_modifier_hz(hz: f64) {
    let mut m_lock = unsafe { MODULATOR.lock().unwrap() };
    *m_lock = Some(Box::new(signal::rate(44_100.0).const_hz(hz).sine()));
}

fn set_modifier_silent() {
    let mut m_lock = unsafe { MODULATOR.lock().unwrap() };
    *m_lock = None;
}

fn run<T: Sample>(device: Device, config: StreamConfig) -> Stream {
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let data_callback = |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
        let mut c_lock = unsafe { CARRIER.lock().unwrap() };
        let mut m_lock = unsafe { MODULATOR.lock().unwrap() };

        if let (Some(carrier), Some(modifier)) = (c_lock.as_deref_mut(), m_lock.as_deref_mut()) {
            for sample in data.iter_mut() {
                let frame = (carrier.next() + modifier.next()) as f32;
                *sample = Sample::from(&frame);
            }
        } else {
            for sample in data.iter_mut() {
                *sample = Sample::from(&0.0);
            }
        }
    };

    device
        .build_output_stream(&config, data_callback, err_fn)
        .unwrap()
}

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Couldn't open output device");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    let stream = match sample_format {
        SampleFormat::F32 => run::<f32>(device, config),
        SampleFormat::I16 => run::<i16>(device, config),
        SampleFormat::U16 => run::<u16>(device, config),
    };

    stream.play().unwrap();

    let harmonic = 1.0 / 5.0;
    let song = song();

    for cur in song {

        if let Some(note) = cur.freq {
            set_harmonic(note, harmonic);
        } else {
            set_carrier_silent();
            set_modifier_silent();
        }

        std::thread::sleep(Duration::from_millis(cur.duration));
    }
}
