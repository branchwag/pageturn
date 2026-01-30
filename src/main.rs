use eframe::egui;
use eframe::wasm_bindgen::JsCast;
use egui::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Vec2};
use std::f32::consts::PI;

fn main() {
    // Redirect panic messages to console.error
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let web_options = eframe::WebOptions::default();

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    setup_custom_fonts(&cc.egui_ctx);
                    Ok(Box::new(JournalApp::new()))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}

#[derive(Clone)]
struct BlogPost {
    title: String,
    content: String,
    date: String,
}

struct JournalApp {
    posts: Vec<BlogPost>,
    current_page: usize,
    page_turn_progress: f32,
    is_turning: bool,
    turn_direction: i32, // -1 for backward, 1 for forward
}

impl Default for JournalApp {
    fn default() -> Self {
        Self::new()
    }
}

impl JournalApp {
    fn new() -> Self {
        Self {
            posts: vec![
                BlogPost {
                    title: "Welcome...".to_string(),
                    content: "This is the first entry in my journal. The beginning of a new chapter.".to_string(),
                    date: "January 2026".to_string(),
                },
                BlogPost {
                    title: "Day 2".to_string(),
                    content: "Another day, another page. The journey continues with new discoveries and insights.".to_string(),
                    date: "January 2026".to_string(),
                },
                BlogPost {
                    title: "Day 3".to_string(),
                    content: "Reflections on the path so far. Sometimes the smallest moments hold the greatest meaning.".to_string(),
                    date: "January 2026".to_string(),
                },
                BlogPost {
                    title: "Day 4".to_string(),
                    content: "Looking forward to what comes next. Each page turn reveals new possibilities.".to_string(),
                    date: "January 2026".to_string(),
                },
            ],
            current_page: 0,
            page_turn_progress: 0.0,
            is_turning: false,
            turn_direction: 0,
        }
    }

    fn draw_parchment_page(&self, ui: &mut egui::Ui, rect: Rect, post: &BlogPost, is_left: bool) {
        let painter = ui.painter();

        // Parchment color with subtle gradient
        let parchment_base = Color32::from_rgb(245, 235, 210);

        // Draw the page background
        painter.rect_filled(rect, Rounding::same(2.0), parchment_base);

        // Add subtle texture by drawing semi-transparent lines
        let texture_color = Color32::from_rgba_premultiplied(210, 200, 175, 15);
        for i in 0..20 {
            let y = rect.min.y + (rect.height() * i as f32 / 20.0);
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(0.5, texture_color),
            );
        }

