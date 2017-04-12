extern crate stm32f7_discovery as stm32f7;

//STRUCTS
#[derive(Copy, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}
#[derive(Copy, Clone)]
pub struct Vector {
    pub delta_x: i16,
    pub delta_y: i16,
    pub last_anchor: Point,
}
#[derive(Copy, Clone)]
pub struct Box {
    pub start: Point,
    pub length_x: u16,
    pub length_y: u16,
    pub color: u16,
}

////
//CONSTANTS
pub const BACKGROUND_COLOR: u16 = 0x11ac;
pub const FIRST_COLOR: u16 = 0xffff;
pub const SECOND_COLOR: u16 = 0xfae0;
pub const THIRD_COLOR: u16 = 0x07e0;
pub const X_DIM_RES: u16 = 480;
pub const Y_DIM_RES: u16 = 272;
//pub const SMOOTH_MULTIPLIER: u16 = 20; //not in use
pub const TOGGLE_COOLDOWN: usize = 1500;

////
//INITIALIZERS
pub fn init_point(new_x: i16, new_y: i16) -> Point {
    return Point{x: new_x, y: new_y};
}
pub fn init_vector(new_delta_x: i16, new_delta_y: i16) -> Vector {
    return Vector{delta_x: new_delta_x, delta_y: new_delta_y, last_anchor: Point{x: 0, y: 0}};
}
pub fn init_box(new_start: Point, new_length_x: u16, new_length_y: u16, new_color: u16) -> Box {
    return Box{start: new_start, length_x: new_length_x, length_y: new_length_y, color: new_color};
}

////
//CALCULATORS, COMPUTES

//calculating vector just by assuming a fixed angle and a fixed length
pub fn calculate_vector(input_vector: &mut Vector, sinus: f32) -> &mut Vector {
    //compute intensioned length of vector (root[x^2 + y^2])
    let length = (Y_DIM_RES/2) - 10; //just hardcoded
    //compute deltas
    input_vector.delta_y = (sinus * length as f32) as i16;
    let raw_delta_x = sqrt(length.pow(2) as i16 - input_vector.delta_y.pow(2));
    input_vector.delta_x = raw_delta_x as i16;
    return input_vector;
}

pub fn is_in_box(current_x: u16, current_y: u16, input_box: &Box) -> bool {
    //x
    if (current_x as i16 >= input_box.start.x) && ((current_x as i16) < (input_box.start.x + input_box.length_x as i16)) {
        //y
        if (current_y as i16 >= input_box.start.y) && ((current_y as i16) < (input_box.start.y + input_box.length_y as i16)) {
            return true;
        }
    }
    return false;
}

fn sqrt(a: i16) -> i16 {
    if a <= 0 {
        return 0;
    }
    let big_a: u64 = (a as u64) << 10;
    let mut current_root_approx: u64 = (big_a as u64 + (1 << 10)) >> 1;
    for _ in 0..10 {
        current_root_approx = (current_root_approx + ((big_a / current_root_approx) << 10)) >> 1;
    }
    current_root_approx >>= 9;
    if (current_root_approx & 0x1) == 1 {
        current_root_approx += 2;
    }
    return (current_root_approx >> 1) as i16;
}

////
//PRINT ALGORITHMS

//Bresenhams Algorithm for drawing lines with integers
//vec: Vector
//from_x: start x coordinate of vector
//from_y: start y coordinate of vector 
//color: color
//lcd: display
pub fn print_vector(vec: &mut Vector, from_x: i16, from_y: i16, lcd: &mut stm32f7::lcd::Lcd, color: u16) {
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
    vec.last_anchor.x = from_x;
    vec.last_anchor.y = from_y;
}

//removing a vector by drawing itself with BACKGROUND_COLOR
pub fn remove_vector(vec: &mut Vector, lcd: &mut stm32f7::lcd::Lcd) {
    let from_x = vec.last_anchor.x;
    let from_y = vec.last_anchor.y;
    print_vector(vec, from_x, from_y, lcd, BACKGROUND_COLOR);
}

//print a box
pub fn print_box(input_box: &Box, lcd: &mut stm32f7::lcd::Lcd) {
    for x in input_box.start.x..(input_box.start.x + input_box.length_x as i16) {
        for y in input_box.start.y..(input_box.start.y + input_box.length_y as i16) {
            lcd.print_point_color_at(limit(x, X_DIM_RES) as u16, limit(y, Y_DIM_RES) as u16, input_box.color);
        }
    }
}

////
//HELPER METHODS

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
