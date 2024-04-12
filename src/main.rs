use euclid::Vector3D;
use sqlite;

/// Defines units of position.
enum PositionU {}
/// Defines units of velocity.
enum VelocityU {}
/// Defines units of acceleration;
enum AccelerationU {}
enum ForceU {}
/// Defines units of mass.
type MassU = f64;

/// Represents the concept of a particle in classical mechanics.
#[derive(Debug)]
struct Particle {
    position: Vector3D::<f64, PositionU>,
    velocity: Vector3D::<f64, VelocityU>,
    mass: MassU,
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
    fn force_acting_on_particle_1(&self, particles: &[Particle]) -> Vector3D<f64, ForceU> {
        let a = particles[self.particle_1_idx].position;
        let b = particles[self.particle_2_idx].position;
        Vector3D::<f64, ForceU>::new(b.x - a.x, b.y - a.y, b.z - a.z).normalize()
    }
    /// Computes the force acting on `particle_2` due to this interaction.
    fn force_acting_on_particle_2(&self, particles: &[Particle]) -> Vector3D<f64, ForceU> {
        self.force_acting_on_particle_1(particles) * -1.
    }
}

/// Represents a system of particles, i.e. a collection of particles that interact.
#[derive(Debug)]
struct ParticlesSystem {
    particles: Vec<Particle>,
    interactions: Vec<Interaction>,
    time: f64,
    n_time_saved_to_sql: usize,
}

impl ParticlesSystem {
    /// Creates an empty particles system.  
    fn new() -> Self {
        Self {
            particles: Vec::<Particle>::new(),
            interactions: Vec::<Interaction>::new(),
            time: 0.,
            n_time_saved_to_sql: 0,
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
        let mut accelerations = vec![Vector3D::<f64,AccelerationU>::zero(); self.particles.len()]; // A vector with one acceleration for each particle.
        for interaction in &self.interactions {
            accelerations[interaction.particle_1_idx] += interaction.force_acting_on_particle_1(&self.particles).cast_unit()/self.particles[interaction.particle_1_idx].mass;
            accelerations[interaction.particle_2_idx] += interaction.force_acting_on_particle_2(&self.particles).cast_unit()/self.particles[interaction.particle_2_idx].mass;
        }
        // Now we move the system forward in time:
        for (n_particle,p) in self.particles.iter_mut().enumerate() {
            let a = accelerations[n_particle];
            let dv: Vector3D::<f64,VelocityU> = a.cast_unit()*time_step;
            let dr: Vector3D::<f64,PositionU> = p.velocity.cast_unit()*time_step + dv.cast_unit()*time_step/2.;
            p.position = p.position + dr;
            p.velocity = p.velocity + dv;
        }
        self.time += time_step;
	}
    /// Creates an SQLite file to save the data.
    fn create_sqlite_connection(&self, file_name: &String) -> sqlite::Connection {
        let connection = sqlite::open(file_name).unwrap();
        connection.execute("CREATE TABLE particles_system (n_time INTEGER, n_particle INTEGER, position_x FLOAT, position_y FLOAT, position_z FLOAT, velocity_x FLOAT, velocity_y FLOAT, velocity_z FLOAT, mass FLOAT);").unwrap();
        connection.execute("CREATE TABLE time (n_time INTEGER, time FLOAT);").unwrap();
        connection
    }
    /// Save the state of the system into an SQLite file.
    fn dump_to_sqlite(&mut self, connection: &sqlite::Connection) {
        for (n_particle,p) in self.particles.iter().enumerate() {
            let mut query = String::new();
            query.push_str("INSERT INTO particles_system VALUES (");
            query.push_str(&self.n_time_saved_to_sql.to_string());
            query.push_str(", ");
            query.push_str(&n_particle.to_string());
            query.push_str(", ");
            query.push_str(&p.position.x.to_string());
            query.push_str(", ");
            query.push_str(&p.position.y.to_string());
            query.push_str(", ");
            query.push_str(&p.position.z.to_string());
            query.push_str(", ");
            query.push_str(&p.velocity.x.to_string());
            query.push_str(", ");
            query.push_str(&p.velocity.y.to_string());
            query.push_str(", ");
            query.push_str(&p.velocity.z.to_string());
            query.push_str(", ");
            query.push_str(&p.mass.to_string());
            query.push_str(");");
            connection.execute(query).unwrap();
        }
        let mut query = String::new();
        query.push_str("INSERT INTO time VALUES (");
        query.push_str(&self.n_time_saved_to_sql.to_string());
        query.push_str(", ");
        query.push_str(&self.time.to_string());
        query.push_str(");");
        connection.execute(query).unwrap();

        self.n_time_saved_to_sql += 1;
    }
}

fn main() {
	let mut system = ParticlesSystem::new();
    
    let p = Particle {
        position: Vector3D::<f64,PositionU>::new(-1.,0.,0.),
        velocity: Vector3D::<f64,VelocityU>::new(0.,0.,0.),
        mass: 1.,
    };
    system.add_particle(p);
    let p = Particle {
        position: Vector3D::<f64,PositionU>::new(1.,0.,0.),
        velocity: Vector3D::<f64,VelocityU>::new(0.,0.,0.),
        mass: 2.,
    };
    system.add_particle(p);
    let p = Particle {
        position: Vector3D::<f64,PositionU>::new(0.,1.,0.),
        velocity: Vector3D::<f64,VelocityU>::new(0.,0.,0.),
        mass: 3.,
    };
    system.add_particle(p);
    let p = Particle {
        position: Vector3D::<f64,PositionU>::new(0.,-1.,0.),
        velocity: Vector3D::<f64,VelocityU>::new(0.,0.,0.),
        mass: 4.,
    };
    system.add_particle(p);
    
    system.add_interaction(0,1);
    system.add_interaction(1,2);
    system.add_interaction(2,3);

    let connection = system.create_sqlite_connection(&String::from("/home/msenger/Desktop/newton.db"));
    system.dump_to_sqlite(&connection); // Save initial state.
    for n_time in 1..9999999 {
        system.advance_time(0.00001);
        if n_time % 9999 == 0 {
            system.dump_to_sqlite(&connection);
        }
    }
}
