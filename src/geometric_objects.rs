use euclid::Vector3D;
/// Represents an infinite plane.
pub struct Plane {
	pub position: Vector3D::<f64,super::PositionU>,
	pub normal: Vector3D::<f64,super::PositionU>,
}
/// Represents a sphere.
pub struct Sphere {
	pub center: Vector3D::<f64,super::PositionU>,
	pub radius: f64,
}
impl Sphere {
	pub fn is_inside(&self, point: &Vector3D<f64,super::PositionU>) -> bool {
		(self.center - *point).length() < self.radius
	}
}
