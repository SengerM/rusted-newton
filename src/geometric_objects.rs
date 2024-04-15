use euclid::Vector3D;
/// Represents an infinite plane.
pub struct Plane<U> {
	pub position: Vector3D::<f64,U>,
	pub normal: Vector3D::<f64,U>,
}
/// Represents a sphere.
pub struct Sphere<U> {
	pub center: Vector3D::<f64,U>,
	pub radius: f64,
}
impl<U> Sphere<U> {
	pub fn is_inside(&self, point: &Vector3D<f64,U>) -> bool {
		(self.center - *point).length() < self.radius
	}
}
