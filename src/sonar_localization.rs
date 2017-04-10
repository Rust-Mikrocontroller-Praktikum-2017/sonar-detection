

fn get_phase_difference(data1: &[i32], data2: &[i32]) -> f32 {
    //experimental
    let velocity = 340.0;

    let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
    let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
    let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
    let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);
    let freq1 = 1.0 / (zero1_2 - zero1_1) * 2.0;
    let freq2 = 1.0 / (zero2_2 - zero2_1) * 2.0;
    let freq = (freq1 + freq2) / 2.0;
    if (zero1_1 - zero2_1) > (0.5 * velocity / freq) {

    } else {

    }
    
}
fn get_time_difference(data1: &[i32], data2: &[i32]) -> f32 {
    let velocity = 340.0;

    let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
    let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
    let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
    let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);
    let freq1 = 1.0 / (zero1_2 - zero1_1) * 2.0;
    let freq2 = 1.0 / (zero2_2 - zero2_1) * 2.0;
    let freq = (freq1 + freq2) / 2.0;
    if (zero1_1 - zero2_1) > (0.5 * velocity / freq) {
        zero2_1 - zero1_1
    } else {
        zero1_1 - zero2_1
    }
    
}
pub fn get_sound_source_direction_degree(data1: &[i32], data2: &[i32]) -> i8 {
    let pi = 3.14159265358979323846264338327950288;
    let velocity = 340.0;
    let distance = 0.020;
    let dt = get_time_difference(data1, data2);
    let ds = velocity*dt;
    (((ds / distance).asin() / pi) * 180.0) as i8
}

pub fn get_sound_source_direction_sin(data1: &[i32], data2: &[i32]) -> f32 {
    let velocity = 340.0;
    let distance = 0.020;
    let dt = get_time_difference(data1, data2);
    let ds = velocity*dt;

    ds / distance
}
//fn get_time_difference(data1: &[i32], data2: &[i32]) -> f32 {
//    let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
//    let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
//    let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
//    let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);

//    let phase1 = zero2_1 - zero1_1;
//    let phase2 = zero2_2 - zero1_2;
    //if (phase1 < phase2 * 1.05 && phase1 > phase2 * 0.95) {
//        return (phase1 + phase2) / 2.0;
    //}
    //0.0


//}
fn get_first_zero_point_from_pos_to_neg(data: &[i32]) -> f32{
    let mut prev_data: i32 = 0;
    let mut not_first = false;
    let mut counter:u32 = 0;
    for current_data in data.iter() {
        if not_first {
            if (prev_data > 0) && (*current_data <= 0) {
                let dx = 1.0 / 16000.0;
                let dy = current_data - prev_data;
                let dt = counter as f32 * dx + (-1 * prev_data) as f32 / (dy as f32 / dx);
                return dt;
            }
        } else {
            not_first = true;
        }
        prev_data = *current_data;
        counter += 1;
    }
    0.0
}
fn get_second_zero_point_from_pos_to_neg(data: &[i32]) -> f32 {
    let mut prev_data: i32 = 0;
    let mut not_first = false;
    let mut first_zero = true;
    let mut counter:u32 = 0;
    for current_data in data.iter() {
        if not_first {
            if (prev_data > 0) && (*current_data <= 0) {
                if (!first_zero) {
                    let dx = 1.0 / 16000.0;
                    let dy = current_data - prev_data;
                    let dt = counter as f32 * dx + (-1 * prev_data) as f32 / (dy as f32 / dx);
                    return dt;
                }
                first_zero = false;
            }
        } else {
            not_first = true;
        }
        prev_data = *current_data;
        counter += 1;
    }
    0.0
}