use std::time::{Duration, SystemTime};
use std::thread::sleep;

use crate::screen::Screen;

// GLobal constants
const S_PER_FRAME     :f64 = 1.0/60.0;

pub struct Clock {
    t_start_frame   :SystemTime,
    dur_frame       :Duration,

    dur_s_per_frame :Duration,
    time_start      :SystemTime,
    last_s          :u64,
    fps             :u8
}

impl Clock {
    pub fn new() -> Clock {
        return Clock {
            dur_s_per_frame : Duration::from_secs_f64(S_PER_FRAME),
            t_start_frame : SystemTime::now(),
            dur_frame     : Duration::from_secs(0), 

            // fps
            time_start   : SystemTime::now(),
            last_s       : 0,
            fps          : 0,
        }
    }

    // Spend time until next tick to regulate ticks/s
    pub fn wait_next_frame(&mut self) {
        self.dur_frame = self.t_start_frame.elapsed().unwrap();

        // Wait until the frame is over
        if self.dur_frame < self.dur_s_per_frame {
            //println!("Frame OK {:?}", self.dur_frame);
            sleep(self.dur_s_per_frame - self.dur_frame);
        } else {
            // println!("Frame took too long: {:?}", self.dur_frame); TODO: reenable
        }

        self.t_start_frame = SystemTime::now();
    }

    pub fn update_fps(&mut self, screen :&mut Screen) {
        let actual_s = self.time_start.elapsed().unwrap().as_secs(); // Actual second
        self.fps += 1;

        if actual_s != self.last_s {
            self.last_s = actual_s;
            screen.set_title_fps(self.fps);
            self.fps = 0;
        }
    }
}

