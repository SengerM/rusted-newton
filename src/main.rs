use euclid;

enum Position {}
enum Velocity {}

#[derive(Debug)]
struct Particle {
    position: euclid::Vector3D::<f64, Position>,
    velocity: euclid::Vector3D::<f64, Velocity>,
}

fn main() {
    let mut particle = Particle {
        position: euclid::Vector3D::<f64, Position>::new(0.,0.,0.),
        velocity: euclid::Vector3D::<f64, Velocity>::new(0.,0.,0.),
    };
    
    dbg!(&particle);
    
    particle.position += euclid::Vector3D::<f64, Position>::new(1.,0.,0.);
    particle.velocity += euclid::Vector3D::<f64, Velocity>::new(5.,0.,0.);
    
    dbg!(&particle);
}
