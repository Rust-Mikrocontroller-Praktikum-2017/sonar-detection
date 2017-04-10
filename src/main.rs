#![no_std]
#![no_main]
#[warn(unused_unsafe)]

extern crate stm32f7_discovery as stm32f7;
// initialization routines for .data and .bss
extern crate r0;
//extern crate std::f32;
use stm32f7::{system_clock, sdram, lcd, i2c, touch, board, embedded};

#[derive(Copy, Clone)]
struct Point {
    x: i16,
    y: i16,
}
#[derive(Copy, Clone)]
struct Vector {
    delta_x: i16,
    delta_y: i16,
    last_anchor: Point,
}
#[derive(Copy, Clone)]
struct Box {
    start: Point,
    length_x: u16,
    length_y: u16,
}

const BACKGROUND_COLOR: u16 = 0x0;
const FIRST_COLOR: u16 = 0xffff;
const SECOND_COLOR: u16 = 0xd000;
const X_DIM_RES: u16 = 480;
const Y_DIM_RES: u16 = 272;

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        static __DATA_LOAD: u32;
        static __DATA_END: u32;
        static mut __DATA_START: u32;
        static mut __BSS_START: u32;
        static mut __BSS_END: u32;
    }
    let data_load = &__DATA_LOAD;
    let data_start = &mut __DATA_START;
    let data_end = &__DATA_END;
    let bss_start = &mut __BSS_START;
    let bss_end = &__BSS_END;

    // initializes the .data section
    //(copy the data segment initializers from flash to RAM)
    r0::init_data(data_start, data_end, data_load);
    //zeroes the .bss section 
    r0::zero_bss(bss_start, bss_end);
    
    unsafe {
        let scb = stm32f7::cortex_m::peripheral::scb_mut();
        scb.cpacr.modify(|v| v | 0b1111 << 20);
    }

    main(board::hw());
}

fn main (hw: board::Hardware) -> ! {
    let board::Hardware { rcc,
                            pwr,
                            flash, 
                            fmc,
                            ltdc,
                            gpio_a,
                            gpio_b,
                            gpio_c,
                            gpio_d,
                            gpio_e,
                            gpio_f,
                            gpio_g,
                            gpio_h,
                            gpio_i,
                            gpio_j,
                            gpio_k,
                            i2c_3,
                            .. } = hw;
    use embedded::interfaces::gpio::{self, Gpio};
    let mut gpio = Gpio::new(gpio_a,
                                gpio_b,
                                gpio_c,
                                gpio_d,
                                gpio_e,
                                gpio_f,
                                gpio_g,
                                gpio_h,
                                gpio_i,
                                gpio_j,
                                gpio_k);
    system_clock::init(rcc, pwr, flash);    
    // enableall gpioports
    rcc.ahb1enr.update(|r| {
        r.set_gpioaen(true);
        r.set_gpioben(true);
        r.set_gpiocen(true);
        r.set_gpioden(true);
        r.set_gpioeen(true);
        r.set_gpiofen(true);
        r.set_gpiogen(true);
        r.set_gpiohen(true);
        r.set_gpioien(true);
        r.set_gpiojen(true);
        r.set_gpioken(true);
    });

    sdram::init(rcc, fmc, &mut gpio);
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    i2c::init_pins_and_clocks(rcc, &mut gpio);
    let mut i2c_3 = i2c::init(i2c_3);
    touch::check_family_id(&mut i2c_3).unwrap();
    
    lcd.clear_screen();
    lcd.set_background_color(lcd::Color::from_hex(BACKGROUND_COLOR as u32));

    //init audio_main_vec, set anchor, print audio_main_vec
    let aud_main_vec_anchor = Point{x: (X_DIM_RES/2) as i16, y: (Y_DIM_RES/2) as i16};
    let mut audio_main_vec = Vector{delta_x: 126, delta_y: 0, last_anchor: Point{x: aud_main_vec_anchor.x, y: aud_main_vec_anchor.y}};
    print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, FIRST_COLOR);

    //init center_box above aud_main_vec_anchor of audio_main_vec
    let center_box = Box{start: Point{x: aud_main_vec_anchor.x-5, y: aud_main_vec_anchor.y-5}, length_x: 10, length_y: 10};
    
    //init smoothing_box and print
    let smoothing_box = Box{start: Point{x: 20, y: 5}, length_x: 20, length_y: 200};
    print_box(&smoothing_box, &mut lcd);
    let mut smooth_strength: u16 = 1;
    let smooth_multiplier: u16 = 20;

    //init view mode and view_toogle_box and print
    let mut view_actual_waves: bool = false;
    let view_toogle_box = Box{start: Point{x: 20, y: 217}, length_x: 50, length_y: 50};
    print_box(&view_toogle_box, &mut lcd);

    loop {
        //enviro updates
        print_box(&view_toogle_box, &mut lcd);
        if !view_actual_waves {
            print_box(&center_box, &mut lcd);
            print_box(&smoothing_box, &mut lcd);
        }

        //check for touches
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            
            //remove old audio_main_vec
            print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, BACKGROUND_COLOR);
            //update deltas to difference between anchor and touch position and print audio_main_vec again
            audio_main_vec.delta_x = touch.x as i16 - aud_main_vec_anchor.x;
            audio_main_vec.delta_y = touch.y as i16 - aud_main_vec_anchor.y;
            print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, FIRST_COLOR);

            //check box touch
            if (touch.y >= 20) && (touch.y <= 70) {
                if (touch.y >= smoothing_box.start.y as u16) && (touch.y < smoothing_box.start.y as u16 + smoothing_box.length_y) {
                    smooth_strength = (touch.y - smoothing_box.start.y as u16) * smooth_multiplier;
                } else if (touch.y >= view_toogle_box.start.y as u16) && (touch.y < view_toogle_box.start.y as u16 + view_toogle_box.length_y) {
                    view_actual_waves = true;
                }
            }
        }
    }
}

