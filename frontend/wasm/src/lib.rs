#![recursion_limit = "512"]

use std::f32::consts::*;
use std::time::Duration;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::{format::Json, html, Component, ComponentLink, Html, ShouldRender};
use std::collections::{HashSet, HashMap};
use shared_types::{ProcessName, ProcessInfo, UpdateResp};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use stdweb::js;


macro_rules! println {
    ($($tt:tt)*) => {{
        let msg = format!($($tt)*);
        js! { @(no_return) console.log(@{ msg }) }
    }}
}

#[derive(Debug)]
pub struct State {
    link: ComponentLink<Self>,
    update_interval_task: IntervalTask,
    animation_interval_task: Option<IntervalTask>,
    animation_counter: usize,
    process_map: HashMap<ProcessName, Vec<ProcessInfo>>,
    focus: Option<ProcessName>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Tick,
    Update,
    HandleUpdateResp(UpdateResp),
    Click(ProcessName),
}

impl Component for State {
    type Message = Msg;
    type Properties = ();

    fn create(arg: Self::Properties, link: ComponentLink<Self>) -> Self {
        let update_interval_task = IntervalService::new().spawn(Duration::new(5, 0), link.callback(|()| Msg::Update));

        let mut fetch_service = FetchService::new();
        let init = fetch(&mut fetch_service, &link);
        State {
            update_interval_task,
            animation_interval_task: None,
            animation_counter: 0,
            link,
            focus: None,
            process_map: HashMap::new(),
            fetch_service,
            fetch_task: Some(init),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.animation_counter -= 1;
                if self.animation_counter == 0 {
                    self.animation_interval_task = None;
                }

                // redraw
                true
            }
            Msg::Update => {
                let task = fetch(&mut self.fetch_service, &self.link);
                self.fetch_task = Some(task);

                // no need to redraw
                false
            }
            Msg::HandleUpdateResp(resp) => {
                self.fetch_task = None;
                self.process_map = resp.process_map;
                let animation_interval_task = IntervalService::new()
                    .spawn(Duration::new(0, 32000), self.link.callback(|()| Msg::Tick));
                self.animation_interval_task = Some(animation_interval_task);
                self.animation_counter = 15;
                true
            }
            Msg::Click(name) => {
                if let Some(prev_focus) = &self.focus {
                    if prev_focus == &name {
                        self.focus = None;
                    } else {
                        self.focus = Some(name);
                    }
                } else {
                    self.focus = Some(name);
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        let mut iso = Isometric::new();
        iso.scale3d(7.0, 7.0, 7.0);

        let t = self.animation_counter as f32 / 15.0;
        let t = t * FRAC_PI_2 + FRAC_PI_4;
        // let t = t * 2.0;
        // let t = (-1.0 + t).abs();
        // let t = 0.5;

        let grid_size = 10;

        let scale = 2.4;

        let mut cubes = Vec::new();

        let mut processes: Vec<(ProcessName, Vec<ProcessInfo>)> = self.process_map.clone().into_iter().collect();
        processes.sort_by(|(_, a), (_, b)| {
            let a: f32 = a.iter().map( |p| p.mem_percent ).sum();
            let b: f32 = b.iter().map( |p| p.mem_percent ).sum();
            b.partial_cmp(&a).unwrap()
        });

        let mut process_info_iter = processes.into_iter().take(25).rev();

        // FIXME: figure out proper coordinate system - currently just using manhattan + iter hax
        for x in -grid_size..grid_size {
            for y in -grid_size..grid_size {
                // why is this here
                let d = distance_manhattan(x, y);
                if d < 4 {

                    let x = x as f32;
                    let y = y as f32;

                    if let Some((name, process_infos)) = process_info_iter.next() {
                        // let dist_cart = distance_cartesian(x, y);
                        // let cubic_input = 0.0_f32.max(1.0_f32.min((t * 4.8) - (dist_cart / 5.0)));
                        // // TODO: figure out what is causing jitter in cubic_input
                        // let te = cubic_input; //cubic_in_out(cubic_input);

                        //if (Math.abs(x) >= innergridsize || Math.abs(y) >= innergridsize) continue;
                        //elems += 1;

                        // I think this is a faithful translation? lmao idk
                        // let dd = if d & 1 > 0 { 1.0 } else { -1.0 };

                        let mem_percents = process_infos.iter().map( |p| p.mem_percent as f32 ).collect();

                        let cube = mk_cube(
                            &mut iso,
                            t,
                            x * scale * 1.42,
                            y * scale * 1.42,
                            0.0,
                            mem_percents,
                            //te * 0.5,
                        );


                        cubes.push((cube, name));
                    }
                } else {
                }
            }
        }


        html!{
            <div>
            <svg viewBox="-100 -75 200 150" xmlns="http://www.w3.org/2000/svg">
            { for cubes.into_iter().rev().map(|(cube, t)| {
                // let t = "foo";

                self.render_cube(cube, t.to_string())
            })
            }
            </svg>
            <text>{ format!("t = {:.3}", t) } </text>
            </div>
        }
    }
}

impl State {
    fn render_cube(&self, cube: Cube, process_name: String) -> Html {
        let outer_path = path_to_points(cube.outer_path);
        let inner_paths: Vec<String> = cube.inner_paths.into_iter().map(path_to_points).collect();
        let base_outer_path = path_to_points(cube.base_outer_path);
        let color = if self.focus == Some(process_name.clone()) { "#d33682" } else { "#ffffff" };
        let process_name_2 = process_name.clone();

        html!{
            <g onclick=self.link.callback( move |_| Msg::Click(process_name.clone()) )>
            <polygon points=base_outer_path fill="#002b36" stroke="black" stroke-width="0.5" opacity="0.7"/>
            <polygon points=outer_path fill=color stroke="black" stroke-width="0.5" opacity="0.7"/>
            {
                for inner_paths.into_iter().map(|path| {
                    html!{
                        <polyline points=path fill="none" stroke="black" stroke-width="0.25" opacity="0.7"/>
                    }
                })

            }
            <text x=cube.text_anchor.x y=cube.text_anchor.y font-size="2" text-anchor="middle">{ process_name_2 }</text>
        </g>
        }
    }
}

fn fetch(fetch_service: &mut FetchService, link: &ComponentLink<State>) -> FetchTask {
    let request = Request::get("/processes")
        .body(yew::format::Nothing)
        .expect("failed building get req");

    let callback = link.callback(
        move |response: Response<Json<Result<UpdateResp, anyhow::Error>>>| {
            println!("got resp");
            let (meta, Json(res)) = response.into_parts();
            let body = res.expect("failure parsing resp body");
            println!("parsed resp");
            if meta.status.is_success() {
                println!("resp is 200: {:?}", body);
                Msg::HandleUpdateResp(body)
            } else {
                panic!("non-200 resp, {:?}", meta.status)
            }
        },
    );

    let task = fetch_service.fetch(request, callback).expect("creating task failed");

    task
}

struct Point {
    x: f32,
    y: f32,
}

fn path_to_points(path: Vec<Point>) -> String {
    let mut s = String::from("");
    for point in path {
        s.push_str(&format!("{:.2},{:.2} ", point.x, point.y));
    }
    s
}

struct Cube {
    inner_paths: Vec<Vec<Point>>,
    outer_path: Vec<Point>,
    base_outer_path: Vec<Point>,
    text_anchor: Point,
}

/// h_percents - nonempty (?) vector of percents, used to determine total height
/// idea is to have multiple segements
fn mk_cube(iso: &mut Isometric, angle: f32, x: f32, y: f32, z: f32, h_percents: Vec<f32>) -> Cube {
    let mut angle = angle % FRAC_PI_2;
    if angle < 0.0 {
        angle += FRAC_PI_2;
    };
    iso.save();
    iso.translate3d(x, y, z);
    iso.rotate_z(angle - FRAC_PI_4);

    let cube_dim = 0.8;
    let total_height: f32 = h_percents.iter().sum();
    let scaled_height: f32 = 12.0 * cube_dim * total_height;
    let scaled_height = (1.0 + scaled_height).log10();

    let height_scaling_factor = scaled_height / total_height;

    // FIXME: HAX?
    // let height_scaling_factor: f32 = 12.0 * cube_dim * total_height;
    // let height_scaling_factor = height_scaling_factor.log10();
    // let height_scaling_factor = total_height / height_scaling_factor;
    // iso.scale3d(1.0, 1.0, height_scaling_factor);

    // TODO: scale via scale3d(1.0, 1.0, scaling_factor) instead

    let mut outer_path = Vec::new();
    outer_path.push(iso.transform(cube_dim, -cube_dim, scaled_height));
    outer_path.push(iso.transform(cube_dim, cube_dim, scaled_height));
    outer_path.push(iso.transform(-cube_dim, cube_dim, scaled_height));
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
    let mut inner_paths = Vec::new();

    let mut h_running_sum = 0.0;
    for h in h_percents.iter() {
        h_running_sum += h * height_scaling_factor;
        let mut inner_path = Vec::new();
        inner_path.push(iso.transform(-cube_dim, -cube_dim, h_running_sum));
        inner_path.push(iso.transform(cube_dim, -cube_dim, h_running_sum));
        inner_paths.push(inner_path);

        let mut inner_path = Vec::new();
        inner_path.push(iso.transform(-cube_dim, -cube_dim, h_running_sum));
        inner_path.push(iso.transform(-cube_dim, cube_dim, h_running_sum));
        inner_paths.push(inner_path);
    }

    // singular vertical line
    let mut inner_path = Vec::new();
    inner_path.push(iso.transform(-cube_dim, -cube_dim, scaled_height));
    inner_path.push(iso.transform(-cube_dim, -cube_dim, 0.0));
    inner_paths.push(inner_path);

    let text_anchor = iso.transform(0.0,0.0,scaled_height);

    iso.restore();

    Cube {
        inner_paths,
        outer_path,
        base_outer_path,
        text_anchor,
    }
}

fn distance_cartesian(x: f32, y: f32) -> f32 {
    return (x * x + y * y).sqrt();
}

// distance from (0,0), I think
// TODO: might be usize?
fn distance_manhattan(x: isize, y: isize) -> isize {
    x.abs() + y.abs()
}

// ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2
fn cubic_in_out(t: f32) -> f32 {
    let t = t * 2.0;
    let x = if t <= 1.0 {
        t * t * t
    } else {
        let t = t - 2.0;
        (t * t * t + 2.0) / 2.0
    };
    x
}

struct Isometric {
    matrix: [f32; 12],
    matrixes: Vec<[f32; 12]>,
    projection: [f32; 4],
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
    fn project(&self, x3: f32, y3: f32, z3: f32) -> Point {
        let x = x3 * self.projection[0] + y3 * self.projection[1];
        let y = x3 * self.projection[2] + y3 * self.projection[3] - z3;

        Point { x, y }
    }

    // seems to take points relative to current iso - so +cube height, -cube width, etc
    fn transform(&self, x: f32, y: f32, z: f32) -> Point {
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
    fn scale3d(&mut self, kx: f32, ky: f32, kz: f32) {
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
    fn rotate_z(&mut self, angle: f32) {
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
    fn translate3d(&mut self, tx: f32, ty: f32, tz: f32) {
        self.matrix[3] += self.matrix[0] * tx + self.matrix[1] * ty + self.matrix[2] * tz;
        self.matrix[7] += self.matrix[4] * tx + self.matrix[5] * ty + self.matrix[6] * tz;
        self.matrix[11] += self.matrix[8] * tx + self.matrix[9] * ty + self.matrix[10] * tz;
    }
}
