use halo2_proofs::circuit::{Layouter, AssignedCell};
use halo2_proofs::{pasta::Fp, dev::MockProver};
use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};

// x^3 + x + 5 = 35
// x2 = x*x
// x3 = x2*x
// x3_x = x3 + x
// x3_x_5 = x3_x + 5
// x3_x_5 ==> 35

trait Ops {
    type Num;
    fn load_private(&self, layouter: impl Layouter<Fp>, x: Option<Fp>) -> Result<Self::Num, Error>;
    fn load_constant(&self, layouter: impl Layouter<Fp>, x: Fp) -> Result<Self::Num, Error>;
    fn mul(&self, layouter: impl Layouter<Fp>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
    fn add(&self, layouter: impl Layouter<Fp>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;

}
struct TheChip {
    config: TheConfig,
}

impl Ops for TheChip {
    type Num = AssignedCell<Fp, Fp>;
    fn load_private(&self, layouter: impl Layouter<Fp>, x: Option<Fp>) -> Result<Self::Num, Error> {
        layouter.assign_region(|| "load private", |mut region| {
            region.assign_advice(|| "private value", column, offset, to)
        })
    }

    fn load_constant(&self, layouter: impl Layouter<Fp>, x: Fp) -> Result<Self::Num, Error> {
        
    }

    fn mul(&self, layouter: impl Layouter<Fp>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        
    }

    fn add(&self, layouter: impl Layouter<Fp>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        
    }
}

impl TheChip {
    fn new(config: TheConfig) -> Self {
        TheChip {
            config,
        }
    }
}

#[derive(Clone, Copy)]
struct TheConfig {
    advice: [Column<Advice>; 2],
}
struct TheCircuit {
    constant: Fp,
    x: Option<Fp>
}

#[derive(Default)]
impl Circuit<Fp> for TheCircuit {
    type Config = TheConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        
    }

    fn synthesize(&self, config: Self::Config, layouter: impl halo2_proofs::circuit::Layouter<Fp>) -> Result<(), Error> {
        let chip = TheChip::new(config);

        let x = chip.load_private();
    }
}

fn main() {
    let x = Fp::from(3);
    let constant = Fp::from(5);
    let res = Fp::from(35);

    let circuit = TheCircuit {
        constant,
        x: Some(x)
    };

    let public_inputs = vec![res];

    let prover = MockProver::run(4, &circuit, vec![public_inputs]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
