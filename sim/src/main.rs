mod agent;
mod rng;
mod rumor;
mod sim;
mod world;

use std::thread::sleep;
use std::time::{Duration, Instant};

use sim::{Preset, Sim, TICKS_PER_SEC};

fn main() {
    // Args: [ticks] [seed] [quick|standard|long].
    // With a tick count, run headless as fast as possible; without, run in real time forever.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let limit: Option<u64> = args.first().and_then(|s| s.parse().ok());
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let preset = args.get(2).and_then(|s| Preset::parse(s)).unwrap_or(Preset::Standard);

    let mut sim = Sim::new(seed, preset, 50);
    println!("seed {seed} | {preset:?} | fragments {:?}", sim.truth.fragments());

    match limit {
        Some(n) => {
            let start = Instant::now();
            for _ in 0..n {
                sim.tick();
            }
            let elapsed = start.elapsed();
            print!("{}", sim.render());
            println!("{}", sim.summary());
            println!("{n} ticks in {elapsed:.2?} ({:.4} ms/tick)", elapsed.as_secs_f64() * 1000.0 / n as f64);
        }
        None => {
            // Fixed timestep: sleep until the next tick's deadline, so slow ticks don't drift the clock.
            let tick_len = Duration::from_millis(1000 / TICKS_PER_SEC);
            let mut next = Instant::now();
            loop {
                sim.tick();
                if sim.tick % TICKS_PER_SEC == 0 {
                    // ANSI: cursor home + clear, so the frame redraws in place.
                    print!("\x1b[H\x1b[2J{}{}\n", sim.render(), sim.summary());
                }
                next += tick_len;
                if let Some(wait) = next.checked_duration_since(Instant::now()) {
                    sleep(wait);
                }
            }
        }
    }
}
