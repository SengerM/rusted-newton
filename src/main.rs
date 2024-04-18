#![allow(warnings)]
use rand::distributions::{Distribution, Uniform};
use euclid::Vector3D;
use particles_system::{units, Particle, ParticlesSystem, Interaction, Force, Constraint, ExternalConstraint, ExternalForce};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

mod geometric_objects;
mod particles_system;

fn main() {
	let PATH_TO_SIMULATION_FOLDER = "/home/msenger/Desktop/rusted";
	let PATH_TO_JSON = format!("{PATH_TO_SIMULATION_FOLDER}/system.json");
	let PATH_TO_SQLITE = format!("{PATH_TO_SIMULATION_FOLDER}/data.db");
	const N_ITERATIONS: u64 = 99999;
	const TIME_STEP: f64 = 0.00001;
	const DUMP_DATA_EVERY_N_ITERATIONS: u64 = 999;

	let mut system = if Path::new(&PATH_TO_JSON).exists() {
		println!("Loading existent simulation...");
		let mut system = ParticlesSystem::from_json(&PATH_TO_JSON);
		system
	} else {
		println!("Creating new simulation...");
		let mut system = ParticlesSystem::new();

		const N_PARTICLES: usize = 5;

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
						},
						0.5,
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
						Force::Sticky(0.2,0.31,10.,99.),
					)
				);
			}
		}
		system
	};

	// Create simulation folder:
	fs::create_dir_all(&PATH_TO_SIMULATION_FOLDER).expect("Failed to create simulation directory");
    // Save initial state:
    system.to_json(&PATH_TO_JSON);
    let conn = system.create_sqlite_connection(&PATH_TO_SQLITE);
    system.dump_to_sqlite(&conn);

    // Simulate:
    let bar = ProgressBar::new(N_ITERATIONS);
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{eta_precise}]").unwrap());
    println!("Simulating...");
    for n_time in 1..N_ITERATIONS {
        bar.inc(1);
        system.advance_time(TIME_STEP);
        if n_time % DUMP_DATA_EVERY_N_ITERATIONS == 0 {
            system.dump_to_sqlite(&conn);
        }
    }
    bar.finish();
    system.to_json(&PATH_TO_JSON);
    println!("Simulation data saved in {PATH_TO_SIMULATION_FOLDER} ✅");
}
