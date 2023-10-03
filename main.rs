// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

#![deny(unsafe_code)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

use std::rc::Rc;
use plotters::prelude::*;
use slint::SharedPixelBuffer;

use slint::{Model, StandardListViewItem, VecModel};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let app = App::new();

    let row_data: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());

    for r in 1..101 {
        let items = Rc::new(VecModel::default());

        for c in 1..5 {
            items.push(StandardListViewItem {
                text: format!("Item {r}.{c}").into(),
                editable: c == 1,
            });
        }

        row_data.push(items.into());
    }

    app.global::<TableViewPageAdapter>().set_row_data(row_data.clone().into());

    app.global::<TableViewPageAdapter>().on_sort_ascending({
        let app_weak = app.as_weak();
        let row_data = row_data.clone();
        move |index| {
            let row_data = row_data.clone();

            let sort_model = Rc::new(row_data.sort_by(move |r_a, r_b| {
                let c_a = r_a.row_data(index as usize).unwrap();
                let c_b = r_b.row_data(index as usize).unwrap();

                c_a.text.cmp(&c_b.text)
            }));

            app_weak.unwrap().global::<TableViewPageAdapter>().set_row_data(sort_model.into());
        }
    });

    app.global::<TableViewPageAdapter>().on_sort_descending({
        let app_weak = app.as_weak();
        let row_data = row_data.clone();
        move |index| {
            let row_data = row_data.clone();

            let sort_model = Rc::new(row_data.sort_by(move |r_a, r_b| {
                let c_a = r_a.row_data(index as usize).unwrap();
                let c_b = r_b.row_data(index as usize).unwrap();

                c_b.text.cmp(&c_a.text)
            }));

            app_weak.unwrap().global::<TableViewPageAdapter>().set_row_data(sort_model.into());
        }
    });

    app.global::<GallerySettings>().on_render_plot(render_plot);


    app.run();
}


fn pdf(x: f64, y: f64, a: f64) -> f64 {
    const SDX: f64 = 0.1;
    const SDY: f64 = 0.1;
    let x = x as f64 / 10.0;
    let y = y as f64 / 10.0;
    a * (-x * x / 2.0 / SDX / SDX - y * y / 2.0 / SDY / SDY).exp()
}

fn render_plot(width: f32, pitch: f32, yaw: f32, amplitude: f32) -> slint::Image {

    println!("width: {width}");

    let mut pixel_buffer = SharedPixelBuffer::new(640, 480);
    let size = (pixel_buffer.width(), pixel_buffer.height());

    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);

    // Plotters requires TrueType fonts from the file system to draw axis text - we skip that for
    // WASM for now.
    #[cfg(target_arch = "wasm32")]
    let backend = wasm_backend::BackendWithoutText { backend };

    let root = backend.into_drawing_area();

    root.fill(&WHITE).expect("error filling drawing area");

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_3d(-3.0..3.0, 0.0..6.0, -3.0..3.0)
        .expect("error building coordinate system");
    chart.with_projection(|mut p| {
        p.pitch = pitch as f64;
        p.yaw = yaw as f64;
        p.scale = 0.7;
        p.into_matrix() // build the projection matrix
    });

    chart.configure_axes().draw().expect("error drawing");

    chart
        .draw_series(
            SurfaceSeries::xoz(
                (-15..=15).map(|x| x as f64 / 5.0),
                (-15..=15).map(|x| x as f64 / 5.0),
                |x, y| pdf(x, y, amplitude as f64),
            )
            .style_func(&|&v| {
                (&HSLColor(240.0 / 360.0 - 240.0 / 360.0 * v / 5.0, 1.0, 0.7)).into()
            }),
        )
        .expect("error drawing series");

    root.present().expect("error presenting");
    drop(chart);
    drop(root);

    slint::Image::from_rgb8(pixel_buffer)
}