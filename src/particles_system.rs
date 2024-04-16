use euclid::Vector3D;
use sqlite;
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs;

use super::geometric_objects;

pub mod units {
	use serde::{Serialize, Deserialize};
	#[derive(Serialize, Deserialize)]
	#[derive(Debug)]
	pub enum Position {}
	#[derive(Serialize, Deserialize)]
	#[derive(Debug)]
	pub enum Velocity {}
	#[derive(Serialize, Deserialize)]
	#[derive(Debug)]
	pub enum Acceleration {}
	#[derive(Serialize, Deserialize)]
	#[derive(Debug)]
	pub enum Force {}
	pub type Mass = f64;
}

/// Represents the concept of a particle in classical mechanics.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Particle {
    pub position: Vector3D::<f64, units::Position>,
    pub velocity: Vector3D::<f64, units::Velocity>,
    pub mass: units::Mass,
}

type ParticleIdx = usize;

/// Represents an interaction between two particles, which will lead to a force.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Interaction {
    force_between_two_particles(ParticleIdx,ParticleIdx,Force),
    external_force(ParticleIdx,ExternalForce),
}

/// Represents a force.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Force {
	/// The force by an ideal spring, parameters are (k,d0).
    Elastic(f64, f64),
    /// The force by a linear damping, the parameter is the proportionality factor between velocity and force.
    Damping(f64),/// Represents a force.
    /// Gravitational force given by Newton's formula.
    Gravitational,
    /// A sticky force, parameters are (d_well, d_max, F_sticky, F_repuls).
    Sticky(f64, f64, f64, f64),
}

impl Force {
    /// Computes the force acting on `particle_1` due to this interaction.
    fn acting_on_a(&self, a: &Particle, b: &Particle) -> Vector3D<f64, units::Force> {
        let r = b.position-a.position;
        match self {
            Force::Elastic(k, d0) => (r.normalize()*(r.length() - (*d0))*(*k)).cast_unit(),
            Force::Damping(c) => (r.normalize()*((b.velocity-a.velocity).dot(r.cast_unit())*(*c))).cast_unit(),
            Force::Gravitational => (r.normalize()*a.mass*b.mass/r.square_length()).cast_unit(),
            Force::Sticky(d_well, d_max, F_sticky, F_repuls) => {
				let d = r.length();
				if d > *d_max {
					Vector3D::<f64,units::Force>::new(0.,0.,0.)
				} else if d > *d_well {
					// Sticky!
					(r.normalize()*(*F_sticky)).cast_unit()
				} else {
					// Repulsive
					(r.normalize()*(*F_repuls)).cast_unit()*-1.
				}
			}
        }
    }
    /// Computes the force acting on `particle_2` due to this interaction.
    fn acting_on_b(&self, a: &Particle, b: &Particle) -> Vector3D<f64, units::Force> {
        self.acting_on_a(a, b) * -1.
    }
}

/// Represents an external force, i.e. a force that acts on a particle due to some external agent.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum ExternalForce {
    LinearDrag(f64),
    Gravitational(Vector3D::<f64,units::Acceleration>),
}

