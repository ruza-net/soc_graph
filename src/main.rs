use rui::{canvas, rui, state, vger::Color, GestureState, Key, LocalRect, Modifiers, Vger};

#[derive(Debug, Clone)]
struct Points {
    it: usize,
    drag: bool,
    pts: Vec<[f32; 2]>,
    connections: Vec<Vec<usize>>,
    active: Option<[f32; 2]>,
}
impl Default for Points {
    fn default() -> Self {
        Self {
            it: 1,
            drag: false,
            pts: vec![],
            connections: vec![],
            active: None,
        }
    }
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let x = a[0] - b[0];
    let y = a[1] - b[1];
    (x * x + y * y).sqrt()
}

const MAX_DIST: f32 = 10.0;

fn move_pt(state: &mut Points, pos: [f32; 2], gesture: GestureState) {
    match gesture {
        GestureState::Began => {
            state.drag = true;
            println!("{:?}", state.pts);
            for row in &state.connections {
                println!(
                    "{}",
                    row.iter().map(|c| c.to_string() + " ").collect::<String>()
                );
            }
            println!();

            let nearest = state
                .pts
                .iter()
                .map(|&p| dist(p, pos))
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap_or((0, MAX_DIST));

            if nearest.1 >= MAX_DIST {
                state.active = Some(pos);
            } else {
                let idx = nearest.0;
                let pt = state.pts.remove(idx);
                state.active = Some(pt);
                state.connections.remove(idx);
                for row in &mut state.connections {
                    row.remove(idx);
                }
            }
        }
        GestureState::Changed => {
            let pt = state.active.as_mut().unwrap();
            *pt = pos;
        }
        GestureState::Ended => {
            state.pts.push(state.active.take().unwrap());
            for row in &mut state.connections {
                row.fill(0);
                row.push(0);
            }
            let len = state.pts.len();
            state.connections.push(vec![0; len]);

            state.it = 0;
            state.drag = false;
        }
    }
}

fn draw_pt(state: &mut Points, vger: &mut Vger) {
    let cyan = vger.color_paint(Color::CYAN);
    let pt_count = state.pts.len();
    for i in 0..pt_count {
        let a = state.pts[i];
        vger.fill_circle(a, 5.0, cyan);

        for j in 0..pt_count {
            let shade = state.connections[i][j] as f32 / state.it as f32 / 2.0;
            let color = vger.color_paint(Color::CYAN.alpha(shade));

            let b = state.pts[j];
            vger.stroke_segment(a, b, 3.0, color);
        }
    }
    if let Some(pt) = state.active {
        let magenta = vger.color_paint(Color::MAGENTA);
        vger.fill_circle(pt, 5.0, magenta);
    }

    if !state.drag {
        simulate(state);
    }
}

fn transition(d: f32, z: usize) -> f64 {
    let d = d as f64;
    (z + 1) as f64 / d / d
}

fn simulate(state: &mut Points) {
    let pt_count = state.pts.len();

    let mut conn_delta = vec![vec![0; pt_count]; pt_count];
    for i in 0..pt_count {
        let mut normalizer = 0.0;
        for j in 0..pt_count {
            if j == i {
                continue;
            }
            let d = dist(state.pts[j], state.pts[i]);
            let z = state.connections[i][j];
            normalizer += transition(d, z);
        }

        for j in 0..pt_count {
            if j == i {
                continue;
            }
            let d = dist(state.pts[i], state.pts[j]);
            let z = state.connections[i][j];
            let val = transition(d, z);

            let prob = val / normalizer;
            if rand::random_bool(prob) {
                conn_delta[i][j] += 1;
            }
        }
    }
    for i in 0..pt_count {
        for j in 0..pt_count {
            state.connections[i][j] += conn_delta[i][j];
        }
    }
    state.it += 1;
}

fn main() {
    rui(state(Points::default, move |pts, _| {
        canvas(move |ctx, _rect, vger| {
            draw_pt(&mut ctx[pts], vger);
        })
        .drag_p(move |ctx, pos, gesture, btn| {
            if btn.is_none() || btn == Some(rui::MouseButton::Left) {
                move_pt(&mut ctx[pts], pos.into(), gesture);
            }
        })
        .anim(move |ctx, _| simulate(&mut ctx[pts]))
    }))
}
