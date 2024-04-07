#[derive(Debug)]
struct PhysicalVector {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug)]
struct Position {
    position: PhysicalVector,
}

#[derive(Debug)]
struct Velocity {
    velocity: PhysicalVector,
}

#[derive(Debug)]
struct Particle {
    position: Position,
    velocity: Velocity,
}

fn main() {
    let stone = Particle {
        position: Position {
            position: PhysicalVector {x: 0., y: 0., z: 0.},
        },
        velocity: Velocity {
            velocity: PhysicalVector {x: 0., y: 0., z: 1.},
        }
    };
    
    dbg!(&stone);
}