impl ExternalForce {
    fn calculate_force(&self, a: &Particle) -> Vector3D<f64, units::Force> {
        match self {
            ExternalForce::LinearDrag(b) => (a.velocity*(*b)).cast_unit()*-1.,
            ExternalForce::Gravitational(g) => (*g*a.mass).cast_unit(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Constraint {
    external_constraint(ParticleIdx,ExternalConstraint),
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum ExternalConstraint {
    infinite_wall(geometric_objects::Plane<units::Position>),
    spherical_container(geometric_objects::Sphere<units::Position>),
}

impl ExternalConstraint {
    fn compute_new_dynamical_variables(&self, particle: &Particle) -> (Vector3D<f64,units::Position>, Vector3D<f64,units::Velocity>) {
        match self {
            ExternalConstraint::infinite_wall(wall) => {
                let d = (particle.position - wall.position).dot(wall.normal);
                if d < 0. {
                    let new_vel: Vector3D<f64,units::Velocity> = particle.velocity - (wall.normal.normalize()*2.0*(particle.velocity.dot(wall.normal.normalize().cast_unit()))).cast_unit();
                    (particle.position, new_vel)
                } else {
                    (particle.position, particle.velocity)
                }
            }
            ExternalConstraint::spherical_container(sphere) => {
                if sphere.is_inside(&particle.position) {
                    (particle.position, particle.velocity)
                } else {
                    let radial_direction = particle.position - sphere.center;
                    let new_vel: Vector3D<f64,units::Velocity> = particle.velocity - particle.velocity.project_onto_vector(radial_direction.cast_unit())*2.;
                    (particle.position, new_vel)
                }
            }
        }
    }
}

/// Represents a system of particles, i.e. a collection of particles that interact.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ParticlesSystem {
    pub particles: Vec<Particle>,
    pub interactions: Vec<Interaction>,
    pub constraints: Vec<Constraint>,
    time: f64,
    n_time_saved_to_sql: usize,
}

impl ParticlesSystem {
    /// Creates an empty particles system.  
    pub fn new() -> Self {
        Self {
            particles: Vec::<Particle>::new(),
            interactions: Vec::<Interaction>::new(),
            constraints:  Vec::<Constraint>::new(),
            time: 0.,
            n_time_saved_to_sql: 0,
        }
    }
    /// Add a particle to the system.
    pub fn add_particle(&mut self, p: Particle) -> usize {
        self.particles.push(p);
        self.particles.len()
    }
    /// Add an interaction between two particles of the system.
    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }
    /// Add a constraint.
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    /// Advance the time and update the system.
    pub fn advance_time(&mut self, time_step: f64) {
        // First we compute the acceleration of each particle using the interactions:
        let mut accelerations = vec![Vector3D::<f64,units::Acceleration>::zero(); self.particles.len()]; // A vector with one acceleration for each particle.
        for interaction in &self.interactions {
            match interaction {
                Interaction::force_between_two_particles(idx_a, idx_b, force) => {
                    let a = &self.particles[*idx_a];
                    let b = &self.particles[*idx_b];
                    accelerations[*idx_a] += force.acting_on_a(a,b).cast_unit()/a.mass;
                    accelerations[*idx_b] += force.acting_on_b(a,b).cast_unit()/b.mass;
                }
                Interaction::external_force(idx,force) => {
                    let a = &self.particles[*idx];
                    accelerations[*idx] += force.calculate_force(a).cast_unit()/a.mass;
                }
            }
        }
        // Now we move the system forward in time:
        for (n_particle,p) in self.particles.iter_mut().enumerate() {
            let a = accelerations[n_particle];
            let dv: Vector3D::<f64,units::Velocity> = a.cast_unit()*time_step;
            let dr: Vector3D::<f64,units::Position> = p.velocity.cast_unit()*time_step + dv.cast_unit()*time_step/2.;
            p.position = p.position + dr;
            p.velocity = p.velocity + dv;
        }
        // Now we check each constraint and make the required updates:
        for constraint in &self.constraints {
            match constraint {
                Constraint::external_constraint(idx,constraint) => {
                    let (new_pos, new_vel) = constraint.compute_new_dynamical_variables(&self.particles[*idx]);
                    self.particles[*idx].position = new_pos;
                    self.particles[*idx].velocity = new_vel;
                }
            }
        }
        self.time += time_step;
	}
    /// Creates an SQLite file to save the data.
    pub fn create_sqlite_connection(&self, file_name: &String) -> sqlite::Connection {
        let connection = sqlite::open(file_name).unwrap();
        connection.execute("CREATE TABLE particles_system (n_time INTEGER, n_particle INTEGER, position_x FLOAT, position_y FLOAT, position_z FLOAT, velocity_x FLOAT, velocity_y FLOAT, velocity_z FLOAT, mass FLOAT);").unwrap();
        connection.execute("CREATE TABLE time (n_time INTEGER, time FLOAT);").unwrap();
        connection
    }
    /// Save the state of the system into an SQLite file.
    pub fn dump_to_sqlite(&mut self, connection: &sqlite::Connection) {
		connection.execute("BEGIN TRANSACTION").unwrap();
        for (n_particle,p) in self.particles.iter().enumerate() {
            let n = &self.n_time_saved_to_sql;
            let n_particle = &n_particle;
            let pos_x = &p.position.x;
            let pos_y = &p.position.y;
            let pos_z = &p.position.z;
            let vel_x = &p.velocity.x;
            let vel_y = &p.velocity.y;
            let vel_z = &p.velocity.z;
            let m = &p.mass;
            connection.execute(
				format!("INSERT INTO particles_system VALUES ({n},{n_particle},{pos_x},{pos_y},{pos_z},{vel_x},{vel_y},{vel_z},{m});")
            ).unwrap();
        }
        let n_time = &self.n_time_saved_to_sql;
        let t = &self.time;
        connection.execute(
			format!("INSERT INTO time VALUES ({n_time},{t});")
        ).unwrap();
		connection.execute("COMMIT").unwrap();
        self.n_time_saved_to_sql += 1;
    }
	/// Save the system into a json file.
	pub fn to_json(&self, file_name: &String) {
		let json_str = serde_json::to_string(&self).expect("Failed to serialize");
		fs::write(file_name, json_str).expect("Unable to write file");
	}
	/// Create a new particles system from a json file.
	pub fn from_json(file_path: &String) -> Self {
		let data = fs::read_to_string(file_path).expect("Unable to read file");
		let system: ParticlesSystem = serde_json::from_str(&data).expect("Failed to deserialize");
		system
	}
}
