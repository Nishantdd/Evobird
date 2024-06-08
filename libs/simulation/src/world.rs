use crate::*;

#[derive(Debug)]
pub struct World{
    pub(crate) animals: Vec<Animal>,
    pub(crate) foods: Vec<Food>,
}
impl World{
    pub fn random(rng: &mut dyn RngCore) -> Self{
        let animals = (0..40)
            .map(|_| Animal::random(rng))
            .collect();

        let foods = (0..60)
            .map(|_| Food::random(rng))
            .collect();

        Self { animals, foods }
    }

    // Getter functions :
    pub fn animal(&self) -> &[Animal]{
        &self.animals
    }
    pub fn food(&self) -> &[Food]{
        &self.foods
    }
}