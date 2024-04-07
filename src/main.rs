#[derive(Debug)]
struct PhysicalVector {
    x: f64,
    y: f64,
    z: f64,
}

impl PhysicalVector {
    fn add(&mut self, other: &PhysicalVector) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
        self.z = self.z + other.z;
    }
}

#[derive(Debug)]
struct Particle {
    position: PhysicalVector,
    velocity: PhysicalVector,
}

fn main() {
    let mut stone = Particle {
        position: PhysicalVector {x: 0., y: 0., z: 0.},
        velocity: PhysicalVector {x: 0., y: 0., z: 1.},
    };
    
    dbg!(&stone);
    
    stone.position.add(&PhysicalVector{x: 1., y: 0., z: 0.});
    
    dbg!(&stone);
}
