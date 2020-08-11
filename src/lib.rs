#![recursion_limit = "512"]

use std::f64::consts::*;
use std::time::Duration;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::{format::Json, html, Component, ComponentLink, Html, Properties, ShouldRender};
use std::collections::{HashSet, HashMap};

#[derive(Debug)]
pub struct State {
    link: ComponentLink<Self>,
    counter: usize,
    interval_task: IntervalTask,
    clicked: HashSet<usize>,
    // process_data: 
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Msg {
    Tick,
    Click(usize),
}

impl Component for State {
    type Message = Msg;
    type Properties = ();

    fn create(arg: Self::Properties, link: ComponentLink<Self>) -> Self {
        let one_second = Duration::new(0, 16000);
        let interval_task = IntervalService::new().spawn(one_second, link.callback(|()| Msg::Tick));
        State {
            interval_task,
            counter: 0,
            link,
            clicked: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.counter = (self.counter + 1) % 500;
                // redraw
                true
            }
            Msg::Click(id) => {
                if self.clicked.contains(&id) {
                    self.clicked.remove(&id);
                } else {
                    self.clicked.insert(id);
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        let mut iso = Isometric::new();
        iso.scale3d(7.0, 7.0, 7.0);

        let t = self.counter as f64 / 500.0;
        let t = t * 2.0;
        let t = (-1.0 + t).abs();

        // let cube = mk_cube(&mut iso, t, 0.0, 0.0, 0.0);

        let grid_size = 10;

        let scale = 2.4;

        let mut cubes = Vec::new();

        let mut entry = 0;

        for x in (-grid_size..grid_size) {
            for y in (-grid_size..grid_size) {
                // why is this here
                let d = distance_manhattan(x, y);
                if (d < 4) {

                    let x = x as f64;
                    let y = y as f64;

                    let dist_cart = distance_cartesian(x, y);
                    let cubic_input = 0.0_f64.max(1.0_f64.min((t * 4.8) - (dist_cart / 5.0)));
                    // TODO: figure out what is causing jitter in cubic_input
                    let te = cubic_input; //cubic_in_out(cubic_input);

                    //if (Math.abs(x) >= innergridsize || Math.abs(y) >= innergridsize) continue;
                    //elems += 1;

                    // I think this is a faithful translation? lmao idk
                    let dd = if d & 1 > 0 { 1.0 } else { -1.0 };

                    let fake_data = 0.7;
                    // let fake_data = 0.5 + ((entry as f64) % 3.0);

                    let cube = mk_cube(
                        &mut iso,
                        dd * (FRAC_PI_4 - te * FRAC_PI_2),
                        fake_data, // TODO: actual data source goes here
                        x * scale * 1.42,
                        y * scale * 1.42,
                        0.0,
                        //te * 0.5,
                    );

                    cubes.push((cube, format!("{}", entry), entry));
                    entry += 1;
                } else {
                }
            }
        }


        html!{
            <div>
            <svg viewBox="-100 -75 200 150" xmlns="http://www.w3.org/2000/svg">
            { for cubes.into_iter().rev().map(|(cube, t, entry)| {
                // let t = "foo";

                self.render_cube(cube, t.to_string(), entry)
            })
            }
            </svg>
            <text>{ format!("t = {:.3}", t) } </text>
            </div>
        }
    }
}

impl State {
    fn render_cube(&self, cube: Cube, text: String, id: usize) -> Html {
        let outer_path = path_to_points(cube.outer_path);
        let inner_path = path_to_points(cube.inner_path);
        let base_outer_path = path_to_points(cube.base_outer_path);
        let color = if self.clicked.contains(&id) { "#d33682" } else { "#ffffff" };

        html!{
            <g>
            <polygon
                points=base_outer_path fill="#002b36" stroke="black" stroke-width="0.5" opacity="0.7"
                />

            <polygon
                points=outer_path fill=color stroke="black" stroke-width="0.5" opacity="0.7"
            onclick=self.link.callback( move |_| Msg::Click(id) )
            />
            <polyline
                points=inner_path fill="none" stroke="black" stroke-width="0.25" opacity="0.7"
                onclick=self.link.callback( move |_| Msg::Click(id) )
            />
            <text x=cube.text_anchor.x y=cube.text_anchor.y font-size="2" text-anchor="middle">{ text }</text>
        </g>
        }
    }
}

struct Point {
    x: f64,
    y: f64,
}

fn path_to_points(path: Vec<Point>) -> String {
    let mut s = String::from("");
    for point in path {
        s.push_str(&format!("{:.2},{:.2} ", point.x, point.y));
    }
    s
}

struct Cube {
    inner_path: Vec<Point>,
    outer_path: Vec<Point>,
    base_outer_path: Vec<Point>,
    text_anchor: Point,
}

fn mk_cube(iso: &mut Isometric, angle: f64, height: f64, x: f64, y: f64, z: f64) -> Cube {
    let mut angle = angle % FRAC_PI_2;
    if (angle < 0.0) {
        angle += FRAC_PI_2;
    };
    iso.save();
    iso.translate3d(x, y, z);
    iso.rotate_z(angle - FRAC_PI_4);

    //context.fillStyle = fill;
    //context.globalAlpha = alpha;

    let cube_dim = 0.8;

    //11.39,9.20
    //12.94,5.90
    //7.06,5.90
    //7.06,8.30 <- problem, should be, like, indented?
    //7.06,11.70
    //12.94,11.70

    let mut outer_path = Vec::new();
    outer_path.push(iso.transform(cube_dim, -cube_dim, height));
    outer_path.push(iso.transform(cube_dim, cube_dim, height));
    outer_path.push(iso.transform(-cube_dim, cube_dim, height));
    outer_path.push(iso.transform(-cube_dim, cube_dim, 0.0));
    outer_path.push(iso.transform(-cube_dim, -cube_dim, 0.0));
    outer_path.push(iso.transform(cube_dim, -cube_dim, 0.0));

    let mut base_outer_path = Vec::new();
    base_outer_path.push(iso.transform(cube_dim, -cube_dim, 0.0));
    base_outer_path.push(iso.transform(cube_dim, cube_dim, 0.0));
    base_outer_path.push(iso.transform(-cube_dim, cube_dim, 0.0));
    base_outer_path.push(iso.transform(-cube_dim, cube_dim, -0.2));
    base_outer_path.push(iso.transform(-cube_dim, -cube_dim, -0.2));
    base_outer_path.push(iso.transform(cube_dim, -cube_dim, -0.2));

    //iso.closePath();
    //context.fill();
    //context.lineWidth = 1.5;
    //context.stroke();

    //context.beginPath();
    let mut inner_path = Vec::new();
    inner_path.push(iso.transform(-cube_dim, -cube_dim, height));
    inner_path.push(iso.transform(cube_dim, -cube_dim, height));
    inner_path.push(iso.transform(-cube_dim, -cube_dim, height));
    inner_path.push(iso.transform(-cube_dim, cube_dim, height));
    inner_path.push(iso.transform(-cube_dim, -cube_dim, height));
    inner_path.push(iso.transform(-cube_dim, -cube_dim, 0.0));
    //context.lineWidth = 0.75;
    //context.stroke();

    // if (t) {
    //   context.fillStyle = "#002b36";
    //   context.textAlign = "center";
    //   context.font = '16px serif';
    //   iso.text(0, 0, +h, t);
    //   context.fillStyle = fill;
    // }

    let text_anchor = iso.transform(0.0,0.0,cube_dim);

    iso.restore();

    Cube {
        inner_path,
        outer_path,
        base_outer_path,
        text_anchor,
    }
}

fn distance_cartesian(x: f64, y: f64) -> f64 {
    return (x * x + y * y).sqrt();
}

// distance from (0,0), I think
// TODO: might be usize?
fn distance_manhattan(x: isize, y: isize) -> isize {
    x.abs() + y.abs()
}

// ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2
fn cubic_in_out(t: f64) -> f64 {
    let t = t * 2.0;
    let x = if (t <= 1.0) {
        t * t * t
    } else {
        let t = t - 2.0;
        (t * t * t + 2.0) / 2.0
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

        Point { x, y }
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
    fn rotate_z(&mut self, angle: f64) {
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
