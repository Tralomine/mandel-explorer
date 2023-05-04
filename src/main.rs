use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use sfml;
use sfml::window::{Key, Event, mouse};
use sfml::graphics::{Color, RenderTarget, Transformable};
use sfml::system::Vector2f;

pub mod mandel;
use mandel::cplx::{self, Cplx};
use mandel::Mandel;

// use mandel::big_float;

const PI:f64 = 3.14159265358979323846264338397;
pub mod colors;

fn pos_to_cplx(x:i32, y:i32, config: &Config) -> cplx::Cplx<f64> {
    // width: usize, height: usize, zoom:f64, offset: cplx::Cplx<f64>
    let min = std::cmp::min(config.size.0, config.size.1) as f64;
    cplx::Cplx{
        re: ((x-config.size.0 as i32/2) as f64)/min/config.zoom,
        im: ((y-config.size.1 as i32/2) as f64)/min/config.zoom,
    } + config.offset
}

fn area(tx: mpsc::Sender<(usize, usize, Mandel)>, rect: sfml::graphics::Rect<usize>, config: Config) {
    let tx = tx.clone();
    let mut closed = true;
    let mut calculate_and_send = |x, y| {
        let mut m = Mandel::new(pos_to_cplx(x as i32, y as i32, &config), config.iter_max);
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
        if rect.width < 128 || rect.height < 128 {
            for x in rect.left+1..rect.left+rect.width-1 {
                for y in rect.top+1..rect.top+rect.height-1 {
                    let mut m = Mandel::new(pos_to_cplx(x as i32, y as i32, &config), config.iter_max);
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
                area(tx1, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+1, width: rect.width/2, height: rect.height-2}, config);
            });
            thread::spawn(move || {
                area(tx2, sfml::graphics::Rect::<usize>{left:rect.left+rect.width/2, top:rect.top+1, width: rect.width/2, height: rect.height-2}, config);
            });
        } else {
            thread::spawn(move || {
                area(tx1, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+1, width: rect.width-2, height: rect.height/2}, config);
            });
            thread::spawn(move || {
                area(tx2, sfml::graphics::Rect::<usize>{left:rect.left+1, top:rect.top+rect.height/2, width: rect.width-2, height: rect.height/2}, config);
            });
        }
    }
}

#[derive(Clone, Copy)]
struct Config {
    pub size: (usize, usize),
    pub zoom: f64,
    pub offset: Cplx<f64>,
    pub iter_max: usize,
    pub smooth: bool,
    pub redraw: bool,
    pub debug: bool,
}

fn process_events(app: &mut sfml::graphics::RenderWindow, config: &mut Config) {
    while let Some(event) = app.poll_event() {
        match event {
            Event::Closed => app.close(),
            Event::MouseButtonPressed{button, x, y} => {
                match button {
                    mouse::Button::Left => {
                        let offset = pos_to_cplx(x, y, &config);
                        config.zoom *= 2.;
                        let offset = offset - pos_to_cplx(x, y, config);
                        let offset = offset + pos_to_cplx((config.size.0/2) as i32, (config.size.1/2) as i32, config);
                        config.offset = offset;
                    }
                    mouse::Button::Right => {
                        let offset = pos_to_cplx(x, y, &config);
                        config.zoom /= 2.;
                        let offset = offset - pos_to_cplx(x, y, config);
                        let offset = offset + pos_to_cplx((config.size.0/2) as i32, (config.size.1/2) as i32, config);
                        config.offset = offset;
                    }
                    mouse::Button::Middle => {
                        config.offset = pos_to_cplx(x, y, config);
                    }
                    _ => ()
                }
                config.redraw = true;
            }
            Event::Resized{width, height} => {
                config.size = (width as usize, height as usize);
                // mandels = vec![vec![Mandel::new_empty();h*2];w*2];
                config.redraw = true;
                app.set_view(&sfml::graphics::View::from_rect(&sfml::graphics::FloatRect::new(0., 0., config.size.0 as f32,  config.size.1 as f32)));
            }
            Event::KeyPressed{code, ..} => {
                match code {
                    Key::Equal => {
                        config.iter_max *= 2;
                        config.redraw = true;
                    }
                    Key::Hyphen => {
                        if config.iter_max > 32 {
                            config.iter_max /= 2;
                            config.redraw = true;
                        }
                    }
                    // Key::Space => {
                    //     redraw = true;
                    //     smooth = !smooth;
                    //     julia_center = offset;
                    // }
                    Key::Num0 => {
                        config.redraw = true;
                        config.zoom = 0.25;
                        config.offset = cplx::Cplx{re:-0.5,im:0.};
                        config.iter_max = 256;
                    }
                    Key::F3 => {
                        config.debug = !config.debug;
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }
}

fn main() {
    let mut config: Config = Config{
        size: (640, 480),
        zoom: 0.25,
        offset: cplx::Cplx{re:-0.5,im:0.},
        iter_max: 256,
        smooth: true,
        redraw: true,
        debug: true,
    };

    let mut app = sfml::graphics::RenderWindow::new(
        sfml::window::VideoMode::desktop_mode(),
        "mandel",
        sfml::window::Style::NONE,
        &sfml::window::ContextSettings::default()
    );
    app.set_position(sfml::system::Vector2i::new(0, 0));

    let mut mandels = vec![vec![Mandel::new_empty();config.size.0*2];config.size.1*2];

    let fira = sfml::graphics::Font::from_file("fira.otf").unwrap();

    let mut tx_calc;
    let (_, mut rx_calc) = mpsc::channel();

    while app.is_open() {
        let frame_start = Instant::now();
        process_events(&mut app, &mut config);
        
        if config.redraw {
            (tx_calc, rx_calc) = mpsc::channel();
            config.size.0 *= 2;
            config.size.1 *= 2;
            mandels = vec![vec![Mandel::new_empty();config.size.0*2];config.size.1*2];
            area(tx_calc.clone(), sfml::graphics::Rect::<usize>{left:0, top:0, width:config.size.0, height:config.size.1}, config);
            config.size.0 /= 2;
            config.size.1 /= 2;

            config.redraw = false;
        }

        loop {
            match rx_calc.try_recv() {
                Ok((x, y, m)) => mandels[x][y] = m,
                Err(_) => break,
            }
            if frame_start.elapsed() >= Duration::from_secs_f64(1./40.) {break;}
        }


        app.clear(Color::BLACK);

        let mut vertices = Vec::with_capacity(config.size.0*config.size.1);
        for x in 0..config.size.0 {
            for y in 0..config.size.1 {
                let mut darken = 0;
                let mut n = 0.;
                let mut check_pos = |x:usize, y:usize| {
                    match mandels[x][y].get_finished() {
                        Some(k) => if k.is_finite() {n += k} else {darken += 1}
                        None => darken += 1
                    }
                };
                check_pos(x*2, y*2);
                check_pos(x*2+1, y*2);
                check_pos(x*2, y*2+1);
                check_pos(x*2+1, y*2+1);

                let ang = if darken == 0 {(
                    mandels[x*2][y*2].get_finished().unwrap()-mandels[x*2+1][y*2+1].get_finished().unwrap(),
                    mandels[x*2+1][y*2].get_finished().unwrap()-mandels[x*2][y*2+1].get_finished().unwrap()
                )} else {(0., 0.)};

                if darken < 4 {
                    let n = n / (4.-darken as f64);

                    // let n = 0.5*mandel::fast_log2(n);
                    let n = 0.5*n.sqrt() + 3.3;
                    // let n = n/8.;
                    // let p = n.fract();
                    // let p2 = n2.fract();harassment
                    // let p = p.powi(20);
                    // let n = n.floor() + (p*2.);
                    // let n = n + (p2*0.5);
                    // let n = 20.*n;
                    // hsv_to_rgb((n*8) as i64, 1., 1.)
                    // let shadow = 128 + ((1.-p2)*128.) as u8;
                    // let shadow2 = 192 + ((1.-p)*64.) as u8;
                    // let shadow = (n*256.).abs() as u8;
                    // let normal = {
                    //     let mag = (ang.0*ang.0+ang.1*ang.1+1.).sqrt();
                    //     (ang.0/mag, -ang.1/mag, 1./mag)
                    // };
                    // const light:(f64, f64, f64) = (0.707107, 0.707107, 1.);
                    //l+2n.l×n
                    let slope = (ang.0*ang.0+ang.1*ang.1).sqrt().atan();
                    let ang = ang.0.atan2(ang.1);
                    let shadow = 0.5f64.cos()*slope.cos() + 0.5f64.sin()*slope.sin()*ang.cos();
                    let shadow = (256.*shadow*shadow) as u8;
                    // let color = colors::hsv_to_rgb(n*32., 0.8, 0.8);
                    // let color = colors::hsv_to_rgb(15.*n, 0.7, 0.8-p*0.5);
                    let color = colors::mm_color(n);
                    // let color = Color::WHITE;
                    let color = color * Color::rgb(shadow, shadow, shadow);
                    // let color = color * Color::rgb(shadow2, shadow2, shadow2);
                    let color = if darken == 0 {
                        color
                    } else {
                        let dark = (4-darken)*64;
                        color * Color::rgb(dark, dark, dark)
                    };
                    // if n.fract() <= 0.03 {Color::BLACK} else
                    // let normal = 0.;
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

        if config.debug {
            let txt = format!("pos: {} + {}i\nzoom: 2^{}\niter max: {}\n", config.offset.re, config.offset.im, config.zoom.log2(), config.iter_max);

            let mut text = sfml::graphics::Text::new(&txt, &fira, 24);
            text.set_outline_thickness(2.);
            text.set_position(sfml::system::Vector2::<f32>{x: 24., y: 24.});
            app.draw(&text);
        }

        app.display();

    }
}
