use rodio::{cpal, Source, Device, OutputStream, OutputStreamHandle, Sink, SupportedStreamConfig};
use cpal::{
    traits::{HostTrait, DeviceTrait},
    SampleRate,
    FromSample,
    SampleFormat
};
use rtrb::{Consumer, Producer, RingBuffer};
use anyhow::Context;

use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use shared::utils::Cell;

#[derive(Clone)]
pub(crate) struct Input(pub(crate) Rc<RefCell<Producer<f32>>>);

impl Default for Input {
    fn default() -> Self {
        Self(RingBuffer::new(0).0.cell())
    }
}

impl Input {
    pub fn write_sample(&mut self, samples: [f32; 2]) {
        self.0.as_ref().borrow_mut().push(left).ok();
        self.0.as_ref().borrow_mut().push(right).ok();
    }

    pub fn replace(&mut self, other: Input) {
        self.0.replace(other.0.into_inner());
    }
}

pub(crate)struct Output(pub(crate) u32, pub(crate) Consumer<f32>);

impl Output {
    pub fn read_sample(&mut self) -> f32 {
        self.1.pop().unwrap_or(0.)
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

pub(crate) struct Audio {
    sample_rate: SampleRate,
    dev_name: String,
    device: Device,
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Sink
}

fn default_device() -> String {
    cpal::default_host().default_output_device().unwrap().name().unwrap_or(Default::default())
}

impl Audio {
    pub fn devices() -> impl Iterator<Item = Device> {
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

    pub(crate) fn bind(&self) -> Input {
        let (producer, consumer) = RingBuffer::new(self.sample_rate() as usize);
        self.sink.append(Output(self.sample_rate(), consumer));
        Input(producer.cell())
    }

    pub(crate) fn new(config: &super::SoundConfig) -> Self {
        let mut audio = Self {
            device: cpal::default_host().default_output_device().unwrap(),
            sink: Sink::new_idle().0,
            dev_name: Default::default(),
            sample_rate: SampleRate(44100),
            handle: None,
            stream: None
        };
        if let Err(e) = audio.switch(&config.dev_name) {
            log::error!("failed to switch to audio device {}: {e:?}", &config.dev_name);
            audio.switch(default_device()).ok();
        }
        audio
    }
}
