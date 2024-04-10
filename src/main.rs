use euclid::Vector3D;

enum Position {} // Defines units of position.
enum Velocity {} // Defines units of velocity.
enum Force {} // Defines units of force.

#[derive(Debug)]
struct Particle {
    position: Vector3D::<f64, Position>,
    velocity: Vector3D::<f64, Velocity>,
    mass: f64,
}

#[derive(Debug)]
struct Interaction<'a> {
	particle_1: &'a Particle,
	particle_2: &'a Particle,
}

impl <'a> Interaction<'a> {
	fn force_acting_on_particle_1(&self) -> Vector3D<f64,Force> {
		let a = self.particle_1.position;
		let b = self.particle_2.position;
		Vector3D::<f64,Force>::new(b.x-a.x, b.y-a.y, b.z-a.z).normalize()
	}
	fn force_acting_on_particle_2(&self) -> Vector3D<f64,Force> {
		self.force_acting_on_particle_1() * -1.
	}
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
	
	dbg!(interaction_ac.force_acting_on_particle_1());
	dbg!(interaction_ac.force_acting_on_particle_2());
	
	let particles = vec![&a,&b];
	let interactions = vec![&interaction_ab, &interaction_ac, &interaction_bc];
}
