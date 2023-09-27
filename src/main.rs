use odb_poly_decomp::{decomposer::Decomposer, point::Point};

pub fn main() {
    tracing_subscriber::fmt()
        .pretty()
        // enable everything
        .with_max_level(tracing::Level::TRACE)
        // sets this to be the default, global collector for this application.
        .init();

    let points: Vec<Point> = [(0, 0), (2, 0), (2, 2), (1, 2), (1, 1), (0, 1)]
        .into_iter()
        .map(|(x, y)| Point::new(x, y))
        .collect();

    let result = Decomposer::decompose(points);
    println!("{:?}", result);
}
