use image::{ImageBuffer, Rgba, imageops::FilterType};
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = 256u32;
    let mut img = ImageBuffer::from_pixel(size, size, Rgba([0, 0, 0, 0]));

    let bg_top = (30.0, 27.0, 75.0); // #1E1B4B
    let bg_bot = (49.0, 46.0, 129.0); // #312E81
    let stroke_indigo = Rgba([79, 70, 229, 255]); // #4F46E5
    let grid_bg = Rgba([24, 24, 56, 255]); // #181838
    let grid_line = Rgba([67, 56, 202, 255]); // #4338CA
    let grid_line_subtle = Rgba([55, 48, 163, 200]); // #3730A3
    let inactive_dot = Rgba([75, 85, 99, 255]); // #4B5563
    let loop_fill = Rgba([15, 175, 154, 60]); // #0FAF9A transparent
    let loop_stroke = Rgba([45, 212, 191, 255]); // #2DD4BF
    let active_node = Rgba([45, 212, 191, 255]); // #2DD4BF

    let pad = 16.0;
    let box_size = 224.0;
    let radius = 48.0;

    // Helper: Distance to rounded box
    fn rounded_box_dist(x: f32, y: f32, bx: f32, by: f32, bw: f32, bh: f32, r: f32) -> f32 {
        let cx = bx + bw / 2.0;
        let cy = by + bh / 2.0;
        let dx = (x - cx).abs() - (bw / 2.0 - r);
        let dy = (y - cy).abs() - (bh / 2.0 - r);
        let ax = dx.max(0.0);
        let ay = dy.max(0.0);
        let outside = (ax * ax + ay * ay).sqrt();
        let inside = dx.max(dy).min(0.0);
        outside + inside - r
    }

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let d_outer = rounded_box_dist(px, py, pad, pad, box_size, box_size, radius);
            if d_outer > 1.0 {
                continue;
            }

            // Outer background gradient
            let t = (py - pad) / box_size;
            let r = (bg_top.0 + t * (bg_bot.0 - bg_top.0)) as u8;
            let g = (bg_top.1 + t * (bg_bot.1 - bg_top.1)) as u8;
            let b = (bg_top.2 + t * (bg_bot.2 - bg_top.2)) as u8;
            let mut col = Rgba([r, g, b, 255]);

            // Outer border (width ~3px)
            if d_outer >= -3.5 {
                col = stroke_indigo;
            }

            // Inner K-map Grid (48..208, 160x160, r=16)
            let d_grid = rounded_box_dist(px, py, 48.0, 48.0, 160.0, 160.0, 16.0);
            if d_grid <= 0.0 {
                col = grid_bg;

                // Subtle grid lines
                for &line_x in &[88.0, 128.0, 168.0] {
                    if (px - line_x).abs() <= 1.0 {
                        col = if (line_x - 128.0).abs() < 1.0 {
                            grid_line
                        } else {
                            grid_line_subtle
                        };
                    }
                }
                for &line_y in &[88.0, 128.0, 168.0] {
                    if (py - line_y).abs() <= 1.0 {
                        col = if (line_y - 128.0).abs() < 1.0 {
                            grid_line
                        } else {
                            grid_line_subtle
                        };
                    }
                }

                // Inactive dots
                let cell_centers = [68.0, 108.0, 148.0, 188.0];
                for &cx in &cell_centers {
                    for &cy in &cell_centers {
                        let is_active =
                            (cx == 108.0 || cx == 148.0) && (cy == 108.0 || cy == 148.0);
                        let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt();
                        if !is_active && dist <= 3.0 {
                            col = inactive_dot;
                        }
                    }
                }

                // Active Loop at (92, 92, 72x72, r=16)
                let d_loop = rounded_box_dist(px, py, 92.0, 92.0, 72.0, 72.0, 16.0);
                if d_loop <= 0.0 {
                    // Alpha blend loop fill
                    let a = loop_fill[3] as f32 / 255.0;
                    let nr = (col[0] as f32 * (1.0 - a) + loop_fill[0] as f32 * a) as u8;
                    let ng = (col[1] as f32 * (1.0 - a) + loop_fill[1] as f32 * a) as u8;
                    let nb = (col[2] as f32 * (1.0 - a) + loop_fill[2] as f32 * a) as u8;
                    col = Rgba([nr, ng, nb, 255]);
                }
                if d_loop.abs() <= 2.0 {
                    col = loop_stroke;
                }

                // Active nodes
                for &cx in &[108.0, 148.0] {
                    for &cy in &[108.0, 148.0] {
                        let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt();
                        if dist <= 6.0 {
                            col = active_node;
                        }
                    }
                }

                // Border of grid
                if d_grid >= -2.0 {
                    col = grid_line;
                }
            }

            // Antialiasing for outer container
            if d_outer > 0.0 {
                let alpha = ((1.0 - d_outer) * 255.0).clamp(0.0, 255.0) as u8;
                col[3] = alpha;
            }

            img.put_pixel(x, y, col);
        }
    }

    // Ensure assets directory exists
    std::fs::create_dir_all("assets")?;
    std::fs::create_dir_all("assets/screenshots")?;

    // 1. Save icon-256.png
    let png_path = Path::new("assets/icon-256.png");
    img.save(png_path)?;
    println!("Saved {}", png_path.display());

    // 2. Build multi-resolution icon.ico: 16, 32, 48, 64, 128, 256
    let sizes = [16u32, 32, 48, 64, 128, 256];
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &s in &sizes {
        let resized = if s == 256 {
            img.clone()
        } else {
            image::imageops::resize(&img, s, s, FilterType::Lanczos3)
        };

        let rgba_data = resized.into_raw();
        let icon_image = ico::IconImage::from_rgba_data(s, s, rgba_data);
        let entry = ico::IconDirEntry::encode(&icon_image)?;
        icon_dir.add_entry(entry);
    }

    let ico_path = Path::new("assets/icon.ico");
    let ico_file = File::create(ico_path)?;
    icon_dir.write(ico_file)?;
    println!(
        "Saved {} with {} resolutions",
        ico_path.display(),
        sizes.len()
    );

    Ok(())
}
