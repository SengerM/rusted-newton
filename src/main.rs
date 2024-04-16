#![allow(warnings)]
use rand::distributions::{Distribution, Uniform};
use euclid::Vector3D;
use particles_system::{units, Particle, ParticlesSystem, Interaction, Force, Constraint, ExternalConstraint, ExternalForce};

mod geometric_objects;
mod particles_system;

fn main() {
	let mut system = ParticlesSystem::new();

    const N_PARTICLES: usize = 11;

    let step = Uniform::new(-0.5, 0.5);
    let mut rng = rand::thread_rng();

    for n_particle in 1..(N_PARTICLES+1) {
        system.add_particle(
            Particle {
                position: Vector3D::<f64,units::Position>::new(step.sample(&mut rng),step.sample(&mut rng),0.),
                velocity: Vector3D::<f64,units::Velocity>::new(0.,0.,0.),
                mass: 1.,
            }
        );
    }
    
    for n_particle in 0..system.particles.len() {
        system.add_interaction(
            Interaction::external_force(
                n_particle,
                ExternalForce::Gravitational(Vector3D::<f64,units::Acceleration>::new(0.,-1.,0.)),
            )
        );
        system.add_interaction(
            Interaction::external_force(
                n_particle,
                ExternalForce::LinearDrag(1.),
            )
        );
        system.add_constraint(
            Constraint::external_constraint(
                n_particle,
                ExternalConstraint::spherical_container(
                    geometric_objects::Sphere::<units::Position> {
                        center: Vector3D::<f64,units::Position>::new(0.,0.,0.),
                        radius: 1.,
                    }
                )
            )
        );
        for m_particle in 0..system.particles.len() {
            if m_particle <= n_particle {
                continue;
            }
            system.add_interaction(
                Interaction::force_between_two_particles(
                    n_particle,
                    m_particle,
                    Force::Sticky(0.2,0.21,10.,99.),
                )
            );
        }
    }

    let conn = system.create_sqlite_connection(&String::from("/home/msenger/Desktop/newton.db"));
    system.dump_to_sqlite(&conn); // Save initial state.
    for n_time in 1..999999 {
        system.advance_time(0.00001);
        if n_time % 9999 == 0 {
            system.dump_to_sqlite(&conn);
        }
    }
    system.to_json(&String::from("/home/msenger/Desktop/system.json"));
}
