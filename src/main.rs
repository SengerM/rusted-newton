use euclid::Vector3D;
use particles_system::{units, Particle, ParticlesSystem, Interaction, Force, Constraint, ExternalConstraint, ExternalForce};

mod geometric_objects;
mod particles_system;

fn main() {
	let mut system = ParticlesSystem::new();
    
    system.add_particle(
        Particle {
            position: Vector3D::<f64,units::Position>::new(1.,0.,0.),
            velocity: Vector3D::<f64,units::Velocity>::new(0.,0.,0.),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,units::Position>::new(0.,1.,0.),
            velocity: Vector3D::<f64,units::Velocity>::new(0.,0.,0.),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,units::Position>::new(-1.,0.,0.),
            velocity: Vector3D::<f64,units::Velocity>::new(0.,-0.,0.),
            mass: 1.,
        }
    );
    system.add_particle(
        Particle {
            position: Vector3D::<f64,units::Position>::new(0.,-1.,0.),
            velocity: Vector3D::<f64,units::Velocity>::new(-1.,0.,0.),
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

    for particle_idx in 0..system.particles.len() {
        system.add_interaction(
            Interaction::external_force(
                particle_idx,
                ExternalForce::Gravitational(Vector3D::<f64,units::Acceleration>::new(0.,-3.,0.)),
            )
        );
        system.add_constraint(
            Constraint::external_constraint(
                particle_idx,
                ExternalConstraint::spherical_container(
                    geometric_objects::Sphere::<units::Position> {
                        center: Vector3D::<f64,units::Position>::new(0.,0.,0.),
                        radius: 1.,
                    }
                )
            )
        )
    }

    let connection = system.create_sqlite_connection(&String::from("/home/msenger/Desktop/newton.db"));
    system.dump_to_sqlite(&connection); // Save initial state.
    for n_time in 1..999999 {
        system.advance_time(0.00001);
        if n_time % 9999 == 0 {
            system.dump_to_sqlite(&connection);
        }
    }
}
