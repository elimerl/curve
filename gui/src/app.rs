use curve_core::fvd;
use curve_core::spline::TrackSpline;
use eframe::egui_glow;
use eframe::glow;

use egui::mutex::Mutex;
use egui::Color32;
use egui::Modifiers;
use egui::Pos2;
use egui::Rect;
use egui::Stroke;
use egui::Vec2b;
use egui_plot::AxisHints;
use egui_plot::HPlacement;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotPoints;
use glam::Mat4;
use glam::Quat;
use glam::Vec3;
use std::fmt::format;
use std::sync::Arc;

use curve_core::transitions::{Transition, TransitionFunction, Transitions};

pub struct TemplateApp {
    rotating_triangle: Arc<Mutex<RotatingTriangle>>,

    transitions: Transitions,
    transition_idx: usize,

    spline: TrackSpline,
    cam_pos: f32,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");

        let transitions = Transitions::new(1., 0., 0.);

        let spline = fvd::create_spline(&transitions, Vec3::Y, 5.);
        Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl, &spline))),
            spline,
            transitions,
            transition_idx: 0,
            cam_pos: 0.,
        }
    }
}

impl eframe::App for TemplateApp {
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });
        egui::Window::new("Section Editor").show(ctx, |ui| {
            // for (i, transition) in self.transitions.transitions.iter().enumerate() {
            //     if egui::Button::new("section")
            //         .show(ui, |ui| {
            //             ui.label(format!("Section {}", i));
            //         })
            //         .response
            //         .clicked()
            //     {
            //         dbg!(i);
            //     }
            // }
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Section Editor");
                        {
                            let mut transition = self.transitions.transitions[self.transition_idx];
                            ui.horizontal(|ui| {
                                ui.label("Length");
                                ui.add(
                                    egui::DragValue::new(&mut transition.length)
                                        .clamp_range(0.1f32..=f32::INFINITY)
                                        .suffix("s")
                                        .speed(0.01)
                                        .fixed_decimals(1)
                                        .update_while_editing(true),
                                );
                                transition.length = (transition.length * 10.).round() / 10.0;
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

                            self.transitions.transitions[self.transition_idx] = transition;
                        }
                        {
                            let mut vert_transition =
                                self.transitions.transitions[self.transition_idx].vert;
                            ui.vertical(|ui| {
                                ui.heading("Normal");
                                ui.horizontal(|ui| {
                                    ui.label("Type");
                                    egui::ComboBox::from_id_source("type_combobox_normal")
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
                                            egui::DragValue::new(&mut vert_transition.change)
                                                .clamp_range(-10f32..=10f32)
                                                .suffix("g")
                                                .speed(0.1)
                                                .fixed_decimals(1)
                                                .update_while_editing(true),
                                        )
                                        .secondary_clicked()
                                    {
                                        vert_transition.change = 0.;
                                    };
                                    vert_transition.change =
                                        (vert_transition.change * 10.).round() / 10.0;
                                });
                            });
                            self.transitions.transitions[self.transition_idx].vert =
                                vert_transition;
                        }
                        {
                            let mut lat_transition =
                                self.transitions.transitions[self.transition_idx].lat;
                            ui.vertical(|ui| {
                                ui.heading("Lateral");
                                ui.horizontal(|ui| {
                                    ui.label("Type");
                                    egui::ComboBox::from_id_source("type_combobox_lat")
                                        .selected_text(match lat_transition.function {
                                            TransitionFunction::Linear => "Linear",
                                            TransitionFunction::Quadratic => "Quadratic",
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
                                            egui::DragValue::new(&mut lat_transition.change)
                                                .clamp_range(-10f32..=10f32)
                                                .suffix("g")
                                                .speed(0.1)
                                                .fixed_decimals(1)
                                                .update_while_editing(true),
                                        )
                                        .secondary_clicked()
                                    {
                                        lat_transition.change = 0.;
                                    };
                                    lat_transition.change =
                                        (lat_transition.change * 10.).round() / 10.0;
                                });
                            });
                            self.transitions.transitions[self.transition_idx].lat = lat_transition;
                        }
                        {
                            let mut roll_transition =
                                self.transitions.transitions[self.transition_idx].roll;
                            ui.vertical(|ui| {
                                ui.heading("Roll");
                                ui.horizontal(|ui| {
                                    ui.label("Type");
                                    egui::ComboBox::from_id_source("type_combobox_roll")
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
                                            egui::DragValue::new(&mut roll_transition.change)
                                                .clamp_range(-1000f32..=1000f32)
                                                .suffix("°/s")
                                                .speed(1.)
                                                .fixed_decimals(1)
                                                .update_while_editing(true),
                                        )
                                        .secondary_clicked()
                                    {
                                        roll_transition.change = 0.
                                    };
                                    roll_transition.change =
                                        (roll_transition.change * 10.).round() / 10.0;
                                });
                            });
                            self.transitions.transitions[self.transition_idx].roll =
                                roll_transition;
                        }
                    });
                },
            );
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::default().fill(Color32::WHITE).show(ui, |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(300.), egui::Sense::hover());

                // Clone locals so we can move them into the paint callback:
                let rotating_triangle = self.rotating_triangle.clone();
                let spline = self.spline.clone();
                let cam_pos = self.cam_pos;
                let callback = egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            let spline = &spline;
                            rotating_triangle
                                .lock()
                                .paint(painter.gl(), spline, cam_pos);
                        },
                    )),
                };
                ui.painter().add(callback);

                // let pos = rect.left_top();

                // // self.custom_painting(ui);
                // // let view = Mat4::look_at_rh(Vec3::ONE * 5., Vec3::ZERO, Vec3::Y);
                // let offscreen = |p: Vec3| {
                //     ((p.x <= -1. || p.x >= 1.) || (p.y <= -1. || p.y >= 1.)) || (p.z <= 0.)
                // };

                // let project_point = |p: Vec3, obj: Mat4| {
                //     let p = (proj * view * obj).project_point3(p);
                //     (
                //         egui::Vec2::new(
                //             0.5 * (p.x + 1.) * rect.width(),
                //             0.5 * (-p.y + 1.) * rect.height(),
                //         ),
                //         p,
                //     )
                // };
                // let draw_line = |p1: Vec3, p2: Vec3, obj: Mat4, color: Color32| {
                //     let proj_a = project_point(p1, obj);
                //     let proj_b = project_point(p2, obj);

                //     if offscreen(proj_a.1) && offscreen(proj_b.1) {
                //         return;
                //     }

                //     ui.painter()
                //         .with_clip_rect(rect)
                //         .line_segment([pos + proj_a.0, pos + proj_b.0], Stroke::new(2., color));
                // };

                // draw_line(Vec3::ZERO, Vec3::X, Mat4::IDENTITY, Color32::RED);
                // draw_line(Vec3::ZERO, Vec3::Y, Mat4::IDENTITY, Color32::GREEN);
                // draw_line(Vec3::ZERO, Vec3::Z, Mat4::IDENTITY, Color32::BLUE);

                // for x in -10..10 {
                //     draw_line(
                //         Vec3::new(x as f32, -10., -10.),
                //         Vec3::new(x as f32, -10., 10.),
                //         Mat4::IDENTITY,
                //         Color32::GRAY,
                //     );
                //     draw_line(
                //         Vec3::new(-10., -10., x as f32),
                //         Vec3::new(10., -10., x as f32),
                //         Mat4::IDENTITY,
                //         Color32::GRAY,
                //     );
                // }

                // for points in self.spline.points.windows(2) {
                //     let a = points[0];
                //     let b = points[1];
                //     draw_line(a.0, b.0, Mat4::IDENTITY, Color32::BLACK);
                //     let heartline_offset_a = a.1 * Vec3::Y * -1.1; // FIXME change to const
                //     let rail_offset_a = a.1 * Vec3::X * 1.5 * 0.5; // FIXME change to const
                //     let heartline_offset_b = b.1 * Vec3::Y * -1.1; // FIXME change to const
                //     let rail_offset_b = b.1 * Vec3::X * 1.5 * 0.5; // FIXME change to const

                //     draw_line(
                //         a.0 + heartline_offset_a - rail_offset_a,
                //         b.0 + heartline_offset_b - rail_offset_b,
                //         Mat4::IDENTITY,
                //         Color32::BLACK,
                //     );

                //     draw_line(
                //         a.0 + heartline_offset_a + rail_offset_a,
                //         b.0 + heartline_offset_b + rail_offset_b,
                //         Mat4::IDENTITY,
                //         Color32::BLACK,
                //     );

                //     draw_line(
                //         a.0,
                //         a.0 + a.1 * Vec3::Y * 0.2,
                //         Mat4::IDENTITY,
                //         Color32::LIGHT_BLUE,
                //     );
                // }
            });
            let plot = Plot::new("Transitions").allow_scroll(false);
            let vert = (0..=(self.transitions.length() * 50.) as usize).map(|v| {
                [
                    v as f64 / 50.0,
                    (self
                        .transitions
                        .interpolate(v as f32 / 50.)
                        .map(|v| v.0)
                        .unwrap_or(f32::INFINITY) as f64),
                ]
            });
            let lat = (0..=(self.transitions.length() * 50.) as usize).map(|v| {
                [
                    v as f64 / 50.0,
                    self.transitions
                        .interpolate(v as f32 / 50.)
                        .map(|v| v.1)
                        .unwrap_or(f32::INFINITY) as f64,
                ]
            });
            let roll = (0..=(self.transitions.length() * 50.) as usize).map(|v| {
                [
                    v as f64 / 50.0,
                    (self
                        .transitions
                        .interpolate(v as f32 / 50.)
                        .map(|v| v.2)
                        .unwrap_or(f32::INFINITY) as f64)
                        .to_radians(),
                ]
            });

            plot.custom_y_axes(vec![
                AxisHints::default().formatter(|v, _, _| {
                    format!("{}g", (v * 100_000_000.0).round() / 100_000_000.0)
                }),
                AxisHints::default()
                    .placement(HPlacement::Right)
                    .formatter(|v, _, _| format!("{:.1}°", v.to_degrees())),
            ])
            .legend(Legend::default())
            .auto_bounds(Vec2b::new(true, false))
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
        });

        self.spline = fvd::create_spline(&self.transitions, Vec3::Y, 5.);

        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_down(egui::Key::W)) {
                self.cam_pos += 0.1;
            }
            if ctx.input(|i| i.key_down(egui::Key::S)) {
                self.cam_pos -= 0.1
            }
        }
    }
}

