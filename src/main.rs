// https://sotrh.github.io/learn-wgpu/

use renderer::run;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "0");
    pollster::block_on(run());
}

// current
// https://web.cs.wpi.edu/~matt/courses/cs563/talks/shadow/shadow.html