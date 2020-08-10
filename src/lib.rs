#![recursion_limit = "512"]

use std::f64::consts::*;
use std::time::Duration;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::{format::Json, html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Debug)]
pub struct State {
    link: ComponentLink<Self>,
    counter: usize,
    interval_task: IntervalTask,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Msg {
    Tick,
    Click,
}

impl Component for State {
    type Message = Msg;
    type Properties = ();

    fn create(arg: Self::Properties, link: ComponentLink<Self>) -> Self {
        let one_second = Duration::new(0, 1000);
        let interval_task = IntervalService::new().spawn(one_second, link.callback(|()| Msg::Tick));
        State {
            interval_task,
            counter: 0,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.counter = (self.counter + 1) % 1000;

                // redraw
                true
            }
            _ => {
                // TODO: handle click events
                false
            }
        }
    }

    fn view(&self) -> Html {
        let mut iso = Isometric::new();
        iso.scale3d(3.0, 3.0, 3.0);


        let t = self.counter as f64 / 1000.0;
        let t = t * PI;

        let cube = mkCube(&mut iso, t, 0.0, 0.0, 0.0);
        let outer_path = path_to_points(cube.outer_path);
        html! {
            <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
            <polygon points=outer_path
            fill="none" stroke="black" />
            </svg>
        }
    }
}


struct Point{
    x: f64,
    y: f64,
}

fn path_to_points(path: Vec<Point>) -> String {
    let mut s = String::from("");
    for point in path {
        s.push_str(&format!("{:.2},{:.2} ", point.x + 10.0, point.y + 10.0));
    }
    s
}


struct Cube {
    inner_path: Vec<Point>,
    outer_path: Vec<Point>,
}

fn mkCube(iso: &mut Isometric, angle: f64, x: f64, y: f64, z: f64) -> Cube {
    let mut angle = angle % FRAC_PI_2;
    if (angle  < 0.0) {
        angle += FRAC_PI_2;
    };
    iso.save();
    iso.translate3d(x, y, z);
    iso.rotateZ(angle - FRAC_PI_4);

    //context.fillStyle = fill;
    //context.globalAlpha = alpha;

    let cubeDim = 0.8;
    let h = cubeDim;

    //11.39,9.20
    //12.94,5.90
    //7.06,5.90
    //7.06,8.30 <- problem, should be, like, indented?
    //7.06,11.70
    //12.94,11.70

    let mut outer_path = Vec::new();

    outer_path.push(iso.transform(cubeDim, -cubeDim, cubeDim));
    outer_path.push(iso.transform(cubeDim, cubeDim, cubeDim));
    outer_path.push(iso.transform(-cubeDim, cubeDim, cubeDim));
    outer_path.push(iso.transform(-cubeDim, cubeDim, -cubeDim));
    outer_path.push(iso.transform(-cubeDim, -cubeDim, -cubeDim));
    outer_path.push(iso.transform(cubeDim, -cubeDim, -cubeDim));

    //iso.closePath();
    //context.fill();
    //context.lineWidth = 1.5;
    //context.stroke();

    //context.beginPath();
    let mut inner_path = Vec::new();
    inner_path.push(iso.transform(-cubeDim, -cubeDim, cubeDim));
    inner_path.push(iso.transform(cubeDim, -cubeDim, cubeDim));
    inner_path.push(iso.transform(-cubeDim, -cubeDim, cubeDim));
    inner_path.push(iso.transform(-cubeDim, cubeDim, cubeDim));
    inner_path.push(iso.transform(-cubeDim, -cubeDim, cubeDim));
    inner_path.push(iso.transform(-cubeDim, -cubeDim, -cubeDim));
    //context.lineWidth = 0.75;
    //context.stroke();

    // if (t) {
    //   context.fillStyle = "#002b36";
    //   context.textAlign = "center";
    //   context.font = '16px serif';
    //   iso.text(0, 0, +h, t);
    //   context.fillStyle = fill;
    // }

    iso.restore();

    Cube{ inner_path, outer_path}
  }


fn distanceCartesian(x: f64, y: f64) -> f64 {
    return (x * x + y * y).sqrt();
}

// distance from (0,0), I think
// TODO: might be usize?
fn distanceManhattan(x: isize, y: isize) -> isize {
    x.abs() + y.abs()
}

// ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2
fn cubicInOut(t: f64) -> f64 {
    let t = t * 2.0;
    let x = if (t <= 1.0) {
        t * t * t
    } else {
        let t2 = t - 2.0;
        (t2 * t * t + 2.0) / 2.0
    };
    x
}

