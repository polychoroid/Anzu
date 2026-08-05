use nalgebra::Vector3;
use std::sync::OnceLock;

use crate::assets::Mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Point,
    Triangle,
    Square,
    Rectangle,
    Pentagon,
    Hexagon,
    Heptagon,
    Octagon,
    Nonagon,
    Decagon,
    Tetrahedron,
    Cube,
    Prism,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

pub fn get_primitive_mesh_ref(primitive: &Primitive) -> &'static Mesh {
    match primitive {
        Primitive::Point => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Point))
        }
        Primitive::Triangle => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Triangle))
        }
        Primitive::Square => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Square))
        }
        Primitive::Rectangle => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Rectangle))
        }
        Primitive::Pentagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Pentagon))
        }
        Primitive::Hexagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Hexagon))
        }
        Primitive::Heptagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Heptagon))
        }
        Primitive::Octagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Octagon))
        }
        Primitive::Nonagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Nonagon))
        }
        Primitive::Decagon => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Decagon))
        }
        Primitive::Tetrahedron => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Tetrahedron))
        }
        Primitive::Cube => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Cube))
        }
        Primitive::Prism => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Prism))
        }
        Primitive::Octahedron => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Octahedron))
        }
        Primitive::Dodecahedron => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Dodecahedron))
        }
        Primitive::Icosahedron => {
            static MESH: OnceLock<Mesh> = OnceLock::new();
            MESH.get_or_init(|| get_primitive_mesh(&Primitive::Icosahedron))
        }
    }
}

