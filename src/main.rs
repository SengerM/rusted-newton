use std::marker::PhantomData;

#[derive(Debug)]
struct PhysicalVector<T> {
    x: f64,
    y: f64,
    z: f64,
    _t: PhantomData<T>,
}

impl<T> PhysicalVector<T> {
    fn add(&mut self, other: &Self) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
        self.z = self.z + other.z;
    }
    
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            _t: PhantomData,
        }
    }
}

#[derive(Debug)]
struct Position;

#[derive(Debug)]
struct Velocity;

fn main() {
    let mut pos = PhysicalVector::<Position>::new(0., 0., 0.);
    //~ let mut vel = PhysicalVector::<Velocity> {x:1., y:0., z:0., _t:PhantomData};
    
    dbg!(&pos);
    
    pos.add(&PhysicalVector::<Position>::new(0., 0., 1.));
    pos.add(&PhysicalVector::<Velocity>::new(0., 0., 1.));
    
    dbg!(&pos);
}