struct Isometric {
    matrix: [f64; 12],
    matrixes: Vec<[f64; 12]>,
    projection: [f64; 4],
}

impl Isometric {
    fn new() -> Self {
        let matrix = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let matrixes = Vec::new();
        let projection = [
            FRAC_PI_6.cos(),
            (PI - FRAC_PI_6).cos(),
            -1.0 * FRAC_PI_6.sin(),
            -1.0 * (PI - FRAC_PI_6).sin(),
        ];
        Self {
            matrix,
            matrixes,
            projection,
        }
    }

    // not an entry point
    fn project(&self, x3: f64, y3: f64, z3: f64) -> Point {
        let x = x3 * self.projection[0] + y3 * self.projection[1];
        let y = x3 * self.projection[2] + y3 * self.projection[3] - z3;

        Point {
            x, y
        }
    }

    // seems to take points relative to current iso - so +cube height, -cube width, etc
    fn transform(&self, x: f64, y: f64, z: f64) -> Point {
        self.project(
            x * self.matrix[0] + y * self.matrix[1] + z * self.matrix[2] + self.matrix[3],
            x * self.matrix[4] + y * self.matrix[5] + z * self.matrix[6] + self.matrix[7],
            x * self.matrix[8] + y * self.matrix[9] + z * self.matrix[10] + self.matrix[11],
        )
    }

    fn save(&mut self) {
        self.matrixes.push(self.matrix);
    }

    fn restore(&mut self) {
        match self.matrixes.pop() {
            Some(m) => {
                self.matrix = m;
            }
            None => {}
        }
    }

    // | a b c d |   | kx  0  0 0 |   | a * kx b * ky c * kz d |
    // | e f g h | * |  0 ky  0 0 | = | e * kx f * ky g * kz h |
    // | i j k l |   |  0  0 kz 0 |   | i * kx j * ky k * kz l |
    // | 0 0 0 1 |   |  0  0  0 1 |   |      0      0      0 1 |
    fn scale3d(&mut self, kx: f64, ky: f64, kz: f64) {
        self.matrix[0] *= kx;
        self.matrix[1] *= ky;
        self.matrix[2] *= kz;
        self.matrix[4] *= kx;
        self.matrix[5] *= ky;
        self.matrix[6] *= kz;
        self.matrix[8] *= kx;
        self.matrix[9] *= ky;
        self.matrix[10] *= kz;
    }

    // | a b c d |   | cos -sin 0 0 |   | a * cos + b * sin a * -sin + b * cos c d |
    // | e f g h | * | sin  cos 0 0 | = | e * cos + f * sin e * -sin + f * cos g h |
    // | i j k l |   |   0    0 1 0 |   | i * cos + j * sin i * -sin + j * cos k l |
    // | 0 0 0 1 |   |   0    0 0 1 |   |                 0                  0 0 1 |
    fn rotateZ(&mut self, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let a = self.matrix[0];
        let b = self.matrix[1];
        let e = self.matrix[4];
        let f = self.matrix[5];
        let i = self.matrix[8];
        let j = self.matrix[9];
        self.matrix[0] = a * cos + b * sin;
        self.matrix[1] = a * -sin + b * cos;
        self.matrix[4] = e * cos + f * sin;
        self.matrix[5] = e * -sin + f * cos;
        self.matrix[8] = i * cos + j * sin;
        self.matrix[9] = i * -sin + j * cos;
    }

    // | a b c d |   | 1 0 0 tx |   | a b c a * tx + b * ty + c * tz + d |
    // | e f g h | * | 0 1 0 ty | = | e f g e * tx + f * ty + g * tz + h |
    // | i j k l |   | 0 0 1 tz |   | i j k i * tx + j * ty + k * tz + l |
    // | 0 0 0 1 |   | 0 0 0  1 |   | 0 0 0                            1 |
    fn translate3d(&mut self, tx: f64, ty: f64, tz: f64) {
        self.matrix[3] += self.matrix[0] * tx + self.matrix[1] * ty + self.matrix[2] * tz;
        self.matrix[7] += self.matrix[4] * tx + self.matrix[5] * ty + self.matrix[6] * tz;
        self.matrix[11] += self.matrix[8] * tx + self.matrix[9] * ty + self.matrix[10] * tz;
    }
}