fn get_primitive_mesh(primitive: &Primitive) -> Mesh {
    match primitive {
        Primitive::Point => Mesh {
            id: 1,
            points: vec![Vector3::new(0.0, 0.0, 0.0)],
            index: None,
            uv: Some(vec![[0.5, 0.5]]),
        },
        Primitive::Triangle => Mesh {
            id: 2,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(-0.5, -0.5, 0.0),
                Vector3::new(0.5, -0.5, 0.0),
            ],
            index: Some(vec![0, 1, 2]),
            uv: Some(vec![[0.5, 0.0], [0.0, 1.0], [1.0, 1.0]]),
        },
        Primitive::Square => Mesh {
            id: 3,
            points: vec![
                Vector3::new(-0.5, -0.5, 0.0),
                Vector3::new(0.5, -0.5, 0.0),
                Vector3::new(0.5, 0.5, 0.0),
                Vector3::new(-0.5, 0.5, 0.0),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3]),
            uv: Some(vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]),
        },
        Primitive::Rectangle => Mesh {
            id: 4,
            points: vec![
                Vector3::new(-0.75, -0.5, 0.0),
                Vector3::new(0.75, -0.5, 0.0),
                Vector3::new(0.75, 0.5, 0.0),
                Vector3::new(-0.75, 0.5, 0.0),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3]),
            uv: Some(vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]),
        },
        Primitive::Pentagon => Mesh {
            id: 5,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.47552827, 0.1545085, 0.0),
                Vector3::new(0.29389262, -0.4045085, 0.0),
                Vector3::new(-0.29389262, -0.4045085, 0.0),
                Vector3::new(-0.47552827, 0.1545085, 0.0),
            ],
            index: Some(vec![0, 2, 1, 0, 3, 2, 0, 4, 3]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.7377641, 0.3454915],
                [0.6469463, 0.9045085],
                [0.3530537, 0.9045085],
                [0.26223588, 0.3454915],
            ]),
        },
        Primitive::Hexagon => Mesh {
            id: 6,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.4330127, 0.25, 0.0),
                Vector3::new(0.4330127, -0.25, 0.0),
                Vector3::new(0.0, -0.5, 0.0),
                Vector3::new(-0.4330127, -0.25, 0.0),
                Vector3::new(-0.4330127, 0.25, 0.0),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.9330127, 0.25],
                [0.9330127, 0.75],
                [0.5, 1.0],
                [0.0669873, 0.75],
                [0.0669873, 0.25],
            ]),
        },
        Primitive::Heptagon => Mesh {
            id: 7,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.39091575, 0.3117449, 0.0),
                Vector3::new(0.48746395, -0.11126047, 0.0),
                Vector3::new(0.21694186, -0.45048445, 0.0),
                Vector3::new(-0.21694186, -0.45048445, 0.0),
                Vector3::new(-0.48746395, -0.11126047, 0.0),
                Vector3::new(-0.39091575, 0.3117449, 0.0),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.89091575, 0.1882551],
                [0.98746395, 0.6112605],
                [0.71694183, 0.95048445],
                [0.28305814, 0.95048445],
                [0.01253605, 0.6112605],
                [0.10908425, 0.1882551],
            ]),
        },
        Primitive::Octagon => Mesh {
            id: 8,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.35355338, 0.35355338, 0.0),
                Vector3::new(0.5, 0.0, 0.0),
                Vector3::new(0.35355338, -0.35355338, 0.0),
                Vector3::new(0.0, -0.5, 0.0),
                Vector3::new(-0.35355338, -0.35355338, 0.0),
                Vector3::new(-0.5, 0.0, 0.0),
                Vector3::new(-0.35355338, 0.35355338, 0.0),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 7]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.8535534, 0.14644662],
                [1.0, 0.5],
                [0.8535534, 0.8535534],
                [0.5, 1.0],
                [0.14644662, 0.8535534],
                [0.0, 0.5],
                [0.14644662, 0.14644662],
            ]),
        },
        Primitive::Nonagon => Mesh {
            id: 9,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.3213938, 0.38302222, 0.0),
                Vector3::new(0.49240386, 0.08682409, 0.0),
                Vector3::new(0.4330127, -0.25, 0.0),
                Vector3::new(0.17101008, -0.4698463, 0.0),
                Vector3::new(-0.17101008, -0.4698463, 0.0),
                Vector3::new(-0.4330127, -0.25, 0.0),
                Vector3::new(-0.49240386, 0.08682409, 0.0),
                Vector3::new(-0.3213938, 0.38302222, 0.0),
            ],
            index: Some(vec![
                0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 7, 0, 7, 8,
            ]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.8213938, 0.11697778],
                [0.99240386, 0.4131759],
                [0.9330127, 0.75],
                [0.6710101, 0.9698463],
                [0.32898992, 0.9698463],
                [0.066987306, 0.75],
                [0.007596135, 0.4131759],
                [0.17860621, 0.11697778],
            ]),
        },
        Primitive::Decagon => Mesh {
            id: 10,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.29389262, 0.4045085, 0.0),
                Vector3::new(0.47552827, 0.1545085, 0.0),
                Vector3::new(0.47552827, -0.1545085, 0.0),
                Vector3::new(0.29389262, -0.4045085, 0.0),
                Vector3::new(0.0, -0.5, 0.0),
                Vector3::new(-0.29389262, -0.4045085, 0.0),
                Vector3::new(-0.47552827, -0.1545085, 0.0),
                Vector3::new(-0.47552827, 0.1545085, 0.0),
                Vector3::new(-0.29389262, 0.4045085, 0.0),
            ],
            index: Some(vec![
                0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 7, 0, 7, 8, 0, 8, 9,
            ]),
            uv: Some(vec![
                [0.5, 0.0],
                [0.7938926, 0.0954915],
                [0.97552824, 0.3454915],
                [0.97552824, 0.6545085],
                [0.7938926, 0.9045085],
                [0.5, 1.0],
                [0.20610738, 0.9045085],
                [0.02447173, 0.6545085],
                [0.02447173, 0.3454915],
                [0.20610738, 0.0954915],
            ]),
        },
        Primitive::Tetrahedron => Mesh {
            id: 11,
            // Regular tetrahedron, equivalent to the (+/-1, +/-1, +/-1) coordinate family.
            points: vec![
                Vector3::new(0.28867513, 0.28867513, 0.28867513),
                Vector3::new(0.28867513, -0.28867513, -0.28867513),
                Vector3::new(-0.28867513, 0.28867513, -0.28867513),
                Vector3::new(-0.28867513, -0.28867513, 0.28867513),
            ],
            index: Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2]),
            uv: Some(vec![
                [0.7886751, 0.21132487],
                [0.7886751, 0.7886751],
                [0.21132487, 0.21132487],
                [0.21132487, 0.7886751],
            ]),
        },
        Primitive::Cube => Mesh {
            id: 12,
            points: vec![
                Vector3::new(-0.5, -0.5, -0.5),
                Vector3::new(0.5, -0.5, -0.5),
                Vector3::new(0.5, 0.5, -0.5),
                Vector3::new(-0.5, 0.5, -0.5),
                Vector3::new(-0.5, -0.5, 0.5),
                Vector3::new(0.5, -0.5, 0.5),
                Vector3::new(0.5, 0.5, 0.5),
                Vector3::new(-0.5, 0.5, 0.5),
            ],
            index: Some(vec![
                0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2,
                7, 3, 3, 7, 4, 3, 4, 0,
            ]),
            uv: Some(vec![
                [0.0, 1.0],
                [1.0, 1.0],
                [1.0, 0.0],
                [0.0, 0.0],
                [0.0, 1.0],
                [1.0, 1.0],
                [1.0, 0.0],
                [0.0, 0.0],
            ]),
        },
        Primitive::Prism => Mesh {
            id: 13,
            points: vec![
                Vector3::new(-0.5, -0.28867513, -0.5),
                Vector3::new(0.5, -0.28867513, -0.5),
                Vector3::new(0.0, 0.57735026, -0.5),
                Vector3::new(-0.5, -0.28867513, 0.5),
                Vector3::new(0.5, -0.28867513, 0.5),
                Vector3::new(0.0, 0.57735026, 0.5),
            ],
            index: Some(vec![
                0, 1, 2, 3, 5, 4, 0, 3, 4, 0, 4, 1, 1, 4, 5, 1, 5, 2, 2, 5, 3, 2, 3, 0,
            ]),
            uv: Some(vec![
                [0.0, 0.7886751],
                [1.0, 0.7886751],
                [0.5, 0.0],
                [0.0, 0.7886751],
                [1.0, 0.7886751],
                [0.5, 0.0],
            ]),
        },
        Primitive::Octahedron => Mesh {
            id: 14,
            points: vec![
                Vector3::new(0.0, 0.0, 0.5),
                Vector3::new(0.5, 0.0, 0.0),
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(-0.5, 0.0, 0.0),
                Vector3::new(0.0, -0.5, 0.0),
                Vector3::new(0.0, 0.0, -0.5),
            ],
            index: Some(vec![
                0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1, 5, 2, 1, 5, 3, 2, 5, 4, 3, 5, 1, 4,
            ]),
            uv: Some(vec![
                [0.5, 0.5],
                [1.0, 0.5],
                [0.5, 0.0],
                [0.0, 0.5],
                [0.5, 1.0],
                [0.5, 0.5],
            ]),
        },
        Primitive::Icosahedron => {
            const PHI: f32 = 1.618_034;
            const SCALE: f32 = 0.262_866;

            Mesh {
                id: 15,
                points: vec![
                    Vector3::new(-1.0 * SCALE, PHI * SCALE, 0.0),
                    Vector3::new(1.0 * SCALE, PHI * SCALE, 0.0),
                    Vector3::new(-1.0 * SCALE, -1.0 * PHI * SCALE, 0.0),
                    Vector3::new(1.0 * SCALE, -1.0 * PHI * SCALE, 0.0),
                    Vector3::new(0.0, -1.0 * SCALE, PHI * SCALE),
                    Vector3::new(0.0, 1.0 * SCALE, PHI * SCALE),
                    Vector3::new(0.0, -1.0 * SCALE, -1.0 * PHI * SCALE),
                    Vector3::new(0.0, 1.0 * SCALE, -1.0 * PHI * SCALE),
                    Vector3::new(PHI * SCALE, 0.0, -1.0 * SCALE),
                    Vector3::new(PHI * SCALE, 0.0, 1.0 * SCALE),
                    Vector3::new(-1.0 * PHI * SCALE, 0.0, -1.0 * SCALE),
                    Vector3::new(-1.0 * PHI * SCALE, 0.0, 1.0 * SCALE),
                ],
                index: Some(vec![
                    0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2,
                    10, 7, 6, 7, 1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4,
                    11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
                ]),
                uv: Some(vec![
                    [0.237134, 0.074675],
                    [0.762866, 0.074675],
                    [0.237134, 0.925325],
                    [0.762866, 0.925325],
                    [0.5, 0.762866],
                    [0.5, 0.237134],
                    [0.5, 0.762866],
                    [0.5, 0.237134],
                    [0.925325, 0.5],
                    [0.925325, 0.5],
                    [0.074675, 0.5],
                    [0.074675, 0.5],
                ]),
            }
        }
        Primitive::Dodecahedron => {
            const PHI: f32 = 1.618_034;
            const INV_PHI: f32 = 0.618_034;
            const SCALE: f32 = 0.309_017;

            Mesh {
                id: 16,
                // Regular dodecahedron coordinate family:
                // (±1,±1,±1), (0,±1/phi,±phi), (±1/phi,±phi,0), (±phi,0,±1/phi)
                points: vec![
                    Vector3::new(-1.0 * SCALE, -1.0 * SCALE, -1.0 * SCALE),
                    Vector3::new(-1.0 * SCALE, -1.0 * SCALE, 1.0 * SCALE),
                    Vector3::new(-1.0 * SCALE, 1.0 * SCALE, -1.0 * SCALE),
                    Vector3::new(-1.0 * SCALE, 1.0 * SCALE, 1.0 * SCALE),
                    Vector3::new(1.0 * SCALE, -1.0 * SCALE, -1.0 * SCALE),
                    Vector3::new(1.0 * SCALE, -1.0 * SCALE, 1.0 * SCALE),
                    Vector3::new(1.0 * SCALE, 1.0 * SCALE, -1.0 * SCALE),
                    Vector3::new(1.0 * SCALE, 1.0 * SCALE, 1.0 * SCALE),
                    Vector3::new(0.0, -INV_PHI * SCALE, -PHI * SCALE),
                    Vector3::new(0.0, -INV_PHI * SCALE, PHI * SCALE),
                    Vector3::new(0.0, INV_PHI * SCALE, -PHI * SCALE),
                    Vector3::new(0.0, INV_PHI * SCALE, PHI * SCALE),
                    Vector3::new(-INV_PHI * SCALE, -PHI * SCALE, 0.0),
                    Vector3::new(-INV_PHI * SCALE, PHI * SCALE, 0.0),
                    Vector3::new(INV_PHI * SCALE, -PHI * SCALE, 0.0),
                    Vector3::new(INV_PHI * SCALE, PHI * SCALE, 0.0),
                    Vector3::new(-PHI * SCALE, 0.0, -INV_PHI * SCALE),
                    Vector3::new(PHI * SCALE, 0.0, -INV_PHI * SCALE),
                    Vector3::new(-PHI * SCALE, 0.0, INV_PHI * SCALE),
                    Vector3::new(PHI * SCALE, 0.0, INV_PHI * SCALE),
                ],
                index: Some(vec![
                    0, 8, 10, 0, 10, 2, 0, 2, 16, 0, 16, 12, 0, 12, 8, 1, 9, 13, 1, 13, 3, 1, 3,
                    18, 1, 18, 12, 1, 12, 9, 4, 14, 19, 4, 19, 5, 4, 5, 17, 4, 17, 8, 4, 8, 14, 6,
                    10, 17, 6, 17, 5, 6, 5, 15, 6, 15, 7, 6, 7, 10, 2, 10, 7, 2, 7, 13, 2, 13, 16,
                    3, 11, 9, 3, 9, 12, 3, 12, 18, 5, 19, 11, 5, 11, 15, 7, 15, 11, 7, 11, 13, 8,
                    17, 10, 9, 11, 19, 9, 19, 14, 9, 14, 8, 16, 13, 18, 14, 12, 1,
                ]),
                uv: Some(vec![
                    [0.190983, 0.809017],
                    [0.190983, 0.809017],
                    [0.190983, 0.190983],
                    [0.190983, 0.190983],
                    [0.809017, 0.809017],
                    [0.809017, 0.809017],
                    [0.809017, 0.190983],
                    [0.809017, 0.190983],
                    [0.5, 0.690983],
                    [0.5, 0.690983],
                    [0.5, 0.309017],
                    [0.5, 0.309017],
                    [0.309017, 1.0],
                    [0.309017, 0.0],
                    [0.690983, 1.0],
                    [0.690983, 0.0],
                    [0.0, 0.5],
                    [1.0, 0.5],
                    [0.0, 0.5],
                    [1.0, 0.5],
                ]),
            }
        }
    }
}

