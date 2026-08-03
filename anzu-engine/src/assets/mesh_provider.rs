use nalgebra::Vector3;

use crate::assets::Mesh;

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

pub fn get_primitive_mesh(primitive: &Primitive) -> Mesh {
    match primitive {
        Primitive::Point => Mesh {
            id: 1,
            points: vec![Vector3::new(0.0, 0.0, 0.0)],
            index: None,
        },
        Primitive::Triangle => Mesh {
            id: 2,
            points: vec![
                Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(-0.5, -0.5, 0.0),
                Vector3::new(0.5, -0.5, 0.0),
            ],
            index: Some(vec![0, 1, 2]),
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
            index: Some(vec![0, 1, 2, 0, 2, 3, 0, 3, 4]),
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
            }
        }
    }
}
