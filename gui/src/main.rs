// #![warn(clippy::all, rust_2018_idioms)]
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::Path;

use curve_core::{
    fvd,
    transitions::{TransitionFunction, Transitions},
};
use egui::{
    plot::{Legend, Line, Plot, PlotPoints, VLine},
    Color32, SidePanel, TopBottomPanel,
};
use glam::{vec3, Mat4, Vec3};
use three_d::{
    Axes, BoundingBox, Camera, ClearState, ColorMaterial, CpuMaterial, CpuMesh, CpuTexture,
    DirectionalLight, FrameOutput, Geometry, Gm, Mesh, OrbitControl, PhysicalMaterial, Srgba,
    Texture2D, Window, WindowSettings,
};

fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        three_d_vec(vec3(5.0, 2.0, 2.5)),
        three_d_vec(vec3(0.0, 0.0, 0.)),
        three_d_vec(vec3(0.0, 1.0, 0.0)),
        three_d_angle(45f32.to_radians()),
        0.1,
        1000.0,
    );
    let mut gui = three_d::GUI::new(&context);
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    sphere.set_transformation(three_d_mat4(
        Mat4::from_translation(vec3(0.0, 1.3, 0.0)) * Mat4::from_scale(Vec3::ONE * 0.2),
    ));

    let mut floor_mesh = CpuMesh::square();
    floor_mesh.uvs = Some(
        floor_mesh
            .uvs
            .unwrap()
            .iter()
            .map(|v| v * 100.0)
            .collect::<Vec<_>>(),
    );

    let mut floor_plane = Gm::new(
        Mesh::new(&context, &floor_mesh),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo_texture: Some(three_d_image(
                    &context,
                    Path::new("/Users/eli/Developer/curve/gui/texture_01.png"),
                )),
                ..Default::default()
            },
        ),
    );
    floor_plane.set_transformation(three_d_mat4(
        Mat4::from_scale(Vec3::ONE * 100.) * Mat4::from_axis_angle(Vec3::X, 90f32.to_radians()),
    ));

    let axes = Axes::new(&context, 0.02, 1.0);

    let light0 = DirectionalLight::new(
        &context,
        1.0,
        Srgba::WHITE,
        &three_d_vec(vec3(0.0, -0.5, -0.5)),
    );

    // app data

    let mut transitions = Transitions::new(1., 0., 0.);
    let mut spline = fvd::create_spline(&transitions, Vec3::Y, 5.);
    let mut transition_idx = 0;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom_panel")
                    .default_height(400.0)
                    .resizable(true)
                    .show(gui_context, |ui| {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::Center)
                                .with_cross_justify(true),
                            |ui| {
                                ui.vertical(|ui| {
                                    ui.heading("Section Editor");
                                    {
                                        let mut transition =
                                            transitions.transitions[transition_idx];
                                        ui.horizontal(|ui| {
                                            ui.label("Length");
                                            ui.add(
                                                egui::DragValue::new(&mut transition.length)
                                                    .clamp_range(0.1f32..=f32::INFINITY)
                                                    .suffix("s")
                                                    .speed(0.01)
                                                    .fixed_decimals(1), // .update_while_editing(true) TODO patch this into egui 0.22
                                            );
                                            transition.length =
                                                (transition.length * 10.).round() / 10.0;
                                        });
                                        ui.horizontal(|ui| {
                                            let mut fixed_speed = transition.speed.is_some();
                                            ui.checkbox(&mut fixed_speed, "Fixed Speed");
                                            if fixed_speed && transition.speed.is_none() {
                                                transition.speed = Some(5.);
                                            }
                                            if !fixed_speed {
                                                transition.speed = None;
                                            }
                                            if let Some(fixed_speed) = &mut transition.speed {
                                                ui.add(
                                                    egui::Slider::new(fixed_speed, 1f32..=500f32)
                                                        .logarithmic(true)
                                                        .fixed_decimals(0)
                                                        .suffix("m/s"),
                                                );
                                            }
                                        });

                                        transitions.transitions[transition_idx] = transition;
                                    }
                                    {
                                        let mut vert_transition =
                                            transitions.transitions[transition_idx].vert;
                                        ui.vertical(|ui| {
                                            ui.heading("Normal");
                                            ui.horizontal(|ui| {
                                                ui.label("Type");
                                                egui::ComboBox::from_id_source(
                                                    "type_combobox_normal",
                                                )
                                                .selected_text(match vert_transition.function {
                                                    TransitionFunction::Linear => "Linear",
                                                    TransitionFunction::Quadratic => "Quadratic",
                                                    TransitionFunction::Cubic => "Cubic",
                                                    TransitionFunction::Plateau => "Plateau",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut vert_transition.function,
                                                        TransitionFunction::Linear,
                                                        "Linear",
                                                    );
                                                    ui.selectable_value(
                                                        &mut vert_transition.function,
                                                        TransitionFunction::Quadratic,
                                                        "Quadratic",
                                                    );
                                                    ui.selectable_value(
                                                        &mut vert_transition.function,
                                                        TransitionFunction::Cubic,
                                                        "Cubic",
                                                    );
                                                    ui.selectable_value(
                                                        &mut vert_transition.function,
                                                        TransitionFunction::Plateau,
                                                        "Plateau",
                                                    );
                                                });
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(
                                                            &mut vert_transition.change,
                                                        )
                                                        .clamp_range(-10f32..=10f32)
                                                        .suffix("g")
                                                        .speed(0.1)
                                                        .fixed_decimals(1), // .update_while_editing(true),
                                                    )
                                                    .secondary_clicked()
                                                {
                                                    vert_transition.change = 0.;
                                                };
                                                vert_transition.change =
                                                    (vert_transition.change * 10.).round() / 10.0;
                                            });
                                        });
                                        transitions.transitions[transition_idx].vert =
                                            vert_transition;
                                    }
                                    {
                                        let mut lat_transition =
                                            transitions.transitions[transition_idx].lat;
                                        ui.vertical(|ui| {
                                            ui.heading("Lateral");
                                            ui.horizontal(|ui| {
                                                ui.label("Type");
                                                egui::ComboBox::from_id_source("type_combobox_lat")
                                                    .selected_text(match lat_transition.function {
                                                        TransitionFunction::Linear => "Linear",
                                                        TransitionFunction::Quadratic => {
                                                            "Quadratic"
                                                        }
                                                        TransitionFunction::Cubic => "Cubic",
                                                        TransitionFunction::Plateau => "Plateau",
                                                    })
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut lat_transition.function,
                                                            TransitionFunction::Linear,
                                                            "Linear",
                                                        );
                                                        ui.selectable_value(
                                                            &mut lat_transition.function,
                                                            TransitionFunction::Quadratic,
                                                            "Quadratic",
                                                        );
                                                        ui.selectable_value(
                                                            &mut lat_transition.function,
                                                            TransitionFunction::Cubic,
                                                            "Cubic",
                                                        );
                                                        ui.selectable_value(
                                                            &mut lat_transition.function,
                                                            TransitionFunction::Plateau,
                                                            "Plateau",
                                                        );
                                                    });
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(
                                                            &mut lat_transition.change,
                                                        )
                                                        .clamp_range(-10f32..=10f32)
                                                        .suffix("g")
                                                        .speed(0.1)
                                                        .fixed_decimals(1), // .update_while_editing(true),
                                                    )
                                                    .secondary_clicked()
                                                {
                                                    lat_transition.change = 0.;
                                                };
                                                lat_transition.change =
                                                    (lat_transition.change * 10.).round() / 10.0;
                                            });
                                        });
                                        transitions.transitions[transition_idx].lat =
                                            lat_transition;
                                    }
                                    {
                                        let mut roll_transition =
                                            transitions.transitions[transition_idx].roll;
                                        ui.vertical(|ui| {
                                            ui.heading("Roll");
                                            ui.horizontal(|ui| {
                                                ui.label("Type");
                                                egui::ComboBox::from_id_source(
                                                    "type_combobox_roll",
                                                )
                                                .selected_text(match roll_transition.function {
                                                    TransitionFunction::Linear => "Linear",
                                                    TransitionFunction::Quadratic => "Quadratic",
                                                    TransitionFunction::Cubic => "Cubic",
                                                    TransitionFunction::Plateau => "Plateau",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut roll_transition.function,
                                                        TransitionFunction::Linear,
                                                        "Linear",
                                                    );
                                                    ui.selectable_value(
                                                        &mut roll_transition.function,
                                                        TransitionFunction::Quadratic,
                                                        "Quadratic",
                                                    );
                                                    ui.selectable_value(
                                                        &mut roll_transition.function,
                                                        TransitionFunction::Cubic,
                                                        "Cubic",
                                                    );
                                                    ui.selectable_value(
                                                        &mut roll_transition.function,
                                                        TransitionFunction::Plateau,
                                                        "Plateau",
                                                    );
                                                });
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(
                                                            &mut roll_transition.change,
                                                        )
                                                        .clamp_range(-1000f32..=1000f32)
                                                        .suffix("°/s")
                                                        .speed(1.)
                                                        .fixed_decimals(1), // .update_while_editing(true),
                                                    )
                                                    .secondary_clicked()
                                                {
                                                    roll_transition.change = 0.
                                                };
                                                roll_transition.change =
                                                    (roll_transition.change * 10.).round() / 10.0;
                                            });
                                        });
                                        transitions.transitions[transition_idx].roll =
                                            roll_transition;
                                    }
                                });
                                let plot = Plot::new("Transitions").allow_scroll(false);
                                let vert = (0..=(transitions.length() * 50.) as usize).map(|v| {
                                    [
                                        v as f64 / 50.0,
                                        (transitions
                                            .interpolate(v as f32 / 50.)
                                            .map(|v| v.0)
                                            .unwrap_or(f32::INFINITY)
                                            as f64),
                                    ]
                                });
                                let lat = (0..=(transitions.length() * 50.) as usize).map(|v| {
                                    [
                                        v as f64 / 50.0,
                                        transitions
                                            .interpolate(v as f32 / 50.)
                                            .map(|v| v.1)
                                            .unwrap_or(f32::INFINITY)
                                            as f64,
                                    ]
                                });
                                let roll = (0..=(transitions.length() * 50.) as usize).map(|v| {
                                    [
                                        v as f64 / 50.0,
                                        (transitions
                                            .interpolate(v as f32 / 50.)
                                            .map(|v| v.2)
                                            .unwrap_or(f32::INFINITY)
                                            as f64)
                                            .to_radians(),
                                    ]
                                });

                                plot
                                    // .custom_y_axes(vec![
                                    //     AxisHints::default().formatter(|v, _, _| {
                                    //         format!("{}g", (v * 100_000_000.0).round() / 100_000_000.0)
                                    //     }),
                                    //     AxisHints::default()
                                    //         .placement(HPlacement::Right)
                                    //         .formatter(|v, _, _| format!("{:.1}°", v.to_degrees())),
                                    // ])
                                    .legend(Legend::default())
                                    .y_axis_formatter(|v, _| {
                                        format!("{}g", (v * 100_000_000.0).round() / 100_000_000.0)
                                    })
                                    .show(ui, |plot_ui| {
                                        plot_ui.line(
                                            Line::new(PlotPoints::from_iter(vert))
                                                .name("vertical")
                                                .color(Color32::BLUE),
                                        );
                                        plot_ui.line(
                                            Line::new(PlotPoints::from_iter(lat))
                                                .name("lateral")
                                                .color(Color32::GREEN),
                                        );
                                        plot_ui.line(
                                            Line::new(PlotPoints::from_iter(roll))
                                                .name("roll")
                                                .color(Color32::RED),
                                        );
                                    });
                            },
                        );
                    });
            },
        );
        control.handle_events(&mut camera, &mut frame_input.events);

        let screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(1., 1., 1., 1., 1.));

        screen.render(
            &camera,
            axes.into_iter().chain(&sphere).chain(&floor_plane),
            &[&light0],
        );

        screen.write(|| gui.render());

        FrameOutput::default()
    });
}

