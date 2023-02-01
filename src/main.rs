use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use sfml;
use sfml::window::{Key, Event, mouse};
use sfml::graphics::{Color, RenderTarget};
use sfml::system::Vector2f;

pub mod mandel;
use mandel::cplx;
use mandel::Mandel;

// use mandel::big_float;
pub mod colors;

fn pos_to_cplx(x:i32, y:i32, width: usize, height: usize, zoom:f64, offset: cplx::Cplx<f64>) -> cplx::Cplx<f64> {
    let min = std::cmp::min(width, height) as f64;
    cplx::Cplx{
        re: ((x-width as i32/2) as f64)/min/zoom,
        im: ((y-height as i32/2) as f64)/min/zoom,
    } + offset
}

fn main() {
    let mut w = 640;
    let mut h = 480;

    let mut app = sfml::graphics::RenderWindow::new(
        sfml::window::VideoMode::desktop_mode(),
        "mandel",
        sfml::window::Style::NONE,
        &sfml::window::ContextSettings::default()
    );
    app.set_position(sfml::system::Vector2i::new(0, 0));

    let mut mandels: Vec<Vec<mandel::Mandel>> = Vec::new();

    let mut redraw = false;
    let mut zoom = 0.25;
    let mut offset = cplx::Cplx{re:-0.5,im:0.};
    // let mut julia_center = offset;
    // let mut smooth = true;
    let mut iter_max = 256;

    let (tx_calc, rx_calc) = mpsc::channel();

    while app.is_open() {
        let frame_start = Instant::now();
        while let Some(event) = app.poll_event() {
            match event {
                Event::Closed => app.close(),
                Event::MouseButtonPressed{button, x, y} => {
                    if button == mouse::Button::Left {
                        offset = pos_to_cplx(x, y, w, h, zoom, offset) -
                            pos_to_cplx(x, y, w, h, zoom*2., offset) +
                            pos_to_cplx((w/2) as i32, (h/2) as i32, w, h, zoom*2., offset);
                        zoom *= 2.;
                    }
                    if button == mouse::Button::Right {
                        offset = pos_to_cplx(x, y, w, h, zoom, offset) -
                            pos_to_cplx(x, y, w, h, zoom/2., offset) +
                            pos_to_cplx((w/2) as i32, (h/2) as i32, w, h, zoom/2., offset);
                        zoom /= 2.;
                    }
                    if button == mouse::Button::Middle {
                        offset = pos_to_cplx(x, y, w, h, zoom, offset);
                    }
                    redraw = true;
                },
                Event::Resized{width, height} => {
                    (w, h) = (width as usize, height as usize);
                    redraw = true;
                    app.set_view(&sfml::graphics::View::from_rect(&sfml::graphics::FloatRect::new(0., 0., w as f32, h as f32)));
                },
                Event::KeyPressed{code, ..} => {
                    match code {
                        Key::Equal => {
                            iter_max *= 2;
                            redraw = true;
                        },
                        Key::Hyphen => {
                            if iter_max > 32 {iter_max /= 2;redraw = true;}
                        },
                        // Key::Space => {
                        //     redraw = true;
                        //     smooth = !smooth;
                        //     julia_center = offset;
                        // },
                        Key::Num0 => {
                            redraw = true;
                            zoom = 0.4;
                            offset = cplx::Cplx{re:-0.5,im:0.};
                            iter_max = 256;
                        },
                        _ => (),
                    }
                },
                _ => ()
            }
        }
        // let now = Instant::now();
        if redraw {
            mandels = Vec::with_capacity(w);
            for x in 0..w {
                mandels.push(vec![Mandel::new_empty();h]);
                let tx = tx_calc.clone();
                thread::spawn(move || {
                    for y in 0..h {
                        let mut m = Mandel::new(pos_to_cplx(x as i32, y as i32, w, h, zoom, offset), iter_max);
                        m.get_mandel_smooth();
                        tx.send((x, y, m)).unwrap();
                    }
                });
            }

            // for m in &mandels {
            //     for m in m {
            //         m.get_mandel_smooth();
            //     }
            // }
            // println!("{:#?}", now.elapsed());
            redraw = false;
        }

        let fps = Instant::now();
        loop {
            match rx_calc.try_recv() {
                Ok((x, y, m)) => mandels[x][y] = m,
                Err(_) => break,
            }
            if fps.elapsed() >= Duration::from_secs_f64(1./60.) {break;}
        }


        app.clear(Color::BLACK);

        let mut vertices = Vec::new();
        for x in 0..w {
            for y in 0..h {
                let n = if mandels[x][y].is_done() {
                    mandels[x][y].get_mandel_smooth()
                } else { -1. };
                if n != f64::INFINITY {
                    // let n = 2.*n.log2();
                    let n = 0.5*n.sqrt() + 3.3;
                    // let n = n/8.;
                    // let p = n.fract();
                    // let p2 = n2.fract();
                    // let p = p.powi(20);
                    // let n = n.floor() + (p*2.);
                    // let n = n + (p2*0.5);
                    // let n = 20.*n;
                    // hsv_to_rgb((n*8) as i64, 1., 1.)
                    // let shadow = 128 + ((1.-p2)*128.) as u8;
                    // let shadow2 = 192 + ((1.-p)*64.) as u8;
                    // let shadow = (n*256.).abs() as u8;
                    // let color = hsv_to_rgb(n*32., 0.8, 0.8);
                    // let color = colors::hsv_to_rgb(15.*n, 0.7, 0.8-p*0.5);
                    // let color = color * Color::rgb(shadow, shadow, shadow);
                    let color = colors::mm_color(n);
                    // let color = color * Color::rgb(shadow2, shadow2, shadow2);
                    // if n.fract() <= 0.03 {Color::BLACK} else
                    let v = sfml::graphics::Vertex::new(
                        Vector2f::new(x as f32, y as f32),
                        color,
                        Vector2f::new(0., 0.),
                    );
                    vertices.push(v);
                }
            }
        }
        app.draw_primitives(&vertices, sfml::graphics::PrimitiveType::POINTS, &sfml::graphics::RenderStates::default());

        app.display();

        let frame_time = frame_start.elapsed();
        if frame_time <= Duration::from_secs_f64(1./60.) {
            thread::sleep(Duration::from_secs_f64(1./60.) - frame_time);
        }
    }
}
