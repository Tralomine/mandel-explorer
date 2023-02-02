use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use sfml;
use sfml::window::{Key, Event, mouse};
use sfml::graphics::{Color, RenderTarget, Transformable};
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

fn area(tx: mpsc::Sender<(usize, usize, Mandel)>, rect: sfml::graphics::Rect<usize>, zoom: f64, offset: cplx::Cplx<f64>, size: (usize, usize), iter_max: usize) {
    let tx = tx.clone();
    let mut closed = true;
    let mut calculate_and_send = |x, y| {
        let mut m = Mandel::new(pos_to_cplx(x as i32, y as i32, size.0, size.1, zoom, offset), iter_max);
        m.calculate_mandel_smooth();
        if m.get_finished().unwrap().is_finite() {closed = false;}
        if let Err(_) = tx.send((x, y, m)) {return false;}
        true
    };
    for x in rect.left..rect.left+rect.width {
        if  !calculate_and_send(x, rect.top) ||
            !calculate_and_send(x, rect.top+rect.height-1) {
            return;
        }
    }
    for y in rect.top..rect.top+rect.height {
        if  !calculate_and_send(rect.left, y) ||
            !calculate_and_send(rect.left+rect.width-1, y) {
            return;
        }
    }

    if !closed {
        if rect.width < 32 || rect.height < 32 {
            for x in rect.left+1..rect.left+rect.width-1 {
                for y in rect.top+1..rect.top+rect.height-1 {
                    let mut m = Mandel::new(pos_to_cplx(x as i32, y as i32, size.0, size.1, zoom, offset), iter_max);
                    m.calculate_mandel_smooth();
                    if let Err(_) = tx.send((x, y, m)) {return;}
                }
            }
            return;
        }

        let tx1 = tx.clone();
        let tx2 = tx.clone();
        if rect.width > rect.height {
            thread::spawn(move || {
                area(tx1, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+1, width: rect.width/2, height: rect.height-2}, zoom, offset, size, iter_max);
            });
            thread::spawn(move || {
                area(tx2, sfml::graphics::Rect::<usize>{left:rect.left+rect.width/2, top:rect.top+1, width: rect.width/2, height: rect.height-2}, zoom, offset, size, iter_max);
            });
        } else {
            thread::spawn(move || {
                area(tx1, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+1, width: rect.width-2, height: rect.height/2}, zoom, offset, size, iter_max);
            });
            thread::spawn(move || {
                area(tx2, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+rect.height/2, width: rect.width-2, height: rect.height/2}, zoom, offset, size, iter_max);
            });
        }
    }
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

    let mut mandels = vec![vec![Mandel::new_empty();h];w];

    let mut redraw = false;
    let mut zoom = 0.25;
    let mut offset = cplx::Cplx{re:-0.5,im:0.};
    // let mut julia_center = offset;
    // let mut smooth = true;
    let mut iter_max = 256;

    let fira = sfml::graphics::Font::from_file("fira.otf").unwrap();
    let mut show_debug = true;

    let mut tx_calc;
    let (_, mut rx_calc) = mpsc::channel();

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
                    mandels = vec![vec![Mandel::new_empty();h];w];
                    redraw = true;
                    app.set_view(&sfml::graphics::View::from_rect(&sfml::graphics::FloatRect::new(0., 0., w as f32, h as f32)));
                },
                Event::KeyPressed{code, ..} => {
                    match code {
                        Key::Equal => {
                            iter_max *= 2;
                            redraw = true;
                        }
                        Key::Hyphen => {
                            if iter_max > 32 {iter_max /= 2;redraw = true;}
                        }
                        // Key::Space => {
                        //     redraw = true;
                        //     smooth = !smooth;
                        //     julia_center = offset;
                        // }
                        Key::Num0 => {
                            redraw = true;
                            zoom = 0.4;
                            offset = cplx::Cplx{re:-0.5,im:0.};
                            iter_max = 256;
                        }
                        Key::F3 => {
                            show_debug = !show_debug;
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        if redraw {
            (tx_calc, rx_calc) = mpsc::channel();
            // mandels = vec![vec![Mandel::new_empty();h];w];
            area(tx_calc.clone(), sfml::graphics::Rect::<usize>{left:0, top:0, width:w, height:h}, zoom, offset, (w, h), iter_max);

            redraw = false;
        }

        loop {
            match rx_calc.try_recv() {
                Ok((x, y, m)) => mandels[x][y] = m,
                Err(_) => break,
            }
            if frame_start.elapsed() >= Duration::from_secs_f64(1./40.) {break;}
        }


        app.clear(Color::BLACK);

        let mut vertices = Vec::with_capacity(h*w);
        for x in 0..w {
            for y in 0..h {
                let n = match mandels[x][y].get_finished() {
                    Some(x) => x,
                    None => f64::NEG_INFINITY
                };
                if n.is_finite() {
                    let n = 0.5*mandel::fast_log2(n);
                    // let n = 0.5*n.sqrt() + 3.3;
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
                    // let color = colors::hsv_to_rgb(n*32., 0.8, 0.8);
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

        if show_debug {
            let fps = 1./frame_start.elapsed().as_secs_f64();
            let txt = format!("pos: {} + {}i\nzoom: 2^{}\nfps: {fps}\n", offset.re, offset.im, zoom.log2());

            let mut text = sfml::graphics::Text::new(&txt, &fira, 24);
            text.set_outline_thickness(2.);
            text.set_position(sfml::system::Vector2::<f32>{x: 24., y: 24.});
            app.draw(&text);
        }

        app.display();

    }
}