struct RotatingTriangle {
    program: glow::Program,
    vertex_array: glow::VertexArray,
    vbo: glow::Buffer,
    vertex_count: usize,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context, spline: &TrackSpline) -> Self {
        use glow::HasContext;

        unsafe {
            let shader_version = if cfg!(target_arch = "wasm32") {
                "#version 300 es"
            } else {
                "#version 330"
            };

            // We construct a buffer and upload the data
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let vertices: [f32; 9] = [
                0.5f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32,
            ];
            let vertices_u8 = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            );
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            // We now construct a vertex array to describe the format of the input buffer
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"in vec2 pos;
                out vec2 vert;
                void main() {
                    vert = pos;
                    gl_Position = vec4(pos - 0.5, 0.0, 1.0);
                }"#,
                r#"precision mediump float;
                in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 1.0, 1.0);
            }"#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.use_program(Some(program));
            Self {
                program,
                vertex_array: vao,
                vbo,
                vertex_count: 3,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext;
        unsafe {
            // gl cleanup
            // gl.delete_program(program);
            // gl.delete_vertex_array(vertex_array);
        }
    }

    fn update_spline_mesh(&mut self, gl: &glow::Context, spline: &TrackSpline) {
        use glow::HasContext;
        unsafe {
            let vertices: [f32; 9] = [
                0.5f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32,
            ];
            let vertices_u8 = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            );

            gl.bind_buffer(0, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            self.vertex_count = vertices.len() / 3;
        }
    }

    fn paint(&self, gl: &glow::Context, spline: &TrackSpline, cam_pos: f32) {
        use glow::HasContext;
        unsafe {
            let proj = Mat4::perspective_infinite_rh(120f32.to_radians(), 1., 0.1);
            let cam_point = spline.evaluate(cam_pos).unwrap();
            let view = Mat4::from_rotation_translation(cam_point.1, cam_point.0);

            gl.use_program(Some(self.program));
            // gl.uniform_matrix_4_f32_slice(
            //     gl.get_uniform_location(self.program, "transform").as_ref(),
            //     false,
            //     &Mat4::IDENTITY.to_cols_array(),
            // );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count as i32);
        }
    }
}
