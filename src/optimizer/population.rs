use itertools::Itertools;
use ordered_float::NotNan;
use rand::rngs::SmallRng;
use rand::seq::index;

#[derive(Debug, Clone)]
pub struct PopulationOptimizer<V> {
    rng: SmallRng,
    values: Vec<WeightedValue<V>>,
    best: usize,
}

pub trait Value: Sized {
    fn evolve(&self, max_actions: usize, rng: &mut SmallRng) -> Vec<WeightedValue<Self>>;
    fn weight(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct WeightedValue<V> {
    value: V,
    weight: NotNan<f64>,
}

impl<V: Value> PopulationOptimizer<V> {
    pub fn new(rng: SmallRng, initial_values: Vec<V>) -> Self {
        let values = initial_values
            .into_iter()
            .map(WeightedValue::new)
            .collect_vec();

        let best = values
            .iter()
            .position_max_by_key(|value| value.weight)
            .unwrap();

        PopulationOptimizer { rng, values, best }
    }

    pub fn evolve(&mut self, max_actions: usize, max_values: usize) {
        // Create new values
        let mut new_values = vec![];
        for value in &self.values {
            new_values.extend(value.value.evolve(max_actions, &mut self.rng));
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
                self.values.iter().map(|value| value.weight).format(", ")
            );

            let values = &self.values;
            let indexes = index::sample_weighted(
                &mut self.rng,
                values.len(),
                |index| values[index].weight,
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

        self.best = self
            .values
            .iter()
            .position_max_by_key(|value| value.weight)
            .unwrap();
    }

    pub fn best(&self) -> &V {
        &self.values[self.best].value
    }

    pub fn into_best(mut self) -> V {
        self.values.swap_remove(self.best).value
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<V: Value> WeightedValue<V> {
    pub fn new(value: V) -> Self {
        WeightedValue {
            weight: NotNan::new(value.weight()).unwrap(),
            value,
        }
    }
}
