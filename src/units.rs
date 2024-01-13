use glam::Vec3;

pub fn m_to_ft(v: f32) -> f32 {
    v * 3.2808399
}

pub fn m_to_ft_vec3(pos: Vec3) -> Vec3 {
    Vec3::new(m_to_ft(pos.x), m_to_ft(pos.y), m_to_ft(pos.z))
}

pub fn mps_to_miph(velocity: f32) -> f32 {
    velocity * 2.2369363
}