//
//

//Bresenhams Algorithm for drawing lines with integers
//vec: Vector
//from_x: start x coordinate of vector
//from_y: start y coordinate of vector 
//color: color
//lcd: display
fn print_vector (vec: &Vector, from_x: i16, from_y: i16, color: u16, lcd: &mut stm32f7::lcd::Lcd) {
    let mut sign_start_x = 0;
    let mut sign_start_y = 0;
    let mut sign_dest_x = 0;
    let mut sign_dest_y = 0;
    if vec.delta_x < 0 {
        sign_start_x = -1;
        sign_dest_x = -1;
    } else if vec.delta_x > 0 {
        sign_start_x = 1;
        sign_dest_x = 1;
    }
    if vec.delta_y < 0 {
        sign_start_y = -1;
    } else if vec.delta_y > 0 {
        sign_start_y = 1;
    }
    let mut longest = vec.delta_x.abs();
    let mut shortest = vec.delta_y.abs();
    if !(longest>shortest) {
        longest = shortest;
        shortest = vec.delta_x.abs();
        if vec.delta_y < 0 {
            sign_dest_y = -1;
        } else if vec.delta_y > 0{
            sign_dest_y = 1;
        }
        sign_dest_x = 0;
    }
    let mut numerator = longest >> 1;
    let mut current_x = from_x;
    let mut current_y = from_y;
    for _ in 0 ..longest+1 {
        lcd.print_point_color_at(limit(current_x, X_DIM_RES) as u16, limit(current_y, Y_DIM_RES) as u16, color);
        numerator += shortest;
        if !(numerator<longest) {
            numerator -= longest;
            current_x += sign_start_x;
            current_y += sign_start_y;
        } else {
            current_x += sign_dest_x;
            current_y += sign_dest_y;
        }
    }
}

//vec: Vector
//from_x: start x coordinate of vector
//from_y: start y coordinate of vector 
//lcd: Display
fn print_vector_reposition(vec: &mut Vector, from_x: i16, from_y: i16, lcd: &mut stm32f7::lcd::Lcd, color: u16) {
    print_vector(vec, from_x, from_y, color, lcd);
    vec.last_anchor.x = from_x;
    vec.last_anchor.y = from_y;
}

//
//

fn print_box(box_input: &Box, lcd: &mut stm32f7::lcd::Lcd) {
    for x in box_input.start.x..(box_input.start.x + box_input.length_x as i16) {
        for y in box_input.start.y..(box_input.start.y + box_input.length_y as i16) {
            lcd.print_point_color_at(limit(x, X_DIM_RES) as u16, limit(y, Y_DIM_RES) as u16, SECOND_COLOR);
        }
    }
}

//
//

//calculating vector just by assuming a fixed angle and a fixed length
fn calculate_vector(input_vector: &mut Vector, sinus: f32) -> &mut Vector {
    //compute intensioned length of vector (root[x^2 + y^2])
    let length = (Y_DIM_RES/2) - 10; //just hardcoded due to undefined behaviour of microcontroller by dealing with f32, f64
    //compute deltas
    input_vector.delta_y = (sinus * length as f32) as i16;
    //let raw_delta_x = ((length.pow(2) as i16 - self.delta_y.pow(2)) as f32).sqrt();
    //input_vector.delta_x = raw_delta_x as i16;
    return input_vector;
}

//prevent out of display cases
//pixel_position: pixel to investigate
//dim_max: border top
fn limit(pixel_position: i16, dim_max: u16) -> i16 {
    if pixel_position < 0 {
        return 0;
    } else if pixel_position >= dim_max as i16 {
        return dim_max as i16 - 1
    } else {
        return pixel_position;
    }
}
