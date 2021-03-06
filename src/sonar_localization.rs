use filter;

//fn get_phase_difference(data1: &[i32], data2: &[i32]) -> f32 {
    //not used
    //in progress, not ready for use
    //let velocity = 340.0;

    //let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
    //let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
    //let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
    //let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);

    //let T1 = 1.0 / (zero1_2 - zero1_1) * 2.0;
    //let T2 = 1.0 / (zero2_2 - zero2_1) * 2.0;
    //let freq = (T1 + T2) / 2.0;


    //} else {

    //}
    
//}
//fn get_time_difference(data1: &[i32], data2: &[i32]) -> f32 {
//    let velocity = 340.0;
//
//    let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
//    let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
//    let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
//    let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);
//    let freq1 = 1.0 / (zero1_2 - zero1_1) * 2.0;
//    let freq2 = 1.0 / (zero2_2 - zero2_1) * 2.0;
//    let freq = (freq1 + freq2) / 2.0;
    //betrag des Lauzeitunterschieds
//    if (zero1_1 - zero2_1) < 0 {
//        let delta_zero = zero2_1 - zero1_1;
//    } else {
//        let delta_zero = zero1_1 - zero2_1;
//    }
    //Wenn delta_zero größer als halbes T (Zeitintervall für eine Sinusschwingung)
    //dann war das signal zuerst am 2. mikrophon ansonsten war es zuerst am ersten
//    if delta_zero > (0.5 * (1 / freq)) {
//        delta_zero
//    } else {
//        -1.0 * delta_zero
//    }
//    
//}
//pub fn get_sound_source_direction_degree(data1: &[i32], data2: &[i32]) -> i8 {
//    let pi = 3.14159265358979323846264338327950288;
    // velocity = Schallgeschwindigkeit
//    let velocity = 340.0;
    // distance = abstand der beiden Mikrofone
//    let distance = 0.020;
    //dt = delta time, Laufzeitunterschied des Signals
//    let dt = get_time_difference(data1, data2);
    //ds = delta strecke, Abstandunterschied der Signalquelle zu den beiden Mikrofonen
//    let ds = velocity*dt;
    //gibt den Winkel zur Signalquelle zurück
//    (((ds / distance).asin() / pi) * 180.0) as i8
//}

pub fn get_sound_source_direction_sin(data: &[(i32, i32)]) -> f32 {
    let mut data1: [i32; filter::AUDIO_BUF_LENGTH] = [0; filter::AUDIO_BUF_LENGTH];
    let mut data2: [i32; filter::AUDIO_BUF_LENGTH] = [0; filter::AUDIO_BUF_LENGTH];
    for i in 0..filter::AUDIO_BUF_LENGTH {
        data2[i] = data[i].0;
        data1[i] = data[i].1;
    }
    // velocity = Schallgeschwindigkeit
    let velocity = 340.0;
    // distance = abstand der beiden Mikrofone
    let distance = 0.020;
    //dt = delta time, Laufzeitunterschied des Signals
    let dt = get_time_difference(&data1, &data2);
    //println!("{}",dt);
    if dt == 0.0 {
        return 0.0
    }
    //ds = delta strecke, Abstandsunterschied der Signalquelle zu den beiden Mikrofonen
    let ds = velocity*dt;
    //gibt den sinus des Winkels zur Signalquelle zurück
    (ds / distance) as f32

    
}
fn get_time_difference(data1: &[i32], data2: &[i32]) -> f32 {
    let zero1_1 = get_first_zero_point_from_pos_to_neg(data1);
    let zero2_1 = get_first_zero_point_from_pos_to_neg(data2);
    let zero1_2 = get_second_zero_point_from_pos_to_neg(data1);
    let zero2_2 = get_second_zero_point_from_pos_to_neg(data2);
    if zero1_1 == -1.0 || zero2_1 == -1.0 || zero1_2 == -1.0 || zero2_2 == -1.0 {
        return 0.0;
    }
    let t1 = zero1_2 - zero1_1;
    let t2 = zero2_2 - zero2_1;
    //println!("{}",t1);
    //println!("{}",t2);
    if t1 * 1.05 < t2 || t1 * 0.95 > t2 {
        return 0.0;
    }
    let t = (t1 + t2) / 2.0;
    if (t < 0.0002) {
        return 0.0;
    }
    // ds1 = Laufzeitunterschied der beiden Signale an der ersten Nullstelle
    let dt1 = zero2_1 - zero1_1;
    // ds2 = Laufzeitunterschied der beiden Signale an der zweiten Nullstelle
    let dt2 = zero2_2 - zero1_2;
    //if (ds1 < ds2 * 1.05 && ds1 > ds2 * 0.95) {
    let dt_temp = (dt1 + dt2) / 2.0;
    //bestimmt ob das signal von links oder von rechts kommt
    if dt_temp > (0.5 * t) {
        t - dt_temp
    } else {
        dt_temp
    }
}
fn get_first_zero_point_from_pos_to_neg(data: &[i32]) -> f32 {
    //berechnet die erste Nullstelle des Signals
    let mut prev_data: i32 = 0;
    let mut not_first = false;
    let mut counter:u32 = 0;
    for current_data in data.iter() {
        if not_first {
            if (prev_data > 0) && (*current_data <= 0) {
                //Annäherung der Nullstelle durch eine Positive und eine Negative Stelle
                //Annäherung durch gerade durch die NullLinie
                let dx = 1.0 / 48000.0;
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
    -1.0
}
fn get_second_zero_point_from_pos_to_neg(data: &[i32]) -> f32 {
    //berechnet die zweite Nullstelle des Signals
    let mut prev_data: i32 = 0;
    let mut not_first = false;
    let mut first_zero = true;
    let mut counter:u32 = 0;
    for current_data in data.iter() {
        if not_first {
            if (prev_data > 0) && (*current_data <= 0) {
                if !first_zero {
                    let dx = 1.0 / 48000.0;
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