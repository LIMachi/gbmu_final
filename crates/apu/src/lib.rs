use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;
use std::time::Duration;
use anyhow::Context;
use rodio::{cpal, Device, OutputStream, OutputStreamHandle, Sink, Source, SupportedStreamConfig};
use cpal::traits::{HostTrait, DeviceTrait};
use rodio::cpal::SampleRate;
use rtrb::{Consumer, Producer};
use shared::mem::IOBus;
use serde::{Deserialize, Serialize};
use shared::utils::Cell;
use crate::cpal::{FromSample, SampleFormat};

mod dsg;

fn default_device() -> String {
    cpal::default_host().default_output_device().unwrap().name().unwrap_or(Default::default())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    #[serde(default = "default_device")]
    dev_name: String
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            dev_name: default_device()
        }
    }
}

struct Audio {
    sample_rate: SampleRate,
    dev_name: String,
    device: Device,
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Sink
}

impl Audio {
    fn devices() -> impl Iterator<Item = Device> {
        cpal::default_host().output_devices().unwrap()
    }

    pub fn sample_rate(&self) -> u32 { self.sample_rate.0 }

    pub fn switch<S: Into<String>>(&mut self, name: S) -> anyhow::Result<&mut Self> {
        let name = name.into();
        if name != self.dev_name {
            let (dev, dev_name) = Self::devices()
                .filter_map(|x| x.name().ok().map(|n| (x, n)))
                .find(|(_, n)| n == &name).context("no such device")?;
            let config = dev.default_output_config().unwrap();
            let (stream, handle) = OutputStream::try_from_device_config(
                &dev, SupportedStreamConfig::new(
                    2,
                    config.sample_rate(),
                    config.buffer_size().clone(),
                    SampleFormat::F32))?;
            let sink = Sink::try_new(&handle)?;
            self.sample_rate = config.sample_rate();
            self.device = dev;
            self.dev_name = dev_name;
            self.stream = Some(stream);
            self.handle = Some(handle);
            self.sink = sink;
        }
        Ok(self)
    }

    pub(crate) fn has_device(&self) -> bool {
        self.handle.is_some()
    }

    pub fn buffer(&self) -> (Producer<f32>, Consumer<f32>) {
        rtrb::RingBuffer::new(self.sample_rate() as usize)
    }

    pub(crate) fn bind<S: Source + 'static + Send>(&self, source: S) where
        <S as Iterator>::Item: rodio::Sample + Send,
        f32: FromSample<<S as Iterator>::Item>, {
        self.sink.append(source);
    }

    pub(crate) fn new(config: &SoundConfig) -> Self {
        let mut audio = Self {
            device: cpal::default_host().default_output_device().unwrap(),
            sink: Sink::new_idle().0,
            dev_name: Default::default(),
            sample_rate: SampleRate(44100),
            handle: None,
            stream: None
        };
        if let Err(e) = audio.switch(&config.dev_name) {
            println!("error switching to config device {}: {e:?}", config.dev_name);
        }
        audio
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Sample { Left, Right }

impl Default for Sample {
    fn default() -> Self { Sample::Left }
}

impl Not for Sample {
    type Output = Self;
    fn not(self) -> Self::Output { match self { Sample::Right => Sample::Left, Sample::Left => Sample::Right } }
}

struct Output(u32, rtrb::Consumer<f32>);

#[derive(Clone)]
struct Input(Rc<RefCell<rtrb::Producer<f32>>>);

impl Default for Input {
    fn default() -> Self {
        Self(rtrb::RingBuffer::new(0).0.cell())
    }
}

impl Input {
    pub fn write_sample(&mut self, value: [f32; 2]) {
        self.0.as_ref().borrow_mut().push(value[0]).ok();
        self.0.as_ref().borrow_mut().push(value[1]).ok();
    }
}

impl Output {
    pub fn read_sample(&mut self) -> f32 {
        self.1.pop().unwrap_or(0.)
    }

}

#[derive(Default)]
pub struct Apu {
    sample: f64,
    sample_rate: usize,
    tick: f64,
    input: Input,
    dsg: dsg::DSG
}

#[derive(Clone)]
pub struct Controller {
    input: Input,
    driver: Rc<RefCell<Audio>>
}

impl Controller {
    pub fn devices() -> impl Iterator<Item = String> {
        Audio::devices().filter_map(|x| x.name().ok())
    }

    pub fn switch<S: Into<String>>(&mut self, name: S) {
        self.driver.as_ref().borrow_mut().switch(name).map(|x| {
            let (prod, cons) = x.buffer();
            self.input.0.replace(prod);
            x.bind(Output(x.sample_rate(), cons));
        }).ok();
    }

    pub fn new(config: &SoundConfig) -> Self {
        let audio = Audio::new(config);
        let (producer, consumer) = audio.buffer();
        audio.bind(Output(audio.sample_rate(), consumer));
        Self { input: Input(producer.cell()), driver: audio.cell() }
    }

    pub fn apu(&self) -> Apu {
        Apu::new(self.driver.as_ref().borrow().sample_rate(), self.input.clone())
    }
}

impl Source for Output {
    fn current_frame_len(&self) -> Option<usize> { None }

    fn channels(&self) -> u16 { 2 }

    fn sample_rate(&self) -> u32 { self.0 }

    fn total_duration(&self) -> Option<Duration> { None }
}

impl Iterator for Output {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_sample())
    }
}

impl Apu {
    fn new(sample_rate: u32, input: Input) -> Self {
        Self { sample: 0., sample_rate: sample_rate as usize, tick: 4_194_304. / sample_rate as f64, input, dsg: dsg::DSG::new() }
    }

    pub fn tick(&mut self) {
        self.sample += 1.;
        if self.sample >= self.tick {
            self.input.write_sample(self.dsg.tick(self.sample_rate as u32));
            self.sample -= self.tick;
        }
    }
}

impl shared::mem::Device for Apu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.dsg.configure(bus);
    }
}
