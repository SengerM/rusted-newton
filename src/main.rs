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
struct Particle {
    position: Vector3D::<f64, PositionU>,
    velocity: Vector3D::<f64, VelocityU>,
    mass: MassU,
}

type ParticleIdx = usize;

/// Represents an interaction between two particles, which will lead to a force.
enum Interaction {
    force_between_two_particles(ParticleIdx,ParticleIdx,Force),
    external_force(ParticleIdx,ExternalForce),
}

/// Represents a force.
enum Force {
    Elastic(f64, f64),
    Damping(f64),
    Gravitational,
}

impl Force {
    /// Computes the force acting on `particle_1` due to this interaction.
    fn acting_on_a(&self, a: &Particle, b: &Particle) -> Vector3D<f64, ForceU> {
        let r = b.position-a.position;
        match self {
            Force::Elastic(k, d0) => (r.normalize()*(r.length() - (*d0))*(*k)).cast_unit(),
            Force::Damping(c) => (r.normalize()*((b.velocity-a.velocity).dot(r.cast_unit())*(*c))).cast_unit(),
            Force::Gravitational => (r.normalize()*a.mass*b.mass/r.square_length()).cast_unit(),
        }
    }
    /// Computes the force acting on `particle_2` due to this interaction.
    fn acting_on_b(&self, a: &Particle, b: &Particle) -> Vector3D<f64, ForceU> {
        self.acting_on_a(a, b) * -1.
    }
}

/// Represents an external force, i.e. a force that acts on a particle due to some external agent.
enum ExternalForce {
    LinearDrag(f64),
}

impl ExternalForce {
    fn calculate_force(&self, a: &Particle) -> Vector3D<f64, ForceU> {
        match self {
            ExternalForce::LinearDrag(b) => (a.velocity*(*b)).cast_unit()*-1.,
        }
    }
}

enum Constraint {
    external_constraint(ParticleIdx,ExternalConstraint),
}

/// Represents an infinite wall that divides space in two halves: 1) Outside the wall and 2) Inside the wall.
struct Plane {
    position: Vector3D::<f64,PositionU>,
    orientation: Vector3D::<f64,PositionU>, // Points towards the outside of the wall.
}
/// Represents a sphere.
struct Sphere {
    center: Vector3D::<f64,PositionU>,
    radius: f64,
}
impl Sphere {
    fn is_inside(&self, point: &Vector3D<f64,PositionU>) -> bool {
        (self.center - *point).length() < self.radius
    }
}

enum ExternalConstraint {
    infinite_wall(Plane),
    spherical_container(Sphere),
}

impl ExternalConstraint {
    fn compute_new_dynamical_variables(&self, particle: &Particle) -> (Vector3D<f64,PositionU>, Vector3D<f64,VelocityU>) {
        match self {
            ExternalConstraint::infinite_wall(wall) => {
                let d = (particle.position - wall.position).dot(wall.orientation);
                if d < 0. {
                    let new_vel: Vector3D<f64,VelocityU> = particle.velocity - (wall.orientation.normalize()*2.0*(particle.velocity.dot(wall.orientation.normalize().cast_unit()))).cast_unit();
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
                    let new_vel: Vector3D<f64,VelocityU> = particle.velocity - particle.velocity.project_onto_vector(radial_direction.cast_unit())*2.;
                    (particle.position, new_vel)
                }
            }
        }
    }
}

/// Represents a system of particles, i.e. a collection of particles that interact.
struct ParticlesSystem {
    particles: Vec<Particle>,
    interactions: Vec<Interaction>,
    constraints: Vec<Constraint>,
    time: f64,
    n_time_saved_to_sql: usize,
}

impl ParticlesSystem {
    /// Creates an empty particles system.  
    fn new() -> Self {
        Self {
            particles: Vec::<Particle>::new(),
            interactions: Vec::<Interaction>::new(),
            constraints:  Vec::<Constraint>::new(),
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
    fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }
    /// Add a constraint.
    fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    /// Advance the time and update the system.
    fn advance_time(&mut self, time_step: f64) {
        // First we compute the acceleration of each particle using the interactions:
        let mut accelerations = vec![Vector3D::<f64,AccelerationU>::zero(); self.particles.len()]; // A vector with one acceleration for each particle.
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
            let dv: Vector3D::<f64,VelocityU> = a.cast_unit()*time_step;
            let dr: Vector3D::<f64,PositionU> = p.velocity.cast_unit()*time_step + dv.cast_unit()*time_step/2.;
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
    
    system.add_particle(
        Particle {
            position: Vector3D::<f64,PositionU>::new(1.,0.,0.).normalize(),
            velocity: Vector3D::<f64,VelocityU>::new(0.,1.,0.).normalize(),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,PositionU>::new(0.,1.,0.).normalize(),
            velocity: Vector3D::<f64,VelocityU>::new(-1.,0.,0.).normalize(),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,PositionU>::new(-1.,0.,0.).normalize(),
            velocity: Vector3D::<f64,VelocityU>::new(0.,-1.,0.).normalize(),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,PositionU>::new(0.,-1.,0.).normalize(),
            velocity: Vector3D::<f64,VelocityU>::new(-1.,1.,0.).normalize(),
            mass: 1.,
        }
    );
    
    system.add_interaction(
        Interaction::force_between_two_particles(
            0,
            1,
            Force::Elastic(1.,0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            1,
            2,
            Force::Elastic(1.,0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            2,
            3,
            Force::Elastic(1.,0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            3,
            0,
            Force::Elastic(1.,0.5),
        )
    );
    
    system.add_interaction(
        Interaction::force_between_two_particles(
            0,
            1,
            Force::Damping(0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            1,
            2,
            Force::Damping(0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            2,
            3,
            Force::Damping(0.5),
        )
    );
    system.add_interaction(
        Interaction::force_between_two_particles(
            3,
            0,
            Force::Damping(0.5),
        )
    );

    for particle_idx in 0..4 {
        system.add_constraint(
            Constraint::external_constraint(
                particle_idx,
                ExternalConstraint::spherical_container(
                    Sphere {
                        center: Vector3D::<f64,PositionU>::new(0.,0.,0.),
                        radius: 1.,
                    }
                )
            )
        )
    }
    
    //~ for idx in 0..4 {
        //~ for xy in [-1.,1.] {
            //~ system.add_constraint(
                //~ Constraint::external_constraint(
                    //~ idx,
                    //~ ExternalConstraint::infinite_wall(
                        //~ Plane {
                            //~ position: Vector3D::<f64,PositionU>::new(xy,0.,0.),
                            //~ orientation: Vector3D::<f64,PositionU>::new(-xy,0.,0.),
                        //~ }
                    //~ )
                //~ )
            //~ );
            //~ system.add_constraint(
                //~ Constraint::external_constraint(
                    //~ idx,
                    //~ ExternalConstraint::infinite_wall(
                        //~ Plane {
                            //~ position: Vector3D::<f64,PositionU>::new(0.,xy,0.),
                            //~ orientation: Vector3D::<f64,PositionU>::new(0.,xy,0.),
                        //~ }
                    //~ )
                //~ )
            //~ );
        //~ }
    //~ }


    let connection = system.create_sqlite_connection(&String::from("/home/msenger/Desktop/newton.db"));
    system.dump_to_sqlite(&connection); // Save initial state.
    for n_time in 1..999999 {
        system.advance_time(0.00001);
        if n_time % 9999 == 0 {
            system.dump_to_sqlite(&connection);
        }
    }
}
