use std::time::{Duration, SystemTime};
use std::thread::sleep;


// GLobal constants
const S_PER_FRAME     :f64 = 1.0/60.0;


pub struct Clock {
    t_start_frame   :SystemTime,
    dur_frame       :Duration,

    dur_s_per_frame :Duration,
    time_start      :SystemTime,
    last_s          :u64,
    fps             :u16,
    last_fps_value  :u16
}

impl Clock {
    pub fn new() -> Clock {
        return Clock {
            // Expected duration of frames in seconds
            dur_s_per_frame: Duration::from_secs_f64(S_PER_FRAME),
            // Start time of the frame
            t_start_frame: SystemTime::now(),
            // Frame duration
            dur_frame: Duration::from_secs(0),

            // FPS
            // Start time of the frame
            time_start: SystemTime::now(),
            // Last second
            last_s: 0,
            // Value incremented in each tick
            fps: 0,
             // Value which will be set to fps at the end of every frame
            last_fps_value: 0,
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

    pub fn get_fps(&mut self) -> u16 {
        let actual_s = self.time_start.elapsed().unwrap().as_secs(); // Actual second
        self.fps += 1;

        if actual_s != self.last_s {
            self.last_s = actual_s;
            self.last_fps_value = self.fps;
            self.fps = 0;
        }

        return self.last_fps_value;
    }
}
