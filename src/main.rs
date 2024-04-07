use euclid;

enum Position {}
enum Velocity {}

fn main() {
    
    let pos = euclid::Vector3D::<f64, Position>::new(0.,2.,0.);
    let pos2 = euclid::Vector3D::<f64, Velocity>::new(1.,1.,0.);
    
    println!("Result: {:?}", pos + pos2);

}
