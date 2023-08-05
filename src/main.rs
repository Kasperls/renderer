// https://sotrh.github.io/learn-wgpu/

use renderer::run;

fn main() {
    pollster::block_on(run());
}

// current
// https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#getting-data-into-a-texture