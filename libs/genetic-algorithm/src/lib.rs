use rand::{seq::SliceRandom, Rng, RngCore};
use std::ops::Index;

pub struct GeneticAlgorithm<S,C,M>{
    selection_method:S,
    crossover_method:C,
    mutation_method:M,
}

impl<S,C,M> GeneticAlgorithm<S,C,M>
    where S:SelectionMethod,
          C:CrossoverMethod,
          M:MutationMethod,
    {
        pub fn new(
            selection_method:S, 
            crossover_method: C,
            mutation_method: M,
        ) -> Self {
            Self { selection_method, crossover_method, mutation_method }
        }

        pub fn evolve<I>(&self, rng: &mut dyn RngCore, population: &[I]) -> (Vec<I>, Statistics)
        where
            I: Individual,
        {
            assert!(!population.is_empty());
            
            let new_population = (0..population.len())
                .map(|_| {
                    // Selection
                    let parent_a = self.selection_method.select(rng, population).chromosome();
                    let parent_b = self.selection_method.select(rng, population).chromosome();
                    // Crossover
                    let mut child = self.crossover_method.crossover(rng, parent_a, parent_b);
                    // Mutation
                    self.mutation_method.mutate(rng, &mut child);
                    I::create(child)
                })
                .collect();

            let stats = Statistics::new(population);
            (new_population, stats)
        }
}

pub trait Individual {
    fn fitness(&self) -> f32;
    fn chromosome(&self) -> &Chromosome;
    fn create(chromosome: Chromosome) -> Self;
}

pub trait SelectionMethod {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual;
}

pub struct RouletteWheelSelection;
impl SelectionMethod for RouletteWheelSelection {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual,
    {
        population
        .choose_weighted(rng, |indiv| indiv.fitness())
        .expect("got an empty population")
    }
}


#[derive(Clone, Debug)]
pub struct Chromosome { //Carrying properties of our birds
    genes: Vec<f32>,
}
impl Chromosome {
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.genes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.genes.iter_mut()
    }
}

/*Bunch of useful properties for easy handling of chromosomes : */

// ---------------------------------------------------------------------
// | this is the type of expression you expect inside the square brackets
// |
// | e.g. if you implemented `Index<&str>`, you could write:
// |   chromosome["yass"]
// ---------------------------------------------------------------------
impl Index<usize> for Chromosome {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.genes[index]
    }
}
// ------------------------------------------------------------------------------------------
// | this is the type of the item an iterator should provide in order to be compatible
// | with our chromosome
// |
// | (sometimes it's called the type an iterator *yields*)
// |
// | intuitively, since our chromosome is built of of floating-point numbers, we
// | expect floating-point numbers in here as well
// -------------- ---------------------------------------------------------------------------
impl FromIterator<f32> for Chromosome {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self {
            genes: iter.into_iter().collect(),
        }
    }
}
// works in the opposite way - it converts a type into an iterator
impl IntoIterator for Chromosome {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

pub trait CrossoverMethod{
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome
    ) -> Chromosome;
}
#[derive(Clone, Debug)]
pub struct UniformCrossover;
impl CrossoverMethod for UniformCrossover{
    fn crossover(
            &self,
            rng: &mut dyn RngCore,
            parent_a: &Chromosome,
            parent_b: &Chromosome
        ) -> Chromosome {
            assert_eq!(parent_a.len(), parent_b.len());
            parent_a
                .iter()
                .zip(parent_b.iter())
                .map(|(&a, &b) | if rng.gen_bool(0.5) {a} else {b})
                .collect()
    }
}

pub trait MutationMethod{
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome);
}
#[derive(Clone, Debug)]
pub struct GaussianMutation {
    /// Probability of changing a gene:
    /// - 0.0 = no genes will be touched
    /// - 1.0 = all genes will be touched
    chance: f32,