        // Draw edge shadow
        if is_left {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.max.x - 20.0, rect.min.y),
                    Vec2::new(20.0, rect.height()),
                ),
                Rounding::ZERO,
                Color32::from_rgba_premultiplied(0, 0, 0, 10),
            );
        } else {
            painter.rect_filled(
                Rect::from_min_size(rect.min, Vec2::new(20.0, rect.height())),
                Rounding::ZERO,
                Color32::from_rgba_premultiplied(0, 0, 0, 10),
            );
        }

        // Draw the content
        let text_rect = rect.shrink(40.0);
        let mut text_pos = text_rect.min;

        // Title
        let title_font = FontId::proportional(28.0);
        let title_color = Color32::from_rgb(60, 50, 40);

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            &post.title,
            title_font,
            title_color,
        );

        text_pos.y += 50.0;

        // Date
        let date_font = FontId::proportional(14.0);
        let date_color = Color32::from_rgb(100, 90, 80);

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            &post.date,
            date_font,
            date_color,
        );

        text_pos.y += 40.0;

        // Content - word wrap manually
        let content_font = FontId::proportional(18.0);
        let content_color = Color32::from_rgb(40, 35, 30);
        let max_width = text_rect.width();

        let words: Vec<&str> = post.content.split_whitespace().collect();
        let mut current_line = String::new();
        let line_height = 28.0;

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let galley =
                painter.layout_no_wrap(test_line.clone(), content_font.clone(), content_color);

            if galley.rect.width() > max_width {
                if !current_line.is_empty() {
                    painter.text(
                        text_pos,
                        egui::Align2::LEFT_TOP,
                        &current_line,
                        content_font.clone(),
                        content_color,
                    );
                    text_pos.y += line_height;
                }
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }

        if !current_line.is_empty() {
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                &current_line,
                content_font,
                content_color,
            );
        }
    }

    fn draw_curling_page(
        &self,
        ui: &mut egui::Ui,
        rect: Rect,
        post: &BlogPost,
        progress: f32,
        turning_right: bool,
    ) {
        // Use easing function for more natural motion
        let eased_progress = self.ease_in_out_cubic(progress);

        // Calculate curl angle (0 to PI)
        let curl_angle = eased_progress * PI;

        // The page curls from the edge, creating a cylinder effect
        let page_width = rect.width();
        let curl_radius = page_width * 0.3; // Radius of the curl cylinder

        // How far the curl has traveled across the page
        let curl_position = if turning_right {
            rect.max.x - (page_width * eased_progress)
        } else {
            rect.min.x + (page_width * eased_progress)
        };

        // Draw the flat part of the page (not yet curled)
        let flat_width = if turning_right {
            (curl_position - rect.min.x).max(0.0)
        } else {
            (rect.max.x - curl_position).max(0.0)
        };

        if flat_width > 1.0 {
            let flat_rect = if turning_right {
                Rect::from_min_size(rect.min, Vec2::new(flat_width, rect.height()))
            } else {
                Rect::from_min_size(
                    Pos2::new(curl_position, rect.min.y),
                    Vec2::new(flat_width, rect.height()),
                )
            };

            // Draw flat part with content
            self.draw_parchment_page(ui, flat_rect, post, !turning_right);
        }

        // Now get painter for the curling parts
        let painter = ui.painter();

        // Draw the curling part
        if curl_angle > 0.01 && curl_angle < PI - 0.01 {
            // Front face of curl (what we see of the original page)
            let front_width = curl_radius * curl_angle.sin();

            if front_width > 1.0 {
                let front_rect = if turning_right {
                    Rect::from_min_size(
                        Pos2::new(curl_position - front_width, rect.min.y),
                        Vec2::new(front_width, rect.height()),
                    )
                } else {
                    Rect::from_min_size(
                        Pos2::new(curl_position, rect.min.y),
                        Vec2::new(front_width, rect.height()),
                    )
                };

                // Calculate lighting based on angle (front face gets darker as it curls)
                let lighting = (1.0 - curl_angle / PI) * 0.7 + 0.3;
                let parchment_color = Color32::from_rgb(
                    (245.0 * lighting) as u8,
                    (235.0 * lighting) as u8,
                    (210.0 * lighting) as u8,
                );

                painter.rect_filled(front_rect, Rounding::same(2.0), parchment_color);

                // Add gradient for depth
                let gradient_width = 20.0;
                let shadow_rect = if turning_right {
                    Rect::from_min_size(
                        Pos2::new(front_rect.min.x, front_rect.min.y),
                        Vec2::new(gradient_width, front_rect.height()),
                    )
                } else {
                    Rect::from_min_size(
                        Pos2::new(front_rect.max.x - gradient_width, front_rect.min.y),
                        Vec2::new(gradient_width, front_rect.height()),
                    )
                };

                let shadow_alpha = (curl_angle.sin() * 60.0) as u8;
                painter.rect_filled(
                    shadow_rect,
                    Rounding::ZERO,
                    Color32::from_rgba_premultiplied(0, 0, 0, shadow_alpha),
                );
            }

            // Back face of curl (reverse side of the page)
            let back_width = curl_radius * (PI - curl_angle).sin();

            if back_width > 1.0 {
                let back_rect = if turning_right {
                    Rect::from_min_size(
                        Pos2::new(curl_position, rect.min.y),
                        Vec2::new(back_width, rect.height()),
                    )
                } else {
                    Rect::from_min_size(
                        Pos2::new(curl_position - back_width, rect.min.y),
                        Vec2::new(back_width, rect.height()),
                    )
                };

                // Back of page is lighter/different color
                let back_lighting = curl_angle.sin() * 0.6 + 0.4;
                let back_color = Color32::from_rgb(
                    (235.0 * back_lighting) as u8,
                    (225.0 * back_lighting) as u8,
                    (200.0 * back_lighting) as u8,
                );

                painter.rect_filled(back_rect, Rounding::same(2.0), back_color);

                // Add highlight on the back edge
                let highlight_width = 15.0;
                let highlight_rect = if turning_right {
                    Rect::from_min_size(
                        Pos2::new(back_rect.max.x - highlight_width, back_rect.min.y),
                        Vec2::new(highlight_width, back_rect.height()),
                    )
                } else {
                    Rect::from_min_size(
                        back_rect.min,
                        Vec2::new(highlight_width, back_rect.height()),
                    )
                };

                let highlight_alpha = ((PI - curl_angle).sin() * 40.0) as u8;
                painter.rect_filled(
                    highlight_rect,
                    Rounding::ZERO,
                    Color32::from_rgba_premultiplied(255, 255, 255, highlight_alpha),
                );
            }
        }

        // Draw shadow cast by the curling page onto the page behind it
        let shadow_width = (curl_radius * 0.8 * eased_progress).min(80.0);
        if shadow_width > 1.0 {
            let shadow_rect = if turning_right {
                Rect::from_min_size(
                    Pos2::new(curl_position, rect.min.y),
                    Vec2::new(shadow_width, rect.height()),
                )
            } else {
                Rect::from_min_size(
                    Pos2::new(curl_position - shadow_width, rect.min.y),
                    Vec2::new(shadow_width, rect.height()),
                )
            };

            let shadow_alpha = (eased_progress * 40.0) as u8;
            painter.rect_filled(
                shadow_rect,
                Rounding::ZERO,
                Color32::from_rgba_premultiplied(0, 0, 0, shadow_alpha),
            );
        }
    }

    // Easing function for smoother animation
    fn ease_in_out_cubic(&self, t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
}

