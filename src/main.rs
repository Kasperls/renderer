// https://sotrh.github.io/learn-wgpu/

use renderer::run;

fn main() {
    pollster::block_on(run());
}

// current
// https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-blinn-phong-model