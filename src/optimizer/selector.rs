use crate::optimizer::population::{Individual, Select};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::{index, SliceRandom};

pub struct Selector {
    max_actions: usize,
    target_population: usize,
    rng: SmallRng,
}

impl Selector {
    pub fn new(max_actions: usize, target_population: usize, rng: SmallRng) -> Self {
        Selector {
            max_actions,
            target_population,
            rng,
        }
    }
}

impl<A, I> Select<A, I> for Selector
where
    I: Individual<Score = u32>,
{
    fn select_actions(&mut self, actions: &mut Vec<A>) {
        if actions.len() > self.max_actions {
            log::debug!(
                "Select {} out of {} actions",
                self.max_actions,
                actions.len()
            );
            actions.shuffle(&mut self.rng);
            actions.truncate(self.max_actions);
        }
    }

    fn select_population(&mut self, population: &mut Vec<I>) {
        if population.len() > self.target_population {
            log::info!(
                "Select around {} out of {} individuals",
                self.target_population,
                population.len()
            );

            // Sort population
            let indexes = index::sample_weighted(
                &mut self.rng,
                population.len(),
                |i| population[i].score(),
                self.target_population,
            )
            .unwrap()
            .into_vec();

            let mut i = 0;
            population.retain(|_| {
                let retain = indexes.contains(&i);
                i += 1;
                retain
            });
        }
    }

    fn best(&self, population: &[I]) -> usize {
        population
            .iter()
            .position_max_by_key(|ind| ind.score())
            .unwrap()
    }
}