impl eframe::App for JournalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Animation loop
        if self.is_turning {
            let delta = 0.03; // Animation speed
            self.page_turn_progress += delta;

            if self.page_turn_progress >= 1.0 {
                self.page_turn_progress = 0.0;
                self.is_turning = false;

                if self.turn_direction > 0 {
                    self.current_page = (self.current_page + 2).min(self.posts.len() - 1);
                } else {
                    self.current_page = self.current_page.saturating_sub(2);
                }
            }

            ctx.request_repaint();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(80, 70, 60)))
            .show(ctx, |ui| {
                let available_size = ui.available_size();
                let journal_width = available_size.x.min(1000.0);
                let journal_height = available_size.y.min(700.0);

                let center_x = available_size.x / 2.0;
                let center_y = available_size.y / 2.0;

                let journal_rect = Rect::from_center_size(
                    Pos2::new(center_x, center_y),
                    Vec2::new(journal_width, journal_height),
                );

                // Split into left and right pages
                let page_width = journal_width / 2.0;
                let left_rect =
                    Rect::from_min_size(journal_rect.min, Vec2::new(page_width, journal_height));
                let right_rect = Rect::from_min_size(
                    Pos2::new(journal_rect.min.x + page_width, journal_rect.min.y),
                    Vec2::new(page_width, journal_height),
                );

                // Draw the journal spine
                let spine_rect = Rect::from_center_size(
                    Pos2::new(center_x, center_y),
                    Vec2::new(8.0, journal_height),
                );
                ui.painter().rect_filled(
                    spine_rect,
                    Rounding::ZERO,
                    Color32::from_rgb(100, 80, 60),
                );

                // Draw pages based on animation state
                if !self.is_turning {
                    // Static view
                    if self.current_page == 0 {
                        // First page alone on right
                        ui.painter().rect_filled(
                            left_rect,
                            Rounding::same(2.0),
                            Color32::from_rgb(245, 235, 210),
                        );
                        self.draw_parchment_page(ui, right_rect, &self.posts[0], false);
                    } else {
                        // Show spread
                        self.draw_parchment_page(
                            ui,
                            left_rect,
                            &self.posts[self.current_page - 1],
                            true,
                        );
                        if self.current_page < self.posts.len() {
                            self.draw_parchment_page(
                                ui,
                                right_rect,
                                &self.posts[self.current_page],
                                false,
                            );
                        } else {
                            ui.painter().rect_filled(
                                right_rect,
                                Rounding::same(2.0),
                                Color32::from_rgb(245, 235, 210),
                            );
                        }
                    }
                } else {
                    // Animation in progress
                    if self.turn_direction > 0 {
                        // Turning forward
                        // Draw the page that will be visible on the left after turn
                        if self.current_page == 0 {
                            if self.posts.len() > 1 {
                                self.draw_parchment_page(ui, left_rect, &self.posts[1], true);
                            }
                        } else if self.current_page + 1 < self.posts.len() {
                            self.draw_parchment_page(
                                ui,
                                left_rect,
                                &self.posts[self.current_page + 1],
                                true,
                            );
                        } else {
                            ui.painter().rect_filled(
                                left_rect,
                                Rounding::same(2.0),
                                Color32::from_rgb(245, 235, 210),
                            );
                        }

                        // Draw the page being revealed underneath on the right
                        if self.current_page + 2 < self.posts.len() {
                            self.draw_parchment_page(
                                ui,
                                right_rect,
                                &self.posts[self.current_page + 2],
                                false,
                            );
                        } else {
                            ui.painter().rect_filled(
                                right_rect,
                                Rounding::same(2.0),
                                Color32::from_rgb(245, 235, 210),
                            );
                        }

                        // Draw the curling right page on top
                        if self.current_page < self.posts.len() {
                            self.draw_curling_page(
                                ui,
                                right_rect,
                                &self.posts[self.current_page],
                                self.page_turn_progress,
                                true,
                            );
                        }
                    } else {
                        // Turning backward
                        // Draw what will be on the left after turning back
                        if self.current_page == 2 {
                            ui.painter().rect_filled(
                                left_rect,
                                Rounding::same(2.0),
                                Color32::from_rgb(245, 235, 210),
                            );
                        } else if self.current_page > 2 {
                            self.draw_parchment_page(
                                ui,
                                left_rect,
                                &self.posts[self.current_page - 3],
                                true,
                            );
                        }

                        // Draw the page being revealed on the right
                        if self.current_page > 1 {
                            self.draw_parchment_page(
                                ui,
                                right_rect,
                                &self.posts[self.current_page - 2],
                                false,
                            );

                            // Draw the curling page
                            self.draw_curling_page(
                                ui,
                                right_rect,
                                &self.posts[self.current_page - 1],
                                self.page_turn_progress,
                                false,
                            );
                        }
                    }
                }

                // Navigation buttons
                let button_y = journal_rect.max.y + 20.0;

                let prev_button_pos = Pos2::new(journal_rect.min.x + 50.0, button_y);
                let prev_button_rect =
                    Rect::from_center_size(prev_button_pos, Vec2::new(80.0, 35.0));

                let next_button_pos = Pos2::new(journal_rect.max.x - 50.0, button_y);
                let next_button_rect =
                    Rect::from_center_size(next_button_pos, Vec2::new(80.0, 35.0));

                // Previous button
                if self.current_page > 0 && !self.is_turning {
                    let prev_response = ui.allocate_rect(prev_button_rect, egui::Sense::click());

                    let button_color = if prev_response.hovered() {
                        Color32::from_rgb(200, 180, 150)
                    } else {
                        Color32::from_rgb(220, 200, 170)
                    };

                    ui.painter()
                        .rect_filled(prev_button_rect, Rounding::same(5.0), button_color);
                    ui.painter().text(
                        prev_button_pos,
                        egui::Align2::CENTER_CENTER,
                        "<- Previous",
                        FontId::proportional(14.0),
                        Color32::from_rgb(60, 50, 40),
                    );

                    if prev_response.clicked() {
                        self.is_turning = true;
                        self.turn_direction = -1;
                        self.page_turn_progress = 0.0;
                    }
                }

                // Next button
                if self.current_page + 2 < self.posts.len() && !self.is_turning {
                    let next_response = ui.allocate_rect(next_button_rect, egui::Sense::click());

                    let button_color = if next_response.hovered() {
                        Color32::from_rgb(200, 180, 150)
                    } else {
                        Color32::from_rgb(220, 200, 170)
                    };

                    ui.painter()
                        .rect_filled(next_button_rect, Rounding::same(5.0), button_color);
                    ui.painter().text(
                        next_button_pos,
                        egui::Align2::CENTER_CENTER,
                        "Next ->",
                        FontId::proportional(14.0),
                        Color32::from_rgb(60, 50, 40),
                    );

                    if next_response.clicked() {
                        self.is_turning = true;
                        self.turn_direction = 1;
                        self.page_turn_progress = 0.0;
                    }
                }

                // Page counter
                let counter_text = if self.current_page == 0 {
                    format!("Page 1 of {}", self.posts.len())
                } else {
                    format!(
                        "Pages {}-{} of {}",
                        self.current_page,
                        (self.current_page + 1).min(self.posts.len()),
                        self.posts.len()
                    )
                };
                ui.painter().text(
                    Pos2::new(center_x, button_y),
                    egui::Align2::CENTER_CENTER,
                    counter_text,
                    FontId::proportional(14.0),
                    Color32::from_rgb(220, 210, 190),
                );
            });
    }
}
