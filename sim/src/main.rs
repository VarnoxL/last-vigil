use std::thread::sleep;
use std::time::Duration;

const TICKS_PER_SEC: u64 = 10;

fn main() {
    // Optional first arg: stop after N ticks (handy for tests); otherwise run forever.
    let limit: Option<u64> = std::env::args().nth(1).and_then(|s| s.parse().ok());
    let tick_len = Duration::from_millis(1000 / TICKS_PER_SEC);

    let mut tick: u64 = 0;
    while limit.map_or(true, |n| tick < n) {
        tick += 1;
        println!("tick {tick}");
        sleep(tick_len); // fixed sleep, not a real fixed-timestep loop; Phase 2 replaces it
    }
}
