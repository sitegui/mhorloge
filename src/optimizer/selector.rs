use crate::optimizer::population::{Individual, Select};
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;

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
    I: Individual,
    I::Score: Ord,
{
    fn select_actions(&mut self, actions: &mut Vec<A>) {
        if actions.len() > self.max_actions {
            actions.shuffle(&mut self.rng);
            actions.truncate(self.max_actions);
        }
    }

    fn select_population(&mut self, population: &mut Vec<I>) {
        if population.len() > self.target_population {
            // Sort population
            population.sort_by_key(|individual| individual.score());

            // Select individuals: `p(i)` is the probability of the `i`-th worst individual to
            // survive. Given that:
            // - `p(0) = 0`: the worst individual will never be selected
            // - `sum(p(i)) = target`: the expected final population size is the target
            // - `p(i) = a + b * i`: the probability falls linearly
            // Then the solution is uniquely defined: `p(i) = 2 * target * i / (n * (n-1))`
            let n = population.len() as u32;
            let target = self.target_population as u32;
            let mut i = 0;
            population.retain(|_| {
                let retain = self.rng.gen_ratio(2 * target * i, n * (n - 1));
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
