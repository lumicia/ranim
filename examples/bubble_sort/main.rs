use log::LevelFilter;
use rand::{SeedableRng, seq::SliceRandom};
use ranim::{
    animation::transform::TransformAnim,
    color::palettes::manim,
    components::Anchor,
    glam::{DVec3, dvec2},
    items::vitem::geometry::Rectangle,
    prelude::*,
    timeline::TimeMark,
    utils::rate_functions::linear,
};

#[scene]
struct BubbleSortScene(pub usize);

impl SceneConstructor for BubbleSortScene {
    fn construct(self, r: &mut RanimScene, _r_cam: ItemId<CameraFrame>) {
        let num = self.0;

        let frame_size = dvec2(8.0 * 16.0 / 9.0, 8.0);
        let padded_frame_size = frame_size * 0.9;

        let anim_step_duration = 15.0 / num.pow(2) as f64;

        let width_unit = padded_frame_size.x / num as f64;
        let height_unit = padded_frame_size.y / num as f64;

        let mut rng = rand_chacha::ChaChaRng::seed_from_u64(114514);
        let mut heights = (1..=num)
            .map(|x| x as f64 * height_unit)
            .collect::<Vec<f64>>();
        heights.shuffle(&mut rng);

        let padded_frame_bl = dvec2(padded_frame_size.x / -2.0, padded_frame_size.y / -2.0);
        let mut r_rects = heights
            .iter()
            .enumerate()
            .map(|(i, &height)| {
                let target_bc_coord = padded_frame_bl.extend(0.0)
                    + DVec3::X * (width_unit * i as f64 + width_unit / 2.0);
                let rect = Rectangle::new(width_unit, height).with(|rect| {
                    rect.stroke_width = 0.0;
                    rect.set_fill_color(manim::WHITE.with_alpha(0.5))
                        .scale(DVec3::splat(0.8))
                        .put_anchor_on(Anchor::edge(0, -1, 0), target_bc_coord);
                });
                r.insert(rect)
            })
            .collect::<Vec<_>>();

        let anim_highlight = |rect: Rectangle| {
            rect.transform(|data| {
                data.set_fill_color(manim::BLUE_C.with_alpha(0.5));
            })
            .with_duration(anim_step_duration)
            .with_rate_func(linear)
        };
        let anim_unhighlight = |rect: Rectangle| {
            rect.transform(|data| {
                data.set_fill_color(manim::WHITE.with_alpha(0.5));
            })
            .with_duration(anim_step_duration)
            .with_rate_func(linear)
        };
        let shift_right = DVec3::X * width_unit;
        let swap_shift = [shift_right, -shift_right];
        let anim_swap = |timeline: &mut RanimScene, r_rectab: &[&ItemId<Rectangle>; 2]| {
            let timelines = timeline.timeline_mut(r_rectab);
            timelines
                .into_iter()
                .zip(swap_shift.iter())
                .for_each(|(timeline, shift)| {
                    timeline.play_with(|rect| {
                        rect.transform(|data| {
                            data.shift(*shift);
                        })
                        .with_duration(anim_step_duration)
                        .with_rate_func(linear)
                    });
                });
        };

        for i in (1..num).rev() {
            for j in 0..i {
                r.timeline_mut(&[&r_rects[j], &r_rects[j + 1]])
                    .into_iter()
                    .for_each(|timeline| {
                        timeline.play_with(anim_highlight);
                    });
                if heights[j] > heights[j + 1] {
                    anim_swap(r, &[&r_rects[j], &r_rects[j + 1]]);
                    r.timelines_mut().sync();
                    heights.swap(j, j + 1);
                    r_rects.swap(j, j + 1);
                }
                r.timeline_mut(&[&r_rects[j], &r_rects[j + 1]])
                    .into_iter()
                    .for_each(|timeline| {
                        timeline.play_with(anim_unhighlight);
                    });
                r.timelines_mut().sync();
            }
        }

        r.insert_time_mark(
            r.timelines().max_total_secs(),
            TimeMark::Capture(format!("preview-{num}.png")),
        );
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(debug_assertions)]
        pretty_env_logger::formatted_timed_builder()
            .filter(Some("ranim"), LevelFilter::Trace)
            .init();
        #[cfg(not(debug_assertions))]
        pretty_env_logger::formatted_timed_builder()
            .filter(Some("ranim"), LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "app")]
    run_scene_app(BubbleSortScene(100));
    #[cfg(not(feature = "app"))]
    {
        render_scene(
            BubbleSortScene(10),
            &AppOptions {
                output_filename: "output-10.mp4",
                ..Default::default()
            },
        );
        render_scene(
            BubbleSortScene(100),
            &AppOptions {
                output_filename: "output-100.mp4",
                ..Default::default()
            },
        );
    }
}
