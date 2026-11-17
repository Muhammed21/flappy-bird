use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use flappy_bench::{autopilot, long_game};
use flappy_core::{Config, World};
use flappy_storage::{Best, decode, encode};

fn tick(c: &mut Criterion) {
    let world = (0..600).fold(World::new(Config::default(), 42), |world, _| {
        world.step(autopilot(&world))
    });
    c.bench_function("world: one tick mid-game", |b| {
        b.iter(|| black_box(&world).step(autopilot(black_box(&world))));
    });
}

fn replay(c: &mut Criterion) {
    let Ok(replay) = long_game(42, 36_000) else {
        return;
    };
    c.bench_function("replay: verify a 10-minute game", |b| {
        b.iter(|| black_box(&replay).verify());
    });
    let save = encode(&replay);
    c.bench_function("storage: decode a 10-minute save", |b| {
        b.iter(|| decode(black_box(&save)));
    });
    c.bench_function("storage: load and verify a 10-minute save", |b| {
        b.iter(|| Best::load(Some(black_box(&save))));
    });
}

criterion_group!(benches, tick, replay);
criterion_main!(benches);
