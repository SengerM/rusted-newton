use euclid::Vector3D;

/// Defines units of position.
enum Position {}
/// Defines units of velocity.
enum Velocity {}
/// Defines units of acceleration;
enum Acceleration {}
enum Force {}
/// Defines units of mass.
type Mass = f64;

/// Represents the concept of a particle in classical mechanics.
#[derive(Debug)]
struct Particle {
    position: Vector3D::<f64, Position>,
    velocity: Vector3D::<f64, Velocity>,
    mass: Mass,
}

type ParticleIdx = usize;

/// Represents an interaction between two particles, which will lead to a force.
#[derive(Debug)]
struct Interaction {
    particle_1_idx: ParticleIdx,
    particle_2_idx: ParticleIdx,
}

impl Interaction {
    /// Computes the force acting on `particle_1` due to this interaction.
    fn force_acting_on_particle_1(&self, particles: &[Particle]) -> Vector3D<f64, Force> {
        let a = particles[self.particle_1_idx].position;
        let b = particles[self.particle_2_idx].position;
        Vector3D::<f64, Force>::new(b.x - a.x, b.y - a.y, b.z - a.z).normalize()
    }
    /// Computes the force acting on `particle_2` due to this interaction.
    fn force_acting_on_particle_2(&self, particles: &[Particle]) -> Vector3D<f64, Force> {
        self.force_acting_on_particle_1(particles) * -1.
    }
}

/// Represents a system of particles, i.e. a collection of particles that interact.
#[derive(Debug)]
struct ParticlesSystem {
    particles: Vec<Particle>,
    interactions: Vec<Interaction>,
}

impl ParticlesSystem {
    /// Creates an empty particles system.  
    fn new() -> Self {
        Self {
            particles: Vec::<Particle>::new(),
            interactions: Vec::<Interaction>::new(),
        }
    }
    /// Add a particle to the system.
    fn add_particle(&mut self, p: Particle) -> usize {
        self.particles.push(p);
        self.particles.len()
    }
    /// Add an interaction between two particles of the system.
    fn add_interaction(&mut self, p_id_a: ParticleIdx, p_id_b: ParticleIdx) {
        let interaction = Interaction { particle_1_idx: p_id_a, particle_2_idx: p_id_b };
        self.interactions.push(interaction);
    }
    /// Advance the time and update the system.
    fn advance_time(&mut self, time_step: f64) {
        // First we compute the acceleration of each particle using the interactions:
        let mut accelerations = vec![Vector3D::<f64,Acceleration>::zero(); self.particles.len()]; // A vector with one acceleration for each particle.
        for interaction in &self.interactions {
            accelerations[interaction.particle_1_idx] += interaction.force_acting_on_particle_1(&self.particles).cast_unit()/self.particles[interaction.particle_1_idx].mass;
            accelerations[interaction.particle_2_idx] += interaction.force_acting_on_particle_1(&self.particles).cast_unit()/self.particles[interaction.particle_2_idx].mass;
        }
        // Now we move the system forward in time:
        for (n_particle,p) in self.particles.iter_mut().enumerate() {
            let a = accelerations[n_particle];
            let dv: Vector3D::<f64,Velocity> = a.cast_unit()*time_step;
            let dr: Vector3D::<f64,Position> = p.velocity.cast_unit()*time_step + dv.cast_unit()*time_step/2.;
            p.position = p.position + dr;
            p.velocity = p.velocity + dv;
        }
	}
}

fn main() {
	let mut system = ParticlesSystem::new();
    
    let mut p = Particle {
        position: Vector3D::<f64,Position>::new(-1.,0.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 1.,
    };
    system.add_particle(p);
    let mut p = Particle {
        position: Vector3D::<f64,Position>::new(1.,0.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 2.,
    };
    system.add_particle(p);
    let mut p = Particle {
        position: Vector3D::<f64,Position>::new(0.,1.,0.),
        velocity: Vector3D::<f64,Velocity>::new(0.,0.,0.),
        mass: 3.,
    };
    system.add_particle(p);
    
    system.add_interaction(0,1);
    system.add_interaction(0,2);
    system.add_interaction(1,2);
	
	dbg!(&system);
    system.advance_time(1.);
    dbg!(&system);
}
