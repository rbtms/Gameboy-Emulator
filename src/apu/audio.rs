extern crate sdl2;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

const FREQ :i32 = 48_000;

pub struct Audio {
    device: AudioQueue<i8>
}


impl Audio {
    pub fn new(subsystem: AudioSubsystem) -> Audio {
        let desired_spec = AudioSpecDesired {
            freq: Some(FREQ),
            channels: Some(2),
            samples: None
        };

        let device = subsystem.open_queue::<i8, _>(None, &desired_spec).unwrap();

        return Audio {
            device
        }
    }

    pub fn queue(&mut self, val_left: u8, val_right: u8, n_samples: u16) {
        if self.device.size() > 10000 {
            return;
        }

        //let max_queue_size = 69905*2; // 2 frames
        //let n_bytes = (max_queue_size as u32-self.device.size()).min(n_samples as u32);
        let n_bytes = n_samples;

        if n_bytes > 0 {
            //println!("size: {} n_bytes: {} left {} right {}", self.device.size(), n_bytes, val_left, val_right);

            let mut data = vec![0;n_bytes as usize];

            for i in 0..n_bytes {
                data[i as usize] = if i&2 == 0 { val_left } else { val_right } as i8;
            }

            self.device.queue_audio(&data).unwrap();
        }
    }

    pub fn pause(&mut self) {
        self.device.pause();
    }

    pub fn resume(&mut self) {
        self.device.clear();
        self.device.resume();
    }
}
