use crate::*;

#[derive(Debug)]
pub struct Brain {
    pub(crate) nn: nn::Network,
}

impl Brain {
    pub fn random(_rng: &mut dyn RngCore, eye: &Eye) -> Self {
        Self {
            nn: nn::Network::random( &Self::topology(eye)),
        }
    }

    pub(crate) fn from_chromosome(
        chromosome: ga::Chromosome,
        eye: &Eye,
    ) -> Self {
        Self {
            nn: nn::Network::from_weights(
                &Self::topology(eye),
                chromosome,
            ),
        }
    }

    pub(crate) fn as_chromosome(&self) -> ga::Chromosome {
        self.nn.weights().collect()
    }

    fn topology(eye: &Eye) -> [nn::LayerTopology; 3] {
        [
            nn::LayerTopology {
                neurons: eye.cells(),
            },
            nn::LayerTopology {
                neurons: 2 * eye.cells(),
            },
            nn::LayerTopology { neurons: 2 },
        ]
    }
}