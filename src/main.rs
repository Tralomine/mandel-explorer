use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use std::f64::consts::SQRT_2;
// use std::f64::consts::PI;

use sfml;
use sfml::system::Vector2f;
use sfml::window::{Key, Event, mouse};
use sfml::graphics::{Color, Image, Rect, RectangleShape, RenderTarget, RenderTexture, Shape, Sprite, Texture, Transformable};

pub mod mandel;
use mandel::cplx::{self, Cplx};
use mandel::Mandel;

// use mandel::big_float;

pub mod colors;

fn pos_to_cplx(x:i32, y:i32, config: &Config) -> cplx::Cplx<f64> {
    // width: usize, height: usize, zoom:f64, offset: cplx::Cplx<f64>
    let min = std::cmp::min(config.size.0, config.size.1) as f64;
    cplx::Cplx{
        re: ((x-config.size.0 as i32/2) as f64)/min/config.zoom,
        im: ((y-config.size.1 as i32/2) as f64)/min/config.zoom,
    } + config.offset
}

fn cplx_to_pos(z:cplx::Cplx<f64>, config: &Config) -> Vector2f {
    let min = std::cmp::min(config.size.0, config.size.1) as f64;
    Vector2f{
        x: ((z.re - config.offset.re)*config.zoom*min) as f32 + (config.size.0/2) as f32,
        y: ((z.im - config.offset.im)*config.zoom*min) as f32 + (config.size.1/2) as f32,
    }
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
    pub redraw: bool,
    pub debug: bool,
    pub aa: usize,
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
                app.set_view(&sfml::graphics::View::from_rect(sfml::graphics::FloatRect::new(0., 0., config.size.0 as f32,  config.size.1 as f32)));
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

#[inline]
fn get_color(m: &Mandel) -> Color {
    match m.get_finished() {
        Some(n) => {
            if n.is_finite() {
                // let n = 0.5*mandel::fast_log2(n);
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
                const LIGHT:(f64, f64, f64) = (-SQRT_2, -SQRT_2, 1.);
                let normal = m.get_shadow().unwrap();
                let shadow = (LIGHT.0*normal.re + LIGHT.1*normal.im + LIGHT.2) / (1.+LIGHT.2);
                let shadow = colors::hsv_to_rgb(185., 0., 0.5+0.5*shadow);
                // let shadow = colors::hsv_to_rgb(185., 0.1*(1.-shadow), 0.75+0.25*shadow);
                // let color = colors::hsv_to_rgb(n*32., 0.8, 0.8);
                // let color = colors::hsv_to_rgb(15.*n, 0.7, 0.8-p*0.5);
                let color = colors::mm_color(n);
                // let color = Color::WHITE;
                let color = color * shadow;
                // let color = color * Color::rgb(shadow, shadow, shadow);
                // let color = color * Color::rgb(shadow2, shadow2, shadow2);
                color
            } else {
                Color::BLACK
            }
        },
        None => Color::BLACK
    }
}

fn generate_bg(pic: Image, old: Config, config: &Config) -> Image {
    // pic = Image::new((config.size.0*config.AA) as u32, (config.size.1*config.AA) as u32);

    let scale = config.zoom/old.zoom;
    let mut texture = Texture::new().unwrap();
    texture.load_from_image(&pic, Rect {left: 0, top: 0, width: (config.size.0*config.aa) as i32, height: (config.size.1*config.aa) as i32}).expect("msg");
    texture.set_smooth(true);
    let mut sprite = Sprite::with_texture(&texture);
    sprite.set_origin(((config.size.0*config.aa) as f32/2., (config.size.1*config.aa) as f32/2.));
    // sprite.scale((1./config.AA as f32, 1./config.AA as f32));

    sprite.set_position((cplx_to_pos(old.offset, &config).x*config.aa as f32, cplx_to_pos(old.offset, &config).y*config.aa as f32));
    sprite.scale((scale as f32, scale as f32));

    let mut bg = RenderTexture::new((config.size.0*config.aa) as u32, (config.size.1*config.aa) as u32).expect("rendertext");
    bg.draw(&sprite);
    let mut red = RectangleShape::new();
    red.set_fill_color(Color { r: 255, g: 0, b: 0, a: 64 });
    red.set_size(((config.size.0*config.aa) as f32, (config.size.1*config.aa) as f32));
    bg.draw(&red);
    bg.display();
    bg.texture().copy_to_image().expect("text-to-pic")
}


fn main() {
    let mut config: Config = Config{
        size: (640, 480),
        zoom: 0.25,
        offset: cplx::Cplx{re:-0.5,im:0.},
        iter_max: 256,
        redraw: true,
        debug: true,
        aa: 2,
    };

    let mut settings = sfml::window::ContextSettings::default();
    settings.antialiasing_level = 8;

    let mut app = sfml::graphics::RenderWindow::new(
        sfml::window::VideoMode::desktop_mode(),
        "mandel",
        sfml::window::Style::NONE,
        &settings,
    );
    app.set_position(sfml::system::Vector2i::new(0, 0));

    // let mut mandels = vec![vec![Mandel::new_empty();config.size.0*2];config.size.1*2];

    let mut pic = Image::new((config.size.0*config.aa) as u32, (config.size.1*config.aa) as u32);

    let fira = sfml::graphics::Font::from_file("fira.otf").unwrap();

    let mut tx_calc;
    let (_, mut rx_calc) = mpsc::channel();

    while app.is_open() {
        let frame_start = Instant::now();
        let old = config;
        process_events(&mut app, &mut config);
        let debug_txt;

        if config.redraw {
            config.redraw = false;

            (tx_calc, rx_calc) = mpsc::channel();
            config.size.0 *= config.aa;
            config.size.1 *= config.aa;
            // mandels = vec![vec![Mandel::new_empty();config.size.0];config.size.1];
            area(tx_calc.clone(), Rect{left:0, top:0, width:config.size.0, height:config.size.1}, config);
            config.size.0 /= config.aa;
            config.size.1 /= config.aa;

            pic = generate_bg(pic, old, &config);
        }

        let mut orbit: Vec<sfml::graphics::Vertex> = Vec::new();
        //calculate orbit to show
        {
            let mouse_pos = sfml::window::mouse::desktop_position() - app.position();
            let pos = pos_to_cplx(mouse_pos.x, mouse_pos.y, &config);
            let mut z = pos;
            debug_txt = format!("mouse pos: [{}, {}]", pos.re, pos.im);
            const M: f64 = 32.;
            for _ in 1..config.iter_max {
                orbit.push(sfml::graphics::Vertex{position: cplx_to_pos(z, &config), color: sfml::graphics::Color::RED,tex_coords: Vector2f{x: 0.,y: 0.}});
                if z.sq_abs() >= M * M {
                    break;
                }
                z = z.square() + pos;
            }
        }

        loop {
            match rx_calc.try_recv() {
                Ok((x, y, m)) => {
                    // mandels[x][y] = m;
                    unsafe {
                        pic.set_pixel(x as u32, y as u32, get_color(&m));
                    }
                },
                Err(_) => break,
            }
            if frame_start.elapsed() >= Duration::from_secs_f64(1./40.) {break;}
        }


        app.clear(Color::BLACK);

        let mut texture = Texture::new().unwrap();
        texture.load_from_image(&pic, Rect {left: 0, top: 0, width: (config.size.0*config.aa) as i32, height: (config.size.1*config.aa) as i32}).expect("msg");
        texture.set_smooth(true);
        let mut sprite = Sprite::with_texture(&texture);
        sprite.scale((1./config.aa as f32, 1./config.aa as f32));
        app.draw(&sprite);

        app.draw_primitives(&orbit, sfml::graphics::PrimitiveType::LINE_STRIP, &sfml::graphics::RenderStates::DEFAULT);

        if config.debug {
            let txt = format!("pos: {} + {}i\nzoom: 2^{}\niter max: {}\n{debug_txt}", config.offset.re, config.offset.im, config.zoom.log2(), config.iter_max);

            let mut text = sfml::graphics::Text::new(&txt, &fira, 24);
            text.set_outline_thickness(2.);
            text.set_position(sfml::system::Vector2::<f32>{x: 24., y: 24.});
            app.draw(&text);
        }

        app.display();

    }
}
