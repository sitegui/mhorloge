pub struct PopulationOptimizer<I, S> {
    population: Vec<I>,
    select: S,
}

pub trait Individual {
    type Action;
    type Score;

    fn possible_actions(&self) -> Vec<Self::Action>;
    fn evolve(&self, action: Self::Action) -> Self;
    fn score(&self) -> Self::Score;
}

pub trait Select<A, I> {
    fn select_actions(&mut self, actions: &mut Vec<A>);
    fn select_population(&mut self, population: &mut Vec<I>);
    fn best(&self, population: &[I]) -> usize;
}

impl<I: Individual<Action = A>, S: Select<A, I>, A> PopulationOptimizer<I, S> {
    pub fn new(initial: Vec<I>, select: S) -> Self {
        PopulationOptimizer {
            population: initial,
            select,
        }
    }

    fn evolve(&mut self) {
        // Create new individuals
        let mut new_individuals = vec![];
        for individual in &self.population {
            let mut actions = individual.possible_actions();
            self.select.select_actions(&mut actions);
            for action in actions {
                new_individuals.push(individual.evolve(action));
            }
        }
        self.population.extend(new_individuals);

        // Select
        self.select.select_population(&mut self.population);
    }

    fn population(&self) -> &[I] {
        &self.population
    }

    fn best(&self) -> &I {
        let best = self.select.best(&self.population);
        &self.population[best]
    }

    fn into_best(mut self) -> I {
        let best = self.select.best(&self.population);
        self.population.swap_remove(best)
    }
}