fn three_d_angle(radians: f32) -> three_d::Radians {
    three_d::Rad(radians)
}

fn three_d_vec(vec: Vec3) -> three_d::Vec3 {
    three_d::Vector3::new(vec.x, vec.y, vec.z)
}

fn three_d_mat4(mat: Mat4) -> three_d::Mat4 {
    three_d::Mat4::new(
        mat.x_axis.x,
        mat.x_axis.y,
        mat.x_axis.z,
        mat.x_axis.w,
        mat.y_axis.x,
        mat.y_axis.y,
        mat.y_axis.z,
        mat.y_axis.w,
        mat.z_axis.x,
        mat.z_axis.y,
        mat.z_axis.z,
        mat.z_axis.w,
        mat.w_axis.x,
        mat.w_axis.y,
        mat.w_axis.z,
        mat.w_axis.w,
    )
}

fn three_d_image(context: &three_d::Context, path: &Path) -> CpuTexture {
    let image = image::open(path).unwrap().to_rgba8();
    let data = image.pixels().map(|v| v.0).collect::<Vec<_>>();

    CpuTexture {
        name: "image".to_string(),
        data: three_d::TextureData::RgbaU8(data),
        width: image.width(),
        height: image.height(),
        min_filter: three_d::Interpolation::Linear,
        mag_filter: three_d::Interpolation::Linear,
        mip_map_filter: Some(three_d::Interpolation::Linear),
        wrap_s: three_d::Wrapping::Repeat,
        wrap_t: three_d::Wrapping::Repeat,
    }
}
