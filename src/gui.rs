use main;

//STRUCTS
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
    color: u16,
}

////
//CONSTANTS
pub const BACKGROUND_COLOR: u16 = 0x0;
pub const FIRST_COLOR: u16 = 0xffff;
pub const SECOND_COLOR: u16 = 0xf228;
pub const THIRD_COLOR: u16 = 0xd681;
pub const X_DIM_RES: u16 = 480;
pub const Y_DIM_RES: u16 = 272;
pub const SMOOTH_MULTIPLIER: u16 = 20;

////
//INITIALIZERS
pub fn init_point(new_x: i16, new_y: i16) -> Point {
    return Point{x: new_x, y: new_y};
}
pub fn init_vector(new_delta_x: i16, new_delta_y: i16) -> Vector {
    return Vector{delta_x: new_delta_x, delta_y: new_delta_y, last_anchor: Point{x: 0, y: 0}};
}
pub fn init_box(new_start: Point, new_length_x: u16, new_length_y: u16, new_color: u16) {
    return Box{start: new_start, length_x: new_length_x, length_y: new_length_y, color: color};
}

pub fn refresh_display(waves_mode_activated: bool, lcd: &mut stm32f7::lcd::Lcd) {

}

////
//CALCULATORS, COMPUTES

//calculating vector just by assuming a fixed angle and a fixed length
pub fn calculate_vector(input_vector: &mut Vector, sinus: f32) -> &mut Vector {
    //compute intensioned length of vector (root[x^2 + y^2])
    let length = (Y_DIM_RES/2) - 10; //just hardcoded
    //compute deltas
    input_vector.delta_y = (sinus * length as f32) as i16;
    let raw_delta_x: f32 = (length.pow(2) as f32 - delta_y.pow(2) as f32).sqrt();
    input_vector.delta_x = raw_delta_x as i16;
    return input_vector;
}

pub fn is_in_box(current_x: u16, current_y: u16, input_box: &Box) -> bool {
    //x
    if (current_x as i16 >= input_box.start.x) && (current_x as i16 < input_box.start.x + input_box.length_x as i16) {
        //y
        if (current_y as i16 >= input_box.start.y) && (current_y as i16 < input_box.start.y + input_box.length_y as i16) {
            return true;
        }
    }
    return false;
}

////
//PRINT ALGORITHMS

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
pub fn print_vector_reposition(vec: &mut Vector, from_x: i16, from_y: i16, lcd: &mut stm32f7::lcd::Lcd, color: u16) {
    print_vector(vec, from_x, from_y, color, lcd);
    vec.last_anchor.x = from_x;
    vec.last_anchor.y = from_y;
}

//prints box
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
