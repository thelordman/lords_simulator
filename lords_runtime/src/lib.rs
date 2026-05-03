use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Sender};
use std::time::Instant;
use lords_sim::Simulation;

pub const TICKS_PER_SECOND: u64 = 1;

pub struct SimRunner {
    pub state: Arc<RwLock<lords_sim::State>>,
    cmd_tx: Sender<Command>,
}

impl SimRunner {
    pub fn new(mut sim: Simulation) -> Self {
        let state = Arc::new(RwLock::new(sim.state.clone()));
        let state_clone = Arc::clone(&state);
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let tick_interval = std::time::Duration::from_secs(TICKS_PER_SECOND);
        let mut next_tick = Instant::now() + tick_interval;

        std::thread::spawn(move || {
            let mut paused = false;
            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Command::Pause  => paused = true,
                        Command::Resume => paused = false,
                        Command::Reset  => { sim.state.time.zero(); paused = false; }
                    }
                }

                if !paused {
                    sim.tick();
                    *state_clone.write().unwrap() = sim.state.clone();
                }

                let now = Instant::now();
                if next_tick > now {
                    std::thread::sleep(next_tick - now);
                }

                next_tick += tick_interval;
            }
        });

        Self { state, cmd_tx }
    }

    pub fn pause(&self)  {
        let _ = self.cmd_tx.send(Command::Pause);
    }
    pub fn resume(&self) {
        let _ = self.cmd_tx.send(Command::Resume);
    }
    pub fn reset(&self)  {
        let _ = self.cmd_tx.send(Command::Reset);
    }
}

pub enum Command {
    Pause,
    Resume,
    Reset,
}
