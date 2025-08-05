use std::marker::PhantomData;

pub trait Simulation {
    type Backend;
    fn run(&self);
}

pub struct SimulationPlugin<T: Simulation> {
    phantom: PhantomData<T>,
}

