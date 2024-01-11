use std::iter;

use glam::{Quat, Vec3};
use xmlwriter::XmlWriter;

#[derive(Debug)]
pub struct TrackSpline {
    pub points: Vec<(Vec3, f32)>, // pos, forward, roll
}

impl TrackSpline {
    pub fn new() -> Self {
        TrackSpline { points: Vec::new() }
    }

    pub fn length(&self) -> f32 {
        self.points
            .windows(2)
            .map(|p| (p[0].0 - p[1].0).length())
            .sum()
    }

    pub fn evaluate(&self, t: f32) -> Option<(Vec3, f32)> {
        if self.points.len() < 2 {
            return None; // Need at least 2 points for a spline
        }

        let t_clamped = (t / self.length()).clamp(0., 1.);

        let segments = self.points.len() - 1;
        let segment_idx = (t_clamped * segments as f32).floor() as usize;
        let t_in_segment = t_clamped * segments as f32 - segment_idx as f32;

        if let (Some(start), Some(end)) = (
            self.points.get(segment_idx),
            self.points.get(segment_idx + 1),
        ) {
            let interpolated_point = Vec3::new(
                start.0.x + t_in_segment * (end.0.x - start.0.x),
                start.0.y + t_in_segment * (end.0.y - start.0.y),
                start.0.z + t_in_segment * (end.0.z - start.0.z),
            );

            Some((
                interpolated_point,
                start.1 + t_in_segment * (end.1 - start.1),
            ))
        } else {
            None
        }
    }

    pub fn to_nolimits_element(&self) -> String {
        let opt = xmlwriter::Options {
            indent: xmlwriter::Indent::None,
            ..xmlwriter::Options::default()
        };

        let mut w = XmlWriter::new(opt);

        w.write_declaration();
        w.start_element("root");
        w.start_element("element");
        w.start_element("description");
        w.write_text("elimerl's fvd export");
        w.end_element();
        for point in &self.points {
            w.start_element("vertex");
            w.start_element("x");
            w.write_text_fmt(format_args!("{:.5}", point.0.x));
            w.end_element();
            w.start_element("y");
            w.write_text_fmt(format_args!("{:.5}", point.0.y));
            w.end_element();
            w.start_element("z");
            w.write_text_fmt(format_args!("{:.5}", point.0.z));
            w.end_element();
            w.start_element("strict");
            w.write_text("false");
            w.end_element();
            w.end_element();
        }
        let mut length_so_far = 0.;
        for points in self.points.windows(2).chain(std::iter::once(
            &[
                self.points[self.points.len() - 1],
                self.points[self.points.len() - 1],
            ][..],
        )) {
            let point = points[0];
            let next_point = points[1];
            w.start_element("roll");
            let quat = Quat::from_euler(glam::EulerRot::XYZ, 0., 0., point.1);
            let right = quat * Vec3::X;
            let up = quat * Vec3::Y;

            w.start_element("ux");
            w.write_text_fmt(format_args!("{:.5}", up.x));
            w.end_element();
            w.start_element("uy");
            w.write_text_fmt(format_args!("{:.5}", up.y));
            w.end_element();
            w.start_element("uz");
            w.write_text_fmt(format_args!("{:.5}", up.z));
            w.end_element();

            w.start_element("rx");
            w.write_text_fmt(format_args!("{:.5}", right.x));
            w.end_element();
            w.start_element("ry");
            w.write_text_fmt(format_args!("{:.5}", right.y));
            w.end_element();
            w.start_element("rz");
            w.write_text_fmt(format_args!("{:.5}", right.z));
            w.end_element();

            w.start_element("coord");
            w.write_text_fmt(format_args!("{:.5}", length_so_far / self.length()));
            w.end_element();

            w.start_element("strict");
            w.write_text("false");
            w.end_element();

            w.end_element();

            length_so_far += (next_point.0 - point.0).length();
        }
        w.end_document()
    }
}