pub fn validate_mesh_indices(mesh: &Mesh) -> anyhow::Result<()> {
    let Some(indices) = mesh.indices() else {
        return Ok(());
    };

    if indices.is_empty() {
        anyhow::bail!("indexed mesh must contain at least one index");
    }

    if indices.len() % 3 != 0 {
        anyhow::bail!(
            "indexed mesh must contain a multiple of 3 indices for triangle-list draws; got {}",
            indices.len()
        );
    }

    for index in indices {
        let idx = usize::from(*index);
        if idx >= mesh.points.len() {
            anyhow::bail!(
                "mesh index {} is out of bounds for {} points",
                idx,
                mesh.points.len()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_PRIMITIVES: [Primitive; 16] = [
        Primitive::Point,
        Primitive::Triangle,
        Primitive::Square,
        Primitive::Rectangle,
        Primitive::Pentagon,
        Primitive::Hexagon,
        Primitive::Heptagon,
        Primitive::Octagon,
        Primitive::Nonagon,
        Primitive::Decagon,
        Primitive::Tetrahedron,
        Primitive::Cube,
        Primitive::Prism,
        Primitive::Octahedron,
        Primitive::Dodecahedron,
        Primitive::Icosahedron,
    ];

    #[test]
    fn mesh_provider_triangle_is_centered_and_ccw() {
        let mesh = get_primitive_mesh(&Primitive::Triangle);
        let indices = mesh.indices().expect("triangle mesh must define indices");

        assert_eq!(indices, [0, 1, 2]);

        let min_x = mesh
            .points
            .iter()
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = mesh
            .points
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = mesh
            .points
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min);
        let max_y = mesh
            .points
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!((min_x + max_x).abs() <= f32::EPSILON);
        assert!((min_y + max_y).abs() <= f32::EPSILON);
        assert!(
            mesh.points
                .iter()
                .all(|p| p.x.abs() <= 1.0 && p.y.abs() <= 1.0)
        );
    }

    #[test]
    fn mesh_provider_triangle_indices_are_in_bounds() {
        let mesh = get_primitive_mesh(&Primitive::Triangle);
        validate_mesh_indices(&mesh).expect("canonical indexed mesh must validate");

        let mut malformed = get_primitive_mesh(&Primitive::Triangle);
        malformed.index = Some(vec![0, 1, 3]);

        assert!(validate_mesh_indices(&malformed).is_err());
    }

    #[test]
    fn mesh_provider_pentagon_indices_are_ccw_for_renderer() {
        let mesh = get_primitive_mesh(&Primitive::Pentagon);
        let indices = mesh.indices().expect("pentagon mesh must define indices");

        assert_eq!(indices, [0, 2, 1, 0, 3, 2, 0, 4, 3]);
    }

    #[test]
    fn mesh_provider_primitives_define_uvs_matching_points() {
        for primitive in ALL_PRIMITIVES {
            let mesh = get_primitive_mesh(&primitive);
            let uv = mesh
                .uv
                .as_ref()
                .expect("every primitive mesh must define UV coordinates");

            assert_eq!(
                uv.len(),
                mesh.points.len(),
                "primitive {:?} UV count must match point count",
                primitive
            );
            assert!(
                uv.iter()
                    .all(|coord| (0.0..=1.0).contains(&coord[0]) && (0.0..=1.0).contains(&coord[1])),
                "primitive {:?} UVs must be normalized to [0.0, 1.0]",
                primitive
            );
        }
    }

    #[test]
    fn mesh_provider_triangle_uvs_use_top_left_image_origin() {
        let mesh = get_primitive_mesh(&Primitive::Triangle);
        let uv = mesh
            .uv
            .as_ref()
            .expect("triangle mesh must define UV coordinates");

        assert_eq!(uv, &vec![[0.5, 0.0], [0.0, 1.0], [1.0, 1.0]]);
    }
}
