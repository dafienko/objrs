mod camera;

use objrs::run;

pub fn main() {
    pollster::block_on(run());
}