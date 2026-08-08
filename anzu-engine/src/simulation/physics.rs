use crate::assets::{Mesh, MeshType, mesh_provider, mesh_provider::Primitive};
use nalgebra::{Matrix3, Matrix4, SymmetricEigen, Vector3};
use std::default::Default;

pub type RuntimeTransformFn = Box<dyn Fn(&BodyState, u64) -> Matrix4<f32>>;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MotionVector {
    pub linear: Vector3<f32>,
    pub angular: Vector3<f32>,
}

impl Default for MotionVector {
    fn default() -> Self {
        return MotionVector {
            linear: Vector3::new(0.0, 0.0, 0.0),
            angular: Vector3::new(0.0, 0.0, 0.0),
        };
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ForceVector {
    pub force: Vector3<f32>,
    pub torque: Vector3<f32>,
}

impl Default for ForceVector {
    fn default() -> Self {
        return ForceVector {
            force: Vector3::new(0.0, 0.0, 0.0),
            torque: Vector3::new(0.0, 0.0, 0.0),
        };
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MassProperties {
    mass: f32,
    inverse_mass: f32,
    inertia: Matrix3<f32>,
    inverse_inertia: Matrix3<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpacialProperties {
    centroid: Vector3<f32>,
    axes: Matrix3<f32>,
    variances: Vector3<f32>,
    extents: Vector3<f32>,
}

pub struct BodyState {
    physics_mesh_id: u32,
    motion_vector: MotionVector,
    force_vector: ForceVector,
    spacial_properties: SpacialProperties,
    mass_properties: MassProperties,
    runtime_transform_fn: RuntimeTransformFn,
}

impl BodyState {
    pub fn motion_vector(&self) -> MotionVector {
        self.motion_vector
    }

    fn default_runtime_transform(body: &BodyState, step_index: u64) -> Matrix4<f32> {
        let phase = step_index as f32 * 0.06;
        let rotation =
            nalgebra::Rotation3::from_euler_angles(0.0, body.motion_vector.angular.y * phase, 0.0);
        let mut matrix = Matrix4::identity();
        matrix
            .fixed_view_mut::<3, 3>(0, 0)
            .copy_from(&rotation.to_homogeneous().fixed_view::<3, 3>(0, 0));
        matrix
    }

    pub fn runtime_transform(&self, step_index: u64) -> Matrix4<f32> {
        (self.runtime_transform_fn)(self, step_index)
    }

    fn get_box_inertia(mass: &f32, extents: &Vector3<f32>) -> Matrix3<f32> {
        let x = extents.x;
        let y = extents.y;
        let z = extents.z;

        let inertia_vector = Vector3::new(
            mass * (y * y + z * z) / 12.0,
            mass * (x * x + z * z) / 12.0,
            mass * (x * x + y * y) / 12.0,
        );

        Matrix3::<f32>::new(
            inertia_vector.x,
            0.0,
            0.0,
            0.0,
            inertia_vector.y,
            0.0,
            0.0,
            0.0,
            inertia_vector.z,
        )
    }

    fn calculate_centroid(mesh: &Mesh) -> Vector3<f32> {
        let n = mesh.points.len();

        assert!(n > 0);

        return (1 / n) as f32 * mesh.points.iter().sum::<Vector3<f32>>();
    }

    fn calculate_covarience_matrix(mesh: &Mesh, centroid: &Vector3<f32>) -> Matrix3<f32> {
        let mut covarience_matrix = Matrix3::<f32>::zeros();

        for p in mesh.points.iter() {
            let d = p - centroid;
            covarience_matrix += d * d.transpose();
        }

        return covarience_matrix;
    }

    fn calculate_extents(
        centroid: &Vector3<f32>,
        covarience_matrix_eigen_vectors: &Matrix3<f32>,
        mesh: &Mesh,
    ) -> Vector3<f32> {
        let mut min = Vector3::<f32>::zeros();
        let mut max = Vector3::<f32>::zeros();

        for p in mesh.points.iter() {
            let local = covarience_matrix_eigen_vectors.transpose() * (p - centroid);

            min.x = min.x.min(local.x);
            min.y = min.y.min(local.y);
            min.z = min.z.min(local.z);

            max.x = max.x.max(local.x);
            max.y = max.y.max(local.y);
            max.z = max.z.max(local.z);
        }

        Vector3::new(max.x - min.x, max.y - min.y, max.z - min.z)
    }

    fn calculate_spacial_properties(mesh: &Mesh) -> SpacialProperties {
        let centroid = Self::calculate_centroid(&mesh);

        let covarience_matrix = Self::calculate_covarience_matrix(&mesh, &centroid);

        let eigen_decomposition = SymmetricEigen::new(covarience_matrix);

        let eigen_vectors = eigen_decomposition.eigenvectors;
        let eigen_values = eigen_decomposition.eigenvalues;

        let extents = Self::calculate_extents(&centroid, &eigen_vectors, &mesh);

        SpacialProperties {
            centroid: centroid,
            axes: eigen_vectors,
            variances: eigen_values,
            extents: extents,
        }
    }

    pub fn new(
        mass: f32,
        mesh_type: MeshType,
        initial_motion: Option<MotionVector>,
        initial_force: Option<ForceVector>,
    ) -> Self {
        Self::new_with_runtime_transform(mass, mesh_type, initial_motion, initial_force, None)
    }

    pub fn new_with_runtime_transform(
        mass: f32,
        mesh_type: MeshType,
        initial_motion: Option<MotionVector>,
        initial_force: Option<ForceVector>,
        runtime_transform_fn: Option<RuntimeTransformFn>,
    ) -> Self {
        let mesh = match mesh_type {
            MeshType::BuiltIn(primitive) => mesh_provider::get_primitive_mesh_ref(&primitive),
            MeshType::FromFile(_file) => mesh_provider::get_primitive_mesh_ref(&Primitive::Cube), // mesh loading is not supported yet
        };

        let motion_vector = match initial_motion {
            Some(motion_vector) => motion_vector,
            None => MotionVector::default(),
        };

        let force_vector = match initial_force {
            Some(force_vector) => force_vector,
            None => ForceVector::default(),
        };

        let spacial_properties = Self::calculate_spacial_properties(mesh);

        let inertia = Self::get_box_inertia(&mass, &spacial_properties.extents);

        let inverse_inertia = match inertia.try_inverse() {
            Some(inverse) => inverse,
            None => Matrix3::<f32>::zeros(),
        };

        let runtime_transform_fn = match runtime_transform_fn {
            Some(transform_fn) => transform_fn,
            None => Box::new(Self::default_runtime_transform),
        };

        return BodyState {
            physics_mesh_id: mesh.id,
            motion_vector: motion_vector,
            force_vector: force_vector,
            spacial_properties: spacial_properties,
            mass_properties: MassProperties {
                mass,
                inverse_mass: 1.0 / mass,
                inertia: inertia,
                inverse_inertia: inverse_inertia,
            },
            runtime_transform_fn,
        };
    }
}

impl Default for BodyState {
    fn default() -> Self {
        Self::new(1.0, MeshType::BuiltIn(Primitive::Cube), None, None)
    }
}

#[test]
fn build_default_body_state() {
    let body = BodyState::default();
    assert_eq!(body.physics_mesh_id, 12);
    assert_eq!(body.motion_vector.linear, Vector3::zeros());
    assert_eq!(body.motion_vector.angular, Vector3::zeros());
    assert_eq!(body.force_vector.force, Vector3::zeros());
    assert_eq!(body.force_vector.torque, Vector3::zeros());
    assert_eq!(body.spacial_properties.centroid, Vector3::zeros());
    assert_eq!(
        body.spacial_properties.axes,
        Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    );
    assert_eq!(body.spacial_properties.extents, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(
        body.spacial_properties.variances,
        Vector3::new(2.0, 2.0, 2.0)
    );
    assert_eq!(
        body.mass_properties.inertia,
        Matrix3::new(
            1.0 / 6.0,
            0.0,
            0.0,
            0.0,
            1.0 / 6.0,
            0.0,
            0.0,
            0.0,
            1.0 / 6.0
        )
    );
    assert_eq!(
        body.mass_properties.inverse_inertia,
        Matrix3::new(6.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 6.0)
    );
    assert_eq!(body.mass_properties.mass, 1.0);
}

#[test]
fn build_dodecahedron_body_state() {
    let body = BodyState::new(
        1.0,
        MeshType::BuiltIn(Primitive::Dodecahedron),
        Some(MotionVector {
            linear: Vector3::new(1.0, 1.0, 1.0),
            angular: Vector3::new(1.0, 1.0, 1.0),
        }),
        Some(ForceVector {
            force: Vector3::new(1.0, 1.0, 1.0),
            torque: Vector3::new(1.0, 1.0, 1.0),
        }),
    );
    assert_eq!(body.physics_mesh_id, 16);
    assert_eq!(body.motion_vector.linear, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(body.motion_vector.angular, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(body.force_vector.force, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(body.force_vector.torque, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(body.spacial_properties.centroid, Vector3::zeros());
    assert_eq!(
        body.spacial_properties.axes,
        Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    );
    assert_eq!(body.spacial_properties.extents, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(
        body.mass_properties.inertia,
        Matrix3::new(
            1.0 / 6.0,
            0.0,
            0.0,
            0.0,
            1.0 / 6.0,
            0.0,
            0.0,
            0.0,
            1.0 / 6.0
        )
    );
    assert_eq!(
        body.mass_properties.inverse_inertia,
        Matrix3::new(6.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 6.0)
    );
    assert_eq!(body.mass_properties.mass, 1.0);
}
