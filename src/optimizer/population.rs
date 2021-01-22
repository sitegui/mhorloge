use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::rngs::SmallRng;
use rand::seq::index;

#[derive(Debug, Clone)]
pub struct PopulationOptimizer<V> {
    rng: SmallRng,
    values: Vec<V>,
    best: usize,
}

pub trait Value: Sized {
    fn evolve(&self, max_actions: usize, rng: &mut SmallRng) -> Vec<Self>;
    fn weight(&self) -> f64;
}

impl<V: Value> PopulationOptimizer<V> {
    pub fn new(rng: SmallRng, initial_values: Vec<V>) -> Self {
        let mut optimizer = PopulationOptimizer {
            rng,
            values: initial_values,
            best: 0,
        };
        optimizer.update_best();
        optimizer
    }

    pub fn evolve_step(&mut self, max_actions: usize, max_values: usize) {
        // Create new values
        let mut new_values = vec![];
        for value in &self.values {
            new_values.extend(value.evolve(max_actions, &mut self.rng));
        }
        self.values.extend(new_values);

        if self.values.len() > max_values {
            log::debug!(
                "Will sample {} out of {} values",
                max_values,
                self.values.len()
            );
            log::debug!(
                "Value weights: {}",
                self.values.iter().map(|value| value.weight()).format(", ")
            );

            let values = &self.values;
            let indexes = index::sample_weighted(
                &mut self.rng,
                values.len(),
                |index| values[index].weight(),
                max_values,
            )
            .unwrap()
            .into_vec();

            let mut i = 0;
            self.values.retain(|_| {
                let retain = indexes.contains(&i);
                i += 1;
                retain
            });
        }

        self.update_best();
    }

    pub fn best(&self) -> &V {
        &self.values[self.best]
    }

    pub fn into_best(mut self) -> V {
        self.values.swap_remove(self.best)
    }

    pub fn evolve_era(&mut self, patience: usize, max_actions: usize, max_values: usize) {
        log::info!(
            "Start era with patience = {}, max_actions = {}, max_values = {}",
            patience,
            max_actions,
            max_values
        );

        let mut prev_weight = 0.;
        let mut repeated = 0;
        let mut step = 0;
        loop {
            let best = self.best();
            if prev_weight >= best.weight() {
                repeated += 1;
                if repeated == patience {
                    break;
                }
            } else {
                repeated = 0;
            }
            prev_weight = best.weight();

            log::info!(
                "Start step {} with {} individuals. Best weight = {}, patience {}/{}",
                step,
                self.values.len(),
                best.weight(),
                repeated,
                patience
            );
            self.evolve_step(max_actions, max_values);
            step += 1;
        }
    }

    pub fn values(&self) -> &[V] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut Vec<V> {
        &mut self.values
    }

    fn update_best(&mut self) {
        self.best = self
            .values
            .iter()
            .position_max_by_key(|value| OrderedFloat(value.weight()))
            .unwrap();
    }
}
