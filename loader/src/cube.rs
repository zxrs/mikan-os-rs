use crate::{EFIEventType, EFISystemTable, EFITimerDelay, EFITpl};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;
const K1: f64 = 40.0;
const DISTANCE_FROM_CAM: f64 = 100.0;

trait Sin {
    type Item;
    fn sin(self) -> Self::Item;
}

impl Sin for f64 {
    type Item = f64;
    fn sin(self) -> Self::Item {
        unsafe { core::intrinsics::sinf64(self) }
    }
}

trait Cos {
    type Item;
    fn cos(self) -> Self::Item;
}

impl Cos for f64 {
    type Item = f64;
    fn cos(self) -> Self::Item {
        unsafe { core::intrinsics::cosf64(self) }
    }
}

fn calcurate_x(a: f64, b: f64, c: f64, i: f64, j: f64, k: f64) -> f64 {
    j * a.sin() * b.sin() * c.cos() - k * a.cos() * b.sin() * c.cos()
        + j * a.cos() * c.sin()
        + k * a.sin() * c.sin()
        + i * b.cos() * c.cos()
}

fn calcurate_y(a: f64, b: f64, c: f64, i: f64, j: f64, k: f64) -> f64 {
    j * a.cos() * c.cos() + k * a.sin() * c.cos() - j * a.sin() * b.sin() * c.sin()
        + k * a.cos() * b.sin() * c.sin()
        - i * b.cos() * c.sin()
}

fn calcurate_z(a: f64, b: f64, _c: f64, i: f64, j: f64, k: f64) -> f64 {
    k * a.cos() * b.cos() - j * a.sin() * b.cos() + i * b.sin()
}

fn calcurate_for_surface(
    z_buffer: &mut [f64],
    buffer: &mut [[u8; 4]],
    a: f64,
    b: f64,
    c: f64,
    cube_x: f64,
    cube_y: f64,
    cube_z: f64,
    rgb: [u8; 4],
) {
    let x = calcurate_x(a, b, c, cube_x, cube_y, cube_z);
    let y = calcurate_y(a, b, c, cube_x, cube_y, cube_z);
    let z = calcurate_z(a, b, c, cube_x, cube_y, cube_z) + DISTANCE_FROM_CAM;

    let z = if z == 0.0 { 1e-6 } else { z };

    let ooz = 1.0 / z;

    let xp = (WIDTH as f64 / 2.0 + K1 * ooz * x * 2.0) as i32;
    let yp = (HEIGHT as f64 / 2.0 + K1 * ooz * y) as i32;

    let idx = xp + yp * WIDTH as i32;
    if idx >= 0 && idx < WIDTH as i32 * HEIGHT as i32 {
        if ooz > z_buffer[idx as usize] {
            z_buffer[idx as usize] = ooz;
            buffer[idx as usize] = rgb;
        }
    }
}

pub fn rotate<'a>(system_table: &'a EFISystemTable, frame_buffer: &mut [u8]) -> ! {
    let mut a = 0.0;
    let mut b = 0.0;
    let mut c = 0.0;
    let mut z_buffer = [0.0; WIDTH * HEIGHT];
    let mut buffer = [[0u8, 0u8, 0u8, 0u8]; WIDTH * HEIGHT];
    let incremental_speed = 0.07;

    let event = system_table
        .boot_services
        .create_event(EFIEventType::Timer(), EFITpl::Application())
        .unwrap();
    let events = [event];

    loop {
        buffer.iter_mut().for_each(|rgb| *rgb = [0, 0, 0, 0]);
        z_buffer.iter_mut().for_each(|b| *b = 0.0);

        a += incremental_speed;
        b += incremental_speed;
        c += 0.01;

        let mut cube_x = -20.0;
        while cube_x < 20.0 {
            let mut cube_y = -20.0;
            while cube_y < 20.0 {
                calcurate_for_surface(
                    &mut z_buffer,
                    &mut buffer,
                    a,
                    b,
                    c,
                    cube_x,
                    cube_y,
                    -20.0,
                    [255, 0, 0, 0],
                );
                calcurate_for_surface(&mut z_buffer, &mut buffer, a, b, c, 20.0, cube_y, cube_x, [
                    0, 255, 0, 0,
                ]);
                calcurate_for_surface(
                    &mut z_buffer,
                    &mut buffer,
                    a,
                    b,
                    c,
                    -20.0,
                    cube_y,
                    cube_x,
                    [0, 0, 255, 0],
                );
                calcurate_for_surface(
                    &mut z_buffer,
                    &mut buffer,
                    a,
                    b,
                    c,
                    -cube_x,
                    cube_y,
                    20.0,
                    [255, 255, 0, 0],
                );
                calcurate_for_surface(
                    &mut z_buffer,
                    &mut buffer,
                    a,
                    b,
                    c,
                    cube_x,
                    -20.0,
                    -cube_y,
                    [255, 0, 255, 0],
                );
                calcurate_for_surface(&mut z_buffer, &mut buffer, a, b, c, cube_x, 20.0, cube_y, [
                    0, 255, 255, 0,
                ]);
                cube_y += 0.5;
            }
            cube_x += 0.5;
        }

        frame_buffer
            .chunks_exact_mut(800 * 4)
            .zip(buffer.chunks(WIDTH).map(|c| c.as_flattened()))
            .for_each(|(dst, src)| {
                let (dst, _) = dst.split_at_mut(WIDTH * 4);
                dst.copy_from_slice(src)
            });

        system_table
            .boot_services
            .set_timer(event, EFITimerDelay::Relative, 300_000)
            .unwrap();
        system_table.boot_services.wait_for_event(&events).unwrap();
    }
}
