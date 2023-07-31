// https://sotrh.github.io/learn-wgpu/

use renderer::run;

fn main() {
    pollster::block_on(run());
}

// current
// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/#we-re-finally-talking-about-them