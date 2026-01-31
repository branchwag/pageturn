use eframe::egui;
use eframe::wasm_bindgen::JsCast;
use egui::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Vec2};
mod post;
mod posts;

fn main() {
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

        // Hide the loading indicator
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.remove();
        }

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
    let mut fonts = egui::FontDefinitions::default();

    // Load handwriting font
    fonts.font_data.insert(
        "handwriting".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/Caveat-Regular.ttf"
        ))),
    );

    // Put handwriting font first for proportional text
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "handwriting".to_owned());

    ctx.set_fonts(fonts);
}

use post::BlogPost;

struct JournalApp {
    posts: Vec<BlogPost>,
    current_page: usize,
    page_turn_progress: f32,
    is_turning: bool,
    turn_direction: i32,
    drag_start: Option<Pos2>,
    #[allow(dead_code)]
    drag_current: Option<Pos2>,
    is_dragging: bool,
}

impl Default for JournalApp {
    fn default() -> Self {
        Self::new()
    }
}

impl JournalApp {
    fn new() -> Self {
        let posts = posts::load_posts();
        Self {
            posts,
            current_page: 0,
            page_turn_progress: 0.0,
            is_turning: false,
            turn_direction: 0,
            drag_start: None,
            drag_current: None,
            is_dragging: false,
        }
    }

    fn total_spreads(&self) -> usize {
        (self.posts.len() + 1) / 2
    }

    fn draw_book_spine(&self, painter: &egui::Painter, book_rect: Rect) {
        let spine_width = 20.0;
        let spine_rect = Rect::from_min_size(
            Pos2::new(book_rect.center().x - spine_width / 2.0, book_rect.min.y),
            Vec2::new(spine_width, book_rect.height()),
        );

        // Spine gradient effect
        let spine_dark = Color32::from_rgb(80, 60, 40);
        let spine_light = Color32::from_rgb(120, 90, 60);

        painter.rect_filled(spine_rect, Rounding::ZERO, spine_dark);

        // Spine highlights
        let highlight_rect = Rect::from_min_size(
            spine_rect.min,
            Vec2::new(3.0, spine_rect.height()),
        );
        painter.rect_filled(highlight_rect, Rounding::ZERO, spine_light);
    }

    fn draw_parchment_page(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        post: Option<&BlogPost>,
        is_left: bool,
        opacity: f32,
    ) {
        let parchment_base = Color32::from_rgba_unmultiplied(245, 235, 210, (255.0 * opacity) as u8);
        let _parchment_edge = Color32::from_rgba_unmultiplied(220, 205, 175, (255.0 * opacity) as u8);

        // Page shadow
        if opacity > 0.5 {
            let shadow_offset = if is_left { 5.0 } else { -5.0 };
            let shadow_rect = rect.translate(Vec2::new(shadow_offset, 3.0));
            painter.rect_filled(
                shadow_rect,
                Rounding::same(2.0),
                Color32::from_rgba_unmultiplied(0, 0, 0, (40.0 * opacity) as u8),
            );
        }

        // Main page
        painter.rect_filled(rect, Rounding::same(2.0), parchment_base);

        // Edge effect for depth
        let edge_width = 15.0;
        let edge_rect = if is_left {
            Rect::from_min_size(
                Pos2::new(rect.max.x - edge_width, rect.min.y),
                Vec2::new(edge_width, rect.height()),
            )
        } else {
            Rect::from_min_size(rect.min, Vec2::new(edge_width, rect.height()))
        };

        // Gradient simulation for edge
        for i in 0..5 {
            let t = i as f32 / 5.0;
            let alpha = (1.0 - t) * 0.15 * opacity;
            let x = if is_left {
                edge_rect.min.x + (edge_width * t)
            } else {
                edge_rect.max.x - (edge_width * t) - 3.0
            };
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(x, edge_rect.min.y),
                    Vec2::new(3.0, edge_rect.height()),
                ),
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 255.0) as u8),
            );
        }

        // Draw content if we have a post
        if let Some(post) = post {
            self.draw_page_content(painter, rect, post, opacity);
        } else {
            // Blank page with subtle texture lines
            let line_color = Color32::from_rgba_unmultiplied(200, 190, 170, (100.0 * opacity) as u8);
            let mut y = rect.min.y + 60.0;
            while y < rect.max.y - 40.0 {
                painter.line_segment(
                    [Pos2::new(rect.min.x + 40.0, y), Pos2::new(rect.max.x - 40.0, y)],
                    Stroke::new(0.5, line_color),
                );
                y += 28.0;
            }
        }
    }

    fn draw_page_content(&self, painter: &egui::Painter, rect: Rect, post: &BlogPost, opacity: f32) {
        let text_rect = rect.shrink(40.0);
        let mut text_pos = text_rect.min;

        let title_font = FontId::proportional(26.0);
        let title_color = Color32::from_rgba_unmultiplied(60, 50, 40, (255.0 * opacity) as u8);

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            &post.title,
            title_font,
            title_color,
        );

        text_pos.y += 45.0;

        let date_font = FontId::proportional(13.0);
        let date_color = Color32::from_rgba_unmultiplied(100, 90, 80, (255.0 * opacity) as u8);

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            &post.date,
            date_font.clone(),
            date_color,
        );

        text_pos.y += 18.0;

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            &format!("By {}", post.author),
            date_font,
            date_color,
        );

        text_pos.y += 30.0;

        // Decorative line
        let line_color = Color32::from_rgba_unmultiplied(180, 160, 130, (200.0 * opacity) as u8);
        painter.line_segment(
            [
                Pos2::new(text_rect.min.x, text_pos.y),
                Pos2::new(text_rect.max.x, text_pos.y),
            ],
            Stroke::new(1.0, line_color),
        );

        text_pos.y += 20.0;

        // Content with word wrap
        let content_font = FontId::proportional(16.0);
        let content_color = Color32::from_rgba_unmultiplied(40, 35, 30, (255.0 * opacity) as u8);
        let max_width = text_rect.width();
        let line_height = 24.0;

        let words: Vec<&str> = post.content.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let galley = painter.layout_no_wrap(test_line.clone(), content_font.clone(), content_color);

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

                    if text_pos.y > text_rect.max.y - line_height {
                        break;
                    }
                }
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }

        if !current_line.is_empty() && text_pos.y <= text_rect.max.y - line_height {
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
        painter: &egui::Painter,
        rect: Rect,
        post: Option<&BlogPost>,
        progress: f32,
        turning_forward: bool,
    ) {
        let eased = ease_out_cubic(progress);
        let page_width = rect.width();
        let page_height = rect.height();

        // Calculate the fold line position
        let fold_x = if turning_forward {
            rect.max.x - (page_width * eased)
        } else {
            rect.min.x + (page_width * eased)
        };

        // Draw the static part of the page (not yet turned)
        let static_width = if turning_forward {
            fold_x - rect.min.x
        } else {
            rect.max.x - fold_x
        };

        if static_width > 1.0 {
            let static_rect = if turning_forward {
                Rect::from_min_size(rect.min, Vec2::new(static_width, page_height))
            } else {
                Rect::from_min_size(
                    Pos2::new(fold_x, rect.min.y),
                    Vec2::new(static_width, page_height),
                )
            };
            self.draw_parchment_page(painter, static_rect, post, !turning_forward, 1.0);
        }

        // Draw the curling/folded part
        if eased > 0.02 && eased < 0.98 {
            let curl_width = (page_width * eased).min(page_width * 0.5);
            let curl_intensity = (eased * std::f32::consts::PI).sin();

            // Shadow under the curl
            let shadow_width = curl_width * 0.8;
            let shadow_alpha = curl_intensity * 0.4;
            let shadow_x = if turning_forward {
                fold_x - shadow_width * 0.5
            } else {
                fold_x - shadow_width * 0.5
            };

            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(shadow_x, rect.min.y + 5.0),
                    Vec2::new(shadow_width, page_height - 10.0),
                ),
                Rounding::same(5.0),
                Color32::from_rgba_unmultiplied(0, 0, 0, (shadow_alpha * 255.0) as u8),
            );

            // The folded page back (shows the reverse side)
            let fold_segments = 8;
            for i in 0..fold_segments {
                let t = i as f32 / fold_segments as f32;
                let segment_width = curl_width / fold_segments as f32;

                // Simulate the curve of the fold
                let curve_offset = (t * std::f32::consts::PI).sin() * curl_width * 0.15;
                let depth_factor = (t * std::f32::consts::PI * 0.5).cos();

                let segment_x = if turning_forward {
                    fold_x + (t * curl_width) - curve_offset
                } else {
                    fold_x - (t * curl_width) - curl_width + curve_offset
                };

                // Lighting based on fold angle
                let lighting = 0.7 + 0.3 * depth_factor;
                let r = (230.0 * lighting) as u8;
                let g = (220.0 * lighting) as u8;
                let b = (195.0 * lighting) as u8;

                let segment_rect = Rect::from_min_size(
                    Pos2::new(segment_x, rect.min.y + curve_offset * 0.3),
                    Vec2::new(segment_width + 2.0, page_height - curve_offset * 0.6),
                );

                painter.rect_filled(
                    segment_rect,
                    Rounding::same(1.0),
                    Color32::from_rgb(r, g, b),
                );
            }

            // Fold edge highlight
            let highlight_alpha = curl_intensity * 0.6;
            painter.line_segment(
                [
                    Pos2::new(fold_x, rect.min.y),
                    Pos2::new(fold_x, rect.max.y),
                ],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, (highlight_alpha * 255.0) as u8)),
            );

            // Inner shadow on the fold
            let inner_shadow_width = 8.0;
            for i in 0..4 {
                let t = i as f32 / 4.0;
                let alpha = (1.0 - t) * curl_intensity * 0.2;
                let x = if turning_forward {
                    fold_x + (t * inner_shadow_width)
                } else {
                    fold_x - (t * inner_shadow_width)
                };
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 255.0) as u8)),
                );
            }
        }
    }

    fn draw_navigation_hints(&self, painter: &egui::Painter, book_rect: Rect) {
        let hint_color = Color32::from_rgba_unmultiplied(255, 255, 255, 150);
        let font = FontId::proportional(14.0);
        let hint_y = book_rect.max.y + 25.0;

        if self.current_page > 0 {
            painter.text(
                Pos2::new(book_rect.min.x + 20.0, hint_y),
                egui::Align2::LEFT_CENTER,
                "< Previous",
                font.clone(),
                hint_color,
            );
        }

        if self.current_page < self.total_spreads().saturating_sub(1) {
            painter.text(
                Pos2::new(book_rect.max.x - 20.0, hint_y),
                egui::Align2::RIGHT_CENTER,
                "Next >",
                font.clone(),
                hint_color,
            );
        }

        // Page indicator
        let page_text = format!("Spread {} of {}", self.current_page + 1, self.total_spreads().max(1));
        painter.text(
            Pos2::new(book_rect.center().x, hint_y),
            egui::Align2::CENTER_CENTER,
            &page_text,
            font,
            hint_color,
        );
    }
}

impl eframe::App for JournalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(74, 63, 53)))
            .show(ctx, |ui| {
                let available = ui.available_rect_before_wrap();

                // Book dimensions
                let book_width = (available.width() * 0.85).min(900.0);
                let book_height = (available.height() * 0.85).min(600.0);
                let book_rect = Rect::from_center_size(
                    available.center(),
                    Vec2::new(book_width, book_height),
                );

                let page_width = (book_width - 30.0) / 2.0;
                let left_page_rect = Rect::from_min_size(
                    book_rect.min,
                    Vec2::new(page_width, book_height),
                );
                let right_page_rect = Rect::from_min_size(
                    Pos2::new(book_rect.min.x + page_width + 30.0, book_rect.min.y),
                    Vec2::new(page_width, book_height),
                );

                let painter = ui.painter();

                // Draw book cover/background
                painter.rect_filled(
                    book_rect.expand(15.0),
                    Rounding::same(8.0),
                    Color32::from_rgb(100, 70, 45),
                );
                painter.rect_filled(
                    book_rect.expand(10.0),
                    Rounding::same(6.0),
                    Color32::from_rgb(139, 90, 43),
                );

                // Get current spread's posts (left and right)
                let left_post = self.posts.get(self.current_page * 2);
                let right_post = self.posts.get(self.current_page * 2 + 1);

                // Draw pages - each spread shows two posts
                if self.is_turning {
                    // Animate the page turn
                    if self.turn_direction > 0 {
                        // Turning forward - right page curls to reveal next spread
                        let next_left = self.posts.get((self.current_page + 1) * 2);
                        let next_right = self.posts.get((self.current_page + 1) * 2 + 1);

                        // Draw the next spread underneath
                        self.draw_parchment_page(painter, left_page_rect, next_left, true, 0.8);
                        self.draw_parchment_page(painter, right_page_rect, next_right, false, 0.8);

                        // Draw the curling page on top (current right page)
                        self.draw_curling_page(painter, right_page_rect, right_post, self.page_turn_progress, true);
                    } else {
                        // Turning backward - revealing previous spread
                        let prev_left = if self.current_page > 0 {
                            self.posts.get((self.current_page - 1) * 2)
                        } else {
                            None
                        };
                        let prev_right = if self.current_page > 0 {
                            self.posts.get((self.current_page - 1) * 2 + 1)
                        } else {
                            None
                        };

                        // Draw the previous spread underneath
                        self.draw_parchment_page(painter, left_page_rect, prev_left, true, 0.8);
                        self.draw_parchment_page(painter, right_page_rect, prev_right, false, 0.8);

                        // Draw the curling page on top (current left page curling back)
                        self.draw_curling_page(painter, left_page_rect, left_post, self.page_turn_progress, false);
                    }

                    // Animate
                    self.page_turn_progress += 0.03;
                    if self.page_turn_progress >= 1.0 {
                        self.is_turning = false;
                        self.page_turn_progress = 0.0;
                        if self.turn_direction > 0 {
                            self.current_page = (self.current_page + 1).min(self.total_spreads().saturating_sub(1));
                        } else if self.current_page > 0 {
                            self.current_page -= 1;
                        }
                        self.turn_direction = 0;
                    }
                    ctx.request_repaint();
                } else {
                    // Static view - show current spread
                    self.draw_parchment_page(painter, left_page_rect, left_post, true, 1.0);
                    self.draw_parchment_page(painter, right_page_rect, right_post, false, 1.0);
                }

                // Draw spine
                self.draw_book_spine(painter, book_rect);

                // Navigation hints
                self.draw_navigation_hints(painter, book_rect);

                // Handle input - extend clickable area to include navigation hints below
                let interaction_rect = Rect::from_min_max(
                    book_rect.min,
                    Pos2::new(book_rect.max.x, book_rect.max.y + 50.0),
                );
                let response = ui.allocate_rect(interaction_rect, egui::Sense::click_and_drag());

                if response.clicked() && !self.is_turning {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let center_x = book_rect.center().x;
                        if pos.x > center_x && self.current_page < self.total_spreads().saturating_sub(1) {
                            // Click on right side - turn forward
                            self.is_turning = true;
                            self.turn_direction = 1;
                            self.page_turn_progress = 0.0;
                        } else if pos.x < center_x && self.current_page > 0 {
                            // Click on left side - turn backward
                            self.is_turning = true;
                            self.turn_direction = -1;
                            self.page_turn_progress = 0.0;
                        }
                    }
                }

                // Drag-based page turning
                if response.drag_started() {
                    self.drag_start = response.interact_pointer_pos();
                    self.is_dragging = true;
                }

                if self.is_dragging && !self.is_turning {
                    if let (Some(start), Some(current)) = (self.drag_start, response.interact_pointer_pos()) {
                        let drag_distance = current.x - start.x;
                        let threshold = book_width * 0.15;

                        if drag_distance < -threshold && self.current_page < self.total_spreads().saturating_sub(1) {
                            self.is_turning = true;
                            self.turn_direction = 1;
                            self.page_turn_progress = 0.0;
                            self.is_dragging = false;
                        } else if drag_distance > threshold && self.current_page > 0 {
                            self.is_turning = true;
                            self.turn_direction = -1;
                            self.page_turn_progress = 0.0;
                            self.is_dragging = false;
                        }
                    }
                }

                if response.drag_stopped() {
                    self.is_dragging = false;
                    self.drag_start = None;
                }
            });
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
