use examples::{
    map::{flat::FlatAndEmpty, generator::MapGenerator},
    runner::RunnerBuilder,
};

fn main() {
    let map = MapGenerator::new(FlatAndEmpty)
        .width(500.0)
        .height(150.0)
        .generate();
    let runner = RunnerBuilder::new(map).expire(Some(60 * 60)).build()?;
    runner.run();
}
