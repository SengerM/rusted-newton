use euclid::Vector3D;

/// Defines units of position.
enum Position {}
/// Defines units of velocity.
enum Velocity {}
/// Defines units of force.
enum Force {}

/// Represents the concept of a particle in classical mechanics.
#[derive(Debug)]
struct Particle {
    position: Vector3D::<f64, Position>,
    velocity: Vector3D::<f64, Velocity>,
    mass: f64,
}

/// Represents an interaction between two particles, which will lead to a force.
#[derive(Debug)]
struct Interaction<'a> {
	particle_1: &'a Particle,
	particle_2: &'a Particle,
}

impl <'a> Interaction<'a> {
	/// Computes the force acting on `particle_1` due to this interaction.
	fn force_acting_on_particle_1(&self) -> Vector3D<f64,Force> {
		let a = self.particle_1.position;
		let b = self.particle_2.position;
		Vector3D::<f64,Force>::new(b.x-a.x, b.y-a.y, b.z-a.z).normalize()
	}
	/// Computes the force acting on `particle_2` due to this interaction.
	fn force_acting_on_particle_2(&self) -> Vector3D<f64,Force> {
		self.force_acting_on_particle_1() * -1.
	}
}

/// Represents a system of particles, i.e. a collection of particles that interact.
#[derive(Debug)]
struct ParticlesSystem<'a> {
	particles: Vec::<&'a Particle>,
	interactions: Vec::<&'a Interaction<'a>>,
}

fn main() {
    let mut a = Particle {
        position: Vector3D::<f64,Position>::new(-1.,0.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 1.,
    };
    let mut b = Particle {
        position: Vector3D::<f64,Position>::new(1.,0.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 2.,
    };
    let mut c = Particle {
        position: Vector3D::<f64,Position>::new(0.,1.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 2.,
    };
    
    let interaction_ab = Interaction {
		particle_1: &a,
		particle_2: &b,
	};
    let interaction_ac = Interaction {
		particle_1: &a,
		particle_2: &c,
	};
    let interaction_bc = Interaction {
		particle_1: &b,
		particle_2: &c,
	};
	
	let system = ParticlesSystem{
		particles: vec![&a,&b,&c],
		interactions: vec![&interaction_ab, &interaction_ac, &interaction_bc],
	};
	dbg!(system);
}