// constructor(context) {
//   //this._moveTo = context.moveTo.bind(context);
//   //this._lineTo = context.lineTo.bind(context);
//   //this._text = context.fillText.bind(context);
//   //this.closePath = context.closePath.bind(context);
//   this._matrix = [
//     1, 0, 0, 0,
//     0, 1, 0, 0,
//     0, 0, 1, 0
//   ];
//   this._matrixes = [];
//     this._projection = [
//     Math.cos(Math.PI / 6), Math.cos(Math.PI - Math.PI / 6),
//     -Math.sin(Math.PI / 6), -Math.sin(Math.PI - Math.PI / 6)
//   ];
// }

// _project(point, x, y, z) {
//   point(
//     x * this._projection[0] + y * this._projection[1],
//     x * this._projection[2] + y * this._projection[3] - z
//   );
// }
// _transform(point, x, y, z) {
//   this._project(
//     point,
//     x * this._matrix[0] + y * this._matrix[1] + z * this._matrix[2] + this._matrix[3],
//     x * this._matrix[4] + y * this._matrix[5] + z * this._matrix[6] + this._matrix[7],
//     x * this._matrix[8] + y * this._matrix[9] + z * this._matrix[10] + this._matrix[11]
//   );
// }

// save() {
//   this._matrixes.push(this._matrix.slice());
// }
// restore() {
//   if (this._matrixes.length) this._matrix = this._matrixes.pop();
// }

// // | a b c d |   | kx  0  0 0 |   | a * kx b * ky c * kz d |
// // | e f g h | * |  0 ky  0 0 | = | e * kx f * ky g * kz h |
// // | i j k l |   |  0  0 kz 0 |   | i * kx j * ky k * kz l |
// // | 0 0 0 1 |   |  0  0  0 1 |   |      0      0      0 1 |
// scale3d(kx, ky, kz) {
//   this._matrix[0] *= kx;
//   this._matrix[1] *= ky;
//   this._matrix[2] *= kz;
//   this._matrix[4] *= kx;
//   this._matrix[5] *= ky;
//   this._matrix[6] *= kz;
//   this._matrix[8] *= kx;
//   this._matrix[9] *= ky;
//   this._matrix[10] *= kz;
// }

// // | a b c d |   | cos -sin 0 0 |   | a * cos + b * sin a * -sin + b * cos c d |
// // | e f g h | * | sin  cos 0 0 | = | e * cos + f * sin e * -sin + f * cos g h |
// // | i j k l |   |   0    0 1 0 |   | i * cos + j * sin i * -sin + j * cos k l |
// // | 0 0 0 1 |   |   0    0 0 1 |   |                 0                  0 0 1 |
// rotateZ(angle) {
//   const cos = Math.cos(angle);
//   const sin = Math.sin(angle);
//   const a = this._matrix[0];
//   const b = this._matrix[1];
//   const e = this._matrix[4];
//   const f = this._matrix[5];
//   const i = this._matrix[8];
//   const j = this._matrix[9];
//   this._matrix[0] = a * cos + b * sin;
//   this._matrix[1] = a * -sin + b * cos;
//   this._matrix[4] = e * cos + f * sin;
//   this._matrix[5] = e * -sin + f * cos;
//   this._matrix[8] = i * cos + j * sin;
//   this._matrix[9] = i * -sin + j * cos;
// }

// // | a b c d |   | 1 0 0 tx |   | a b c a * tx + b * ty + c * tz + d |
// // | e f g h | * | 0 1 0 ty | = | e f g e * tx + f * ty + g * tz + h |
// // | i j k l |   | 0 0 1 tz |   | i j k i * tx + j * ty + k * tz + l |
// // | 0 0 0 1 |   | 0 0 0  1 |   | 0 0 0                            1 |
// translate3d(tx, ty, tz) {
//   this._matrix[3] += this._matrix[0] * tx + this._matrix[1] * ty + this._matrix[2] * tz;
//   this._matrix[7] += this._matrix[4] * tx + this._matrix[5] * ty + this._matrix[6] * tz;
//   this._matrix[11] += this._matrix[8] * tx + this._matrix[9] * ty + this._matrix[10] * tz;
// }

//   text(x, y, z, t) {
//     var _text = this._text;
//     var f = function(x,y) {
//       _text(t,x,y);
//     };
//     this._transform(f, x, y, z);
//   }

//   moveTo(x, y, z) {
//     this._transform(this._moveTo, x, y, z);
//   }
//   lineTo(x, y, z) {
//     this._transform(this._lineTo, x, y, z);
//   }
// }
