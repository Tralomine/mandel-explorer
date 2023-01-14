use std::f64::consts::PI;

use sfml;
use sfml::window::{Key, Event, mouse};
use sfml::graphics::{Color, RenderTarget};
use sfml::system::Vector2f;

pub mod vec2;
pub mod cplx;

fn get_mandel(c: cplx::Cplx<f64>, iter_max: usize) -> usize {
    let mut z = c;
    for n in 1..iter_max {
        if z.sq_abs() >= 4. {
            return n;
        }
        z = z.square()+c;
    }
    iter_max
}

fn get_mandel_smooth(c: cplx::Cplx<f64>, iter_max: usize) -> f64 {
    let mut z = c;
    let mut n = 0.;
    for i in 1..iter_max {
        if z.sq_abs() >= 1048576. { //1024**2, M = 1024
            n = i as f64;
            break;
        }
        z = z.square()+c;
    }
    if n == 0. {return iter_max as f64;}

    n + 1. + (10./z.abs().log2()).log2()
    // N + 1 + ln(p)ln(ln(M)/ln(r)) //M = big escape value, p = power (2 here), r = radius at escape

    // fn inv_sqrt(x: f32) -> f32 {
    //     let i = x.to_bits();
    //     let i = 0x5f3759df - (i >> 1);
    //     let y = f32::from_bits(i);
    
    //     y * (1.5 - 0.5 * x * y * y)
    // }
}

fn mm_color(t: f64) -> Color {
    const A: (f64, f64, f64) = (0.5, 0.5, 0.5);
    const B: (f64, f64, f64) = (0.5, 0.5, 0.5);
    const C: (f64, f64, f64) = (1.0, 1.0, 1.0);
    const D: (f64, f64, f64) = (0.0, 0.1, 0.2);
    Color::rgb(
        ((A.0+B.0*((C.0*t+D.0)*2.*PI).cos()) * 256.) as u8,
        ((A.1+B.1*((C.1*t+D.1)*2.*PI).cos()) * 256.) as u8,
        ((A.2+B.2*((C.2*t+D.2)*2.*PI).cos()) * 256.) as u8,
    )
}

fn hsv_to_rgb(h: i64, s: f64, v: f64) -> Color {
    //360, 1, 1
    let h = (h%360) as f64;
    let c = v*s;
    let x = c*1.-(((h/60.).rem_euclid(2.))-1.).abs();
    let m = v-c;

    let (r, g, b) = {
        if      h < 60.  {(c, x, 0.)}
        else if h < 120. {(x, c, 0.)}
        else if h < 180. {(0., c, x)}
        else if h < 240. {(0., x, c)}
        else if h < 300. {(x, 0., c)}
        else               {(c, 0., x)}
    };

    Color::rgb(((r+m) * 256.) as u8, ((g+m) * 256.) as u8, ((b+m) * 256.) as u8)
}

fn pos_to_cplx(x:i32, y:i32, width:i32, height:i32, zoom:f64, offset: cplx::Cplx<f64>) -> cplx::Cplx<f64> {
    let min = std::cmp::min(width, height) as f64;
    cplx::Cplx{
        re: ((x-width/2) as f64)/min/zoom,
        im: ((y-height/2) as f64)/min/zoom,
    } + offset
}

fn main() {
    let mut w = 640;
    let mut h = 480;

    let mut app = sfml::graphics::RenderWindow::new(
        sfml::window::VideoMode::new(w, h, 32),
        "mandel",
        sfml::window::Style::RESIZE,
        &sfml::window::ContextSettings::default()
    );

    let mut vertices = Vec::new();

    let mut redraw = true;
    let mut zoom = 0.4;
    let mut offset = cplx::Cplx{re:-0.5,im:0.};
    let mut iter_max = 256;
    let mut smooth = true;

    while app.is_open() {
        while let Some(event) = app.poll_event() {
            match event {
                Event::Closed => app.close(),
                Event::MouseButtonPressed{button, x, y} => {
                    offset = pos_to_cplx(x as i32, y as i32, w as i32, h as i32, zoom, offset);
                    if button == mouse::Button::Left {
                        zoom *= 2.;
                    }
                    if button == mouse::Button::Right {
                        zoom /= 2.;
                    }
                    redraw = true;
                },
                Event::Resized{width, height} => {
                    (w, h) = (width, height);
                    redraw = true;
                    app.set_view(&sfml::graphics::View::from_rect(&sfml::graphics::FloatRect::new(0., 0., w as f32, h as f32)));
                },
                Event::KeyPressed{code, alt, ctrl, shift, system} => {
                    match code {
                        Key::Equal => {
                            iter_max *= 2;
                            redraw = true;
                        },
                        Key::Hyphen => {
                            if iter_max > 32 {iter_max /= 2;redraw = true;}
                        },
                        Key::Space => {
                            redraw = true;
                            smooth = !smooth;
                        },
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
        if redraw {
            vertices.clear();
            for x in 0..w {
                for y in 0..h {
                    let c = pos_to_cplx(x as i32, y as i32, w as i32, h as i32, zoom, offset);
                    let color = {
                        let n = if smooth {
                            get_mandel_smooth(c, iter_max)
                        } else {
                            get_mandel(c, iter_max) as f64
                        };
                        if n as usize == iter_max {
                            Color::BLACK
                        } else {
                            // hsv_to_rgb((n*8) as i64, 1., 1.)
                            mm_color(n.ln())
                        }
                    };
                    let v = sfml::graphics::Vertex::new(
                        Vector2f::new(x as f32, y as f32),
                        color,
                        Vector2f::new(0., 0.),
                    );
                    vertices.push(v);
                }
            }
            redraw = false;
        }

        app.clear(Color::BLACK);
        app.draw_primitives(&vertices, sfml::graphics::PrimitiveType::POINTS, &sfml::graphics::RenderStates::default());
        app.display();
    }
}
