use flappy_core::{Config, Input, Phase, World};
use proptest::prelude::*;

fn inputs() -> impl Strategy<Value = Vec<Input>> {
    prop::collection::vec(prop_oneof![Just(Input::Idle), Just(Input::Flap)], 0..600)
}

fn run(inputs: &[Input]) -> Vec<World> {
    inputs
        .iter()
        .scan(World::new(Config::default()), |world, input| {
            *world = world.step(*input);
            Some(world.clone())
        })
        .collect()
}

proptest! {
    #[test]
    fn the_same_inputs_always_give_the_same_game(inputs in inputs()) {
        prop_assert_eq!(run(&inputs), run(&inputs));
    }

    #[test]
    fn the_bird_stays_between_the_top_of_the_screen_and_the_ground(inputs in inputs()) {
        let config = Config::default();
        for world in run(&inputs) {
            let bird = world.bird();
            prop_assert!(bird.y - config.bird_radius >= 0.0);
            prop_assert!(bird.y + config.bird_radius <= config.ground_y);
        }
    }

    #[test]
    fn a_finished_game_stays_finished(inputs in inputs()) {
        let phases: Vec<Phase> = run(&inputs).iter().map(World::phase).collect();
        let first_over = phases.iter().position(|phase| *phase == Phase::Over);
        if let Some(first_over) = first_over {
            prop_assert!(phases.iter().skip(first_over).all(|phase| *phase == Phase::Over));
        }
    }
}
