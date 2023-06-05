use kalman_filtering_rs::{make_k, make_m, new_cov, predict, update};
use peroxide::prelude::{matrix, zeros, Matrix, Shape::Row};
use plotly::{common::Title, layout::Axis, Layout, Plot, Scatter};
use rand_distr::{Distribution, Normal};

const SIGNOISE: f64 = 304.8;
const PHIS: f64 = 10.0;
const TS: f64 = 0.1;
const INIT_S: f64 = 121920.0;
const INIT_U: f64 = -1828.8;
const G: f64 = -9.81;
const MAXT: f64 = 60.0;

fn main() {
    let data = get_data();

    let mut state = matrix(vec![0.0, 0.0, G], 3, 1, Row);

    let mut cov = zeros(3, 3);
    cov[(0, 0)] = 99999999.0;
    cov[(1, 1)] = 99999999.0;
    cov[(2, 2)] = 99999999.0;

    let h = matrix(vec![1.0, 0.0, 0.0], 1, 3, Row);
    let r = matrix(vec![SIGNOISE], 1, 1, Row);

    let mut x_history = vec![];
    let mut v_history = vec![];
    let mut a_history = vec![];

    for i in 0..data.t.len() {
        let x_star = data.x[i];

        let xkminus1 = state.data[0];
        let xdotkminus1 = state.data[1];
        let xdotdotkminus1 = state.data[2];

        let xdotdot_bar = xdotdotkminus1;
        let xdot_bar = xdotkminus1 + xdotdot_bar * TS;
        let x_bar = xkminus1 + xdot_bar * TS + 0.5 * xdotdot_bar * TS.powf(2.0);

        let phi = phi(TS);
        let q = q(TS);
        let m = make_m(&phi, &cov, &q);
        let k = make_k(&m, &h, &r);

        let x_tilda = x_star - x_bar;

        let k1 = k[(0, 0)];
        let k2 = k[(1, 0)];
        let k3 = k[(2, 0)];

        let x_hat = x_bar + k1 * x_tilda;
        let xdot_hat = xdot_bar + k2 * x_tilda;
        let xdotdot_hat = xdotdot_bar + k3 * x_tilda;

        state = matrix(vec![x_hat, xdot_hat, xdotdot_hat], 3, 1, Row);
        cov = new_cov(&k, &h, &m);

        x_history.push(state.data[0]);
        v_history.push(state.data[1]);
        a_history.push(state.data[2]);
    }

    let mut plot = Plot::new();
    let m_trace = Scatter::new(data.t.clone(), data.s.clone()).name("Truth");
    let x_trace = Scatter::new(data.t.clone(), x_history).name("Filter");
    let s_trace = Scatter::new(data.t.clone(), data.x.clone()).name("Measurements");
    plot.add_traces(vec![m_trace, x_trace, s_trace]);
    plot.show();

    let mut plot = Plot::new();
    let filter_trace = Scatter::new(data.t.clone(), v_history).name("Filter Velocity");
    let real_trace = Scatter::new(data.t.clone(), data.v.clone()).name("Real Velocity");
    plot.add_traces(vec![filter_trace, real_trace]);
    plot.show();

    let mut plot = Plot::new();
    let trace = Scatter::new(data.t.clone(), a_history).name("Filter Acceleration");
    let ideal_trace =
        Scatter::new(vec![data.t[0], data.t[data.t.len() - 1]], vec![G, G]).name("Ideal");
    plot.add_traces(vec![trace, ideal_trace]);
    let layout = Layout::default()
        .title(Title::new("Acceleration"))
        .x_axis(Axis::default().title(Title::new("t (s)")))
        .y_axis(
            Axis::default()
                .title(Title::new("Acceleration (m/s^2)"))
                .range(vec![-20.0, 20.0]),
        );
    plot.set_layout(layout);
    plot.show();
}

struct Data {
    pub s: Vec<f64>, // True value distance
    pub x: Vec<f64>, // Measurement distance
    pub t: Vec<f64>, // Time
    pub v: Vec<f64>, // velocity
}

fn get_data() -> Data {
    let mut s = INIT_S;
    let mut t = 0.0;
    // let mut u = ;
    let mut u = INIT_U;
    let g = G;
    let dt = TS;

    let mut s_history = vec![];
    let mut x_history = vec![];
    let mut t_history = vec![];
    let mut v_history = vec![];

    let normal = Normal::new(0.0, SIGNOISE).unwrap();
    let mut rng = rand::thread_rng();

    while t < MAXT {
        // Measurement

        s_history.push(s);
        t_history.push(t);
        v_history.push(u);
        x_history.push(s + normal.sample(&mut rng));

        // Propagate
        let v = u + g * dt;
        let d = 0.5 * (u + v) * dt;

        s += d;
        u = v;
        t += dt;
    }

    return Data {
        s: s_history,
        x: x_history,
        t: t_history,
        v: v_history,
    };
}

fn phi(dt: f64) -> Matrix {
    let phi = matrix(
        vec![
            1.0,
            dt,
            0.5 * dt.powf(2.0), //
            0.0,
            1.0,
            dt, //
            0.0,
            0.0,
            1.0, //
        ],
        3,
        3,
        Row,
    );

    return phi;
}

fn q(dt: f64) -> Matrix {
    let q = matrix(
        vec![
            dt.powf(5.0) / 20.0,
            dt.powf(4.0) / 8.0,
            dt.powf(3.0) / 6.0, //
            dt.powf(4.0) / 8.0,
            dt.powf(3.0) / 3.0,
            dt.powf(2.0) / 2.0, //
            dt.powf(3.0) / 6.0,
            dt.powf(2.0) / 2.0,
            dt,
        ],
        3,
        3,
        Row,
    );

    return PHIS * q;
}