    /// Magnitude of that change:
    /// - 0.0 = touched genes will not be modified
    /// - 3.0 = touched genes will be += or -= by at most 3.0
    coeff: f32,
}
impl GaussianMutation{
    pub fn new(chance:f32, coeff:f32) -> Self {
        assert!(chance >= 0.0 && chance <= 1.0);
        Self { chance, coeff }
    }
}
impl MutationMethod for GaussianMutation{
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome) {
        for gene in child.iter_mut(){
            let sign = if rng.gen_bool(0.5) {-1.0} else {1.0};

            if rng.gen_bool(self.chance as f64){
                *gene += sign * self.coeff * rng.gen::<f32>();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Statistics {
    pub min_fitness: f32,
    pub max_fitness: f32,
    pub avg_fitness: f32,
}
impl Statistics {
    fn new<I>(population: &[I]) -> Self
    where
        I: Individual,
    {
        assert!(!population.is_empty());

        let mut min_fitness = population[0].fitness();
        let mut max_fitness = min_fitness;
        let mut sum_fitness = 0.0;

        for individual in population {
            let fitness = individual.fitness();

            min_fitness = min_fitness.min(fitness);
            max_fitness = max_fitness.max(fitness);
            sum_fitness += fitness;
        }

        Self {
            min_fitness,
            max_fitness,
            avg_fitness: sum_fitness / (population.len() as f32),
        }
    }
}

// Testing the rand.SliceRandom and not leaving it on Developer's Trust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;


    #[derive(Clone, Debug, PartialEq)]
    pub enum TestIndividual {
        /// For tests that require access to the chromosome
        WithChromosome { chromosome: Chromosome },
        /// For tests that don't require access to the chromosome
        WithFitness { fitness: f32 },
    }
    impl TestIndividual {
        fn new(fitness: f32) -> Self {
            Self::WithFitness { fitness }
        }
    }
    impl Individual for TestIndividual {
        fn create(chromosome: Chromosome) -> Self {
            Self::WithChromosome { chromosome }
        }

        fn chromosome(&self) -> &Chromosome {
            match self {
                Self::WithChromosome { chromosome } => chromosome,
                Self::WithFitness { .. } => {
                    panic!("not supported for TestIndividual::WithFitness")
                }
            }
        }

        fn fitness(&self) -> f32 {
            match self {
                Self::WithChromosome { chromosome } => {
                    chromosome.iter().sum()
                    // ^ the simplest fitness function ever - we're just summing all the genes together
                }
                Self::WithFitness { fitness } => *fitness,
            }
        }
    }
    impl PartialEq for Chromosome {
        fn eq(&self, other: &Self) -> bool {
            approx::relative_eq!(self.genes.as_slice(), other.genes.as_slice())
        }
    }


    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    
    #[test]
    fn roulette_wheel_selection() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let population = vec![
            TestIndividual::new(2.0),
            TestIndividual::new(1.0),
            TestIndividual::new(4.0),
            TestIndividual::new(3.0),
        ];

        let mut actual_histogram = BTreeMap::new();

        //          /--| nothing special about this thousand;
        //          v  | a number as low as fifty might do the trick, too
        for _ in 0..1000 {
            let fitness = RouletteWheelSelection
                .select(&mut rng, &population)
                .fitness() as i32;

            *actual_histogram
                .entry(fitness)
                .or_insert(0) += 1;
        }

        let expected_histogram = BTreeMap::from_iter([
            // (fitness, how many times this fitness has been chosen)
            (1, 98),
            (2, 202),
            (3, 278),
            (4, 422),
        ]);

        assert_eq!(actual_histogram, expected_histogram);
    }

    #[test]
    fn uniform_crossover() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());
        let parent_a = (1..=100).map(|n| n as f32).collect();
        let parent_b = (1..=100).map(|n| -n as f32).collect();
        let child = UniformCrossover.crossover(&mut rng, &parent_a, &parent_b);

        // Number of genes different between 'child' and 'parent_a'
        let diff_a = child.iter().zip(parent_a).filter(|(c,p)| *c != p).count();
        let diff_b = child.iter().zip(parent_b).filter(|(c,p)| *c != p).count();

        assert_eq!(diff_a, 49); // Child inherited 49% of parent_a's genes
        assert_eq!(diff_b, 51); // Child inherited 51% of parent_b's genes
    }

    mod gaussian_mutation {
        use super::*;

        fn actual(chance: f32, coeff: f32) -> Vec<f32> {
            let mut rng = ChaCha8Rng::from_seed(Default::default());
            let mut child = vec![1.0, 2.0, 3.0, 4.0, 5.0].into_iter().collect();

            GaussianMutation::new(chance, coeff).mutate(&mut rng, &mut child);

            child.into_iter().collect()
        }
        mod given_zero_chance {
            use approx::assert_relative_eq;

            fn actual(coeff: f32) -> Vec<f32> {
                super::actual(0.0, coeff)
            }

            mod and_zero_coefficient {
                use super::*;

                #[test]
                fn does_not_change_the_original_chromosome() {
                    let actual = actual(0.0);
                    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }
            }

            mod and_nonzero_coefficient {
                use super::*;

                #[test]
                fn does_not_change_the_original_chromosome() {
                    let actual = actual(0.5);
                    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }
            }
        }

        mod given_fifty_fifty_chance {
            use approx::assert_relative_eq;

            fn actual(coeff: f32) -> Vec<f32>{
                super::actual(0.5, coeff)
            }

            mod and_zero_coefficient {
                use super::*;

                #[test]
                fn does_not_change_the_original_chromosome() {
                    let actual = actual(0.0);
                    let expected = vec![1.0,2.0,3.0,4.0,5.0];
                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }

            }

            mod and_nonzero_coefficient {
                use super::*;

                #[test]
                fn slightly_changes_the_original_chromosome() {
                    let actual = actual(0.5);
                    let expected = vec![1.0, 1.7756249, 3.0, 4.1596804, 5.0];
                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }
            }
        }

        mod given_max_chance {
            use approx::assert_relative_eq;

            fn actual(coeff:f32) -> Vec<f32>{
                super::actual(1.0, coeff)
            }
            
            mod and_zero_coefficient {
                use super::*;
                
                #[test]
                fn does_not_change_the_original_chromosome() {
                    let actual = actual(0.0);
                    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];
                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }
            }

            mod and_nonzero_coefficient {
                use super::*;
                
                #[test]
                fn entirely_changes_the_original_chromosome() {
                    let actual = actual(0.5);
                    let expected = vec![1.4545316, 2.1162078, 2.7756248, 3.9505124, 4.638691];
                    assert_relative_eq!(actual.as_slice(), expected.as_slice());
                }
            }
        }
    }



    #[test]
    fn genetic_algorithm() {

        fn individual(genes: &[f32]) -> TestIndividual {
            TestIndividual::create(genes.iter().cloned().collect())
        }

        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let ga = GeneticAlgorithm::new(
            RouletteWheelSelection,
            UniformCrossover,
            GaussianMutation::new(0.5, 0.5),
        );

        let mut population = vec![
            individual(&[0.0, 0.0, 0.0]),
            individual(&[1.0, 1.0, 1.0]),
            individual(&[1.0, 2.0, 1.0]),
            individual(&[1.0, 2.0, 4.0]),
        ];

        // We're running `.evolve()` a few times, so that the differences between the
        // input and output population are easier to spot.
        //
        // No particular reason for a number of 10 - this test would be fine for 5, 20 or
        // even 1000 generations - the only thing that'd change is the magnitude of the
        // difference between the populations.
        let mut _stats = Statistics::new(&population);
        for _ in 0..10 {
            (population, _stats) = ga.evolve(&mut rng, &population);
        }

        let expected_population = vec![
            individual(&[0.44769490, 2.0648358, 4.3058133]),
            individual(&[1.21268670, 1.5538777, 2.8869110]),
            individual(&[1.06176780, 2.2657390, 4.4287640]),
            individual(&[0.95909685, 2.4618788, 4.0247330]),
        ];

        assert_eq!(population, expected_population); // expected has better fitness for each individual so evolve function is working
    }
    
}