use std::ops::Rem;
use std::f64::consts::PI;
use sfml::graphics::Color;


pub fn mm_color(t: f64) -> Color {
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

pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Color {
    //360, 1, 1
    let h = h.rem(360.);
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

