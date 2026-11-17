use flappy_core::{Config, Input, Phase, World};
use proptest::prelude::*;

fn inputs() -> impl Strategy<Value = Vec<Input>> {
    prop::collection::vec(prop_oneof![Just(Input::Idle), Just(Input::Flap)], 0..600)
}

fn run(seed: u64, inputs: &[Input]) -> Vec<World> {
    inputs
        .iter()
        .scan(World::new(Config::default(), seed), |world, input| {
            *world = world.step(*input);
            Some(world.clone())
        })
        .collect()
}

proptest! {
    #[test]
    fn the_same_seed_and_inputs_always_give_the_same_game(seed: u64, inputs in inputs()) {
        prop_assert_eq!(run(seed, &inputs), run(seed, &inputs));
    }

    #[test]
    fn the_bird_stays_between_the_top_of_the_screen_and_the_ground(seed: u64, inputs in inputs()) {
        let config = Config::default();
        for world in run(seed, &inputs) {
            let bird = world.bird();
            prop_assert!(bird.y - config.bird_radius >= 0.0);
            prop_assert!(bird.y + config.bird_radius <= config.ground_y);
        }
    }

    #[test]
    fn a_finished_game_stays_finished(seed: u64, inputs in inputs()) {
        let phases: Vec<Phase> = run(seed, &inputs).iter().map(World::phase).collect();
        let first_over = phases.iter().position(|phase| *phase == Phase::Over);
        if let Some(first_over) = first_over {
            prop_assert!(phases.iter().skip(first_over).all(|phase| *phase == Phase::Over));
        }
    }

    #[test]
    fn the_score_rises_by_at_most_one_per_tick(seed: u64, inputs in inputs()) {
        let scores: Vec<u32> = run(seed, &inputs).iter().map(World::score).collect();
        prop_assert!(scores.windows(2).all(|pair| pair[1] == pair[0] || pair[1] == pair[0] + 1));
    }

    #[test]
    fn listed_pipes_are_inside_the_view_and_in_order(seed: u64, inputs in inputs()) {
        let config = Config::default();
        for world in run(seed, &inputs) {
            let xs: Vec<f32> = world.pipes().map(|pipe| pipe.x).collect();
            prop_assert!(xs.iter().all(|x| *x + config.course.pipe_width >= 0.0 && *x <= config.view_width));
            prop_assert!(xs.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
}
