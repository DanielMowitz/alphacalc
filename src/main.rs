extern crate natural_constants;
use natural_constants::physics::*;
use std::thread;
use flume;
use image::*;

const U:f64 = 1.6605390666050e-27; // Source: https://de.wikipedia.org/wiki/Atomare_Masseneinheit
const M_ALPHA:f64 = 6.644657335720e-27; // Source: https://de.wikipedia.org/wiki/Alphastrahlung;
const MEV_TO_JOULE:f64 = 1.0/6241509000000.0;
const FEMTO:f64 = 10e-15;
const MEGA:f64 = 10e6;

struct Particle{
	pos: (f64, f64),
	vel: (f64,f64)
}

impl Particle{
	fn new(p_x:f64, p_y:f64, v_x:f64, v_y:f64) -> Self {
		return Self{
			pos: (p_x, p_y),
			vel: (v_x, v_y)
		}
	}
}

fn calculate_acceleration(x:f64, y:f64, z_a:f64, z_k:f64, a:f64) -> (f64, f64){
	// Quantum theory function for particle acceleration. z_a and z_k are the proton number of the
	// alpha particle and the atom respctively. a is the mass number of the alpha particle.
	let mult = fine_structure_const*plancks_constant*speed_of_light_vac*z_a*z_k/(U*a*(x.powi(2)+y.powi(2)).sqrt().powi(3));
	return (mult*x, mult*y);
}

fn main() {

	// Controls the speed of the simulation by changing the factor of dt and divisor of the
	// iterations. 10000 is good for testing.
	let performance:u64 = 1;
	
	let energy = 5.0 * MEGA * MEV_TO_JOULE;
	
	// Calculation of speed from energy according to SRT.
	let speed = energy.sqrt()*speed_of_light_vac*(2.0*speed_of_light_vac.powi(2)*M_ALPHA + energy).sqrt()
		/(speed_of_light_vac.powi(2)*M_ALPHA + energy);
	
	// See calculate_acceleration comment for the meanung of these variables.
	let z_a = 2.0;
	let z_k = 79.0;
	let a = 4.0;

	// More or less arbitrary value
	let delta_t = 350.0 * (performance as f64) * 10e-29;

	// Calculate different starting positions
   let y10 = Particle::new(-500.0*FEMTO, 10.0*FEMTO, speed, 0.0);
	let y15 = Particle::new(-500.0*FEMTO, 15.0*FEMTO, speed, 0.0);
	let y30 = Particle::new(-500.0*FEMTO, 30.0*FEMTO, speed, 0.0);
	let y50 = Particle::new(-500.0*FEMTO, 50.0*FEMTO, speed, 0.0);
	let y100 = Particle::new(-500.0*FEMTO, 100.0*FEMTO, speed, 0.0);
	let y200 = Particle::new(-500.0*FEMTO, 200.0*FEMTO, speed, 0.0);

	// Used for sending data between threads
	let (tx, rx) = flume::unbounded();

	// Moves the particle and uses calculate_acceleration to get the new velocity.
	let update_and_send = move |tx: flume::Sender<(i32, f64, f64)>, mut particle: Particle, num| {
		for _ in 0..(10000000000/performance) {
			tx.send((num, particle.pos.0/FEMTO, particle.pos.1/FEMTO)).unwrap();
		
			particle.pos.0 = particle.pos.0 + particle.vel.0 * delta_t;
			particle.pos.1 = particle.pos.1 + particle.vel.1 * delta_t;
		
			let (dv_x, dv_y) = calculate_acceleration(particle.pos.0, particle.pos.1, z_a, z_k, a);
			particle.vel.0 = particle.vel.0 + dv_x * delta_t;
			particle.vel.1 = particle.vel.1 + dv_y * delta_t;
		}
	};

	// Create image buffers.
	let mut y10_plot = RgbImage::new(1001, 301);
	let mut y15_plot = RgbImage::new(1001, 301);
	let mut y30_plot = RgbImage::new(1001, 301);
	let mut y50_plot = RgbImage::new(1001, 301);
	let mut y100_plot = RgbImage::new(1001, 301);
	let mut y200_plot = RgbImage::new(1001, 301);
	
	// Create thread channels and open threads.
	let (tx1, tx2, tx3, tx4, tx5) = (tx.clone(), tx.clone(), tx.clone(), tx.clone(), tx.clone());
	thread::spawn(move || update_and_send(tx, y10, 0));
	thread::spawn(move || update_and_send(tx1, y15, 1));
	thread::spawn(move || update_and_send(tx2, y30, 2));
	thread::spawn(move || update_and_send(tx3, y50, 3));
	thread::spawn(move || update_and_send(tx4, y100, 4));
	thread::spawn(move || update_and_send(tx5, y200, 5));

	// A little Check.
	println!("v = {}c", speed/speed_of_light_vac);

	// Put data from every thread into the according images.
	for message in rx.iter() {
		if message.1 <= 500.0 && message.2 <= 300.0 {
			if message.0 == 0 {
				y10_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			} else if message.0 == 1 {
				y15_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			} else if message.0 == 2 {
				y30_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			} else if message.0 == 3 {
				y50_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			} else if message.0 == 4 {
				y100_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			} else if message.0 == 5 {
				y200_plot.put_pixel((message.1 + 500.0) as u32, message.2 as u32, Rgb([255, 0, 0]));
			}
		}
	}
	
	// Save images
	y10_plot.save("y10.png").unwrap();
	y15_plot.save("y15.png").unwrap();
	y30_plot.save("y30.png").unwrap();
	y50_plot.save("y50.png").unwrap();
	y100_plot.save("y100.png").unwrap();
	y200_plot.save("y200.png").unwrap();

}
