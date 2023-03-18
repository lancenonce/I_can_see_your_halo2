use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Chip};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector, Instance, Fixed};
use halo2_proofs::poly::Rotation;
use halo2_proofs::{dev::MockProver, pasta::Fp};

// x^3 + x + 5 = 35
// x2 = x*x
// x3 = x2*x
// x3_x = x3 + x
// x3_x_5 = x3_x + 5
// x3_x_5 ==> 35

//  |  a  |  b  |  m  |  s  |
//  |  x  |  x  |  1  |  0  |
//  | x2  |  x  |  1  |  0  |
//  | x3  |  x  |  0  |  1  |

// x3+x+5

trait Ops {
    type Num;
    fn load_private(&self, layouter: impl Layouter<Fp>, x: Option<Fp>) -> Result<Self::Num, Error>;
    fn load_constant(&self, layouter: impl Layouter<Fp>, x: Fp) -> Result<Self::Num, Error>;
    fn mul(
        &self,
        layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
    fn add(
        &self,
        layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
    fn expose_public(
        &self,
        layouter: impl Layouter<Fp>,
        num: Self::Num,
        row: usize,
    ) -> Result<Self::Num, Error>;
}
struct TheChip {
    config: TheConfig,
}

impl Chip<Fp> for TheChip {
    type Config = TheConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl Ops for TheChip {
    type Num = AssignedCell<Fp, Fp>;
    fn load_private(&self, layouter: impl Layouter<Fp>, v: Option<Fp>) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "load private",
            |mut region| {
                region.assign_advice(
                    || "private value",
                    config.advice[0],
                    0,
                    || v.ok_or(Error::Synthesis),
                )
            },
        )
    }

    fn load_constant(&self, layouter: impl Layouter<Fp>, x: Fp) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "load constant",
            |mut region| {
                region.assign_advice_from_constant(
                    || "constant",
                    config.advice[0],
                    0,
                    || x.ok_or(Error::Synthesis),
                )
            },
        )
    }

    fn mul(
        &self,
        layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "mul",
            |mut region| {
                config.s_mul.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;
                // multiply a and b
                let v = a.value().and_then(|a| b.value().map(|b| *a * *b));
                region.assign_advice(|| "a*b", config.advice[0], 1, || v.ok_or(Error::Synthesis))
            },
        )
    }

    fn add(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "add",
            |mut region| {
                config.s_add.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;
                // multiply a and b
                let v = a.value().and_then(|a| b.value().map(|b| *a + *b));
                region.assign_advice(|| "a+b", config.advice[0], 1, || v.ok_or(Error::Synthesis))
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<Fp>,
        num: Self::Num,
        row: usize,
    ) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.constrain_instance(num.cell(), config.instance, row)
    }
}

impl TheChip {
    fn new(config: TheConfig) -> Self {
        TheChip { config }
    }

    fn configure(
        meta: &mut ConstraintSystem<Fp>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
        constant: Column<Fixed>,
    ) -> TheConfig {
        meta.enable_constant(constant);
        meta.enable_equality(instance);
        for adv in advice.iter() {
            meta.enable_equality(adv);
        }
        let s_mul = meta.selector();
        let s_add = meta.selector();

        meta.create_gate(
            || "mul/add",
            |meta| {
                let lhs = meta.query_advice(advice[0], Rotation::cur());
                let rhs = meta.query_advice(advice[1], Rotation::cur());
                let out = meta.query_advice(advice[0], Rotation::next());
                let s_mul = meta.query_selector(s_mul);
                let s_add = meta.query_selector(s_add);

                vec![
                    s_mul * (lhs.clone() * rhs.clone() - out.clone()),
                    s_add * (lhs + rhs - out),
                ]
            },
        );
        TheConfig {
            advice,
            instance,
            s_mul,
            s_add,
        }
    }
}

#[derive(Clone, Debug)]
struct TheConfig {
    advice: [Column<Advice>; 2],
    instance: Column<Instance>,
    s_mul: Selector,
    s_add: Selector,
}
struct TheCircuit {
    constant: Fp,
    x: Option<Fp>,
}

impl Circuit<Fp> for TheCircuit {
    type Config = TheConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let advice = [meta.advice_column(), meta.advice_column()];
        let instance = meta.instance_column();
        let constant = meta.fixed_column();
        TheChip::configure(meta, advice, instance, constant)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl halo2_proofs::circuit::Layouter<Fp>,
    ) -> Result<(), Error> {
        let chip = TheChip::new(config);

        let x = chip.load_private(layouter.namespace(|| "load x"), self.x)?;
        let constant = chip.load_constant(layouter.namespace(|| "load constant"), self.constant)?;
        let x2 = chip.mul(layouter.namespace(|| "x2"), x.clone(), x.clone())?;
        let x3 = chip.mul(layouter.namespace(|| "x3"), x.clone(), x2.clone())?;
        let x3_x = chip.add(layouter.namespace(|| "x3+x"), x3.clone(), x.clone())?;
        let x3_x_5 = chip.add(
            layouter.namespace(|| "x3+x+5"),
            x3_x.clone(),
            constant.clone(),
        )?;

        chip.expose_public(layouter.namespace(|| "expose res"), x3_x_5, 0)
    }
}

fn main() {
    let x = Fp::from(3);
    let constant = Fp::from(5);
    let res = Fp::from(35);

    let circuit = TheCircuit {
        constant,
        x: Some(x),
    };

    let public_inputs = vec![res];

    let prover = MockProver::run(4, &circuit, vec![public_inputs]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
