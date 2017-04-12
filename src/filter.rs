pub const AUDIO_BUF_LENGTH: usize = 256;

pub const MULTIPLIER: [i64; 33] = [-399,-46,407,486,46,-525,-626,-46,731,885,46,-1185,-1532,-46,2997,6062,7349,6062,2997,-46,-1532,-1185,46,885,731,-46,-626,-525,46,486,407,-46,-399];


pub struct AudioBuffer {
    //Buffer with the filtered signals
    pub data_filter: [(i32, i32); AUDIO_BUF_LENGTH],//(right,left)
    //Buffer with the raw signals
    pub data_raw:  [(i32, i32); AUDIO_BUF_LENGTH], //(right,left)   
}

//Init audio buffer
pub fn init_audio_buffer() -> AudioBuffer {
    AudioBuffer{data_filter: [(0,0); AUDIO_BUF_LENGTH], data_raw: [(0,0); AUDIO_BUF_LENGTH]}
}

//Filter the raw audio data. Filter one signal at a time
pub fn fir_filter(audio_buf: &mut AudioBuffer, index: usize) {
    //Wie viele Werte einschließlich mit dem aktuellen für die Mittelwert-Berechnung betrachetet werden sollen
    let n = 33;
    let mut sum_right : i32 = 0;
    let mut sum_left : i32 = 0;
    for j in 0..n {

        //sum_right = sum_right + (MULTIPLIER[j] * audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].0 as i64) as i32;
        //sum_left =  sum_left + (MULTIPLIER[j] * audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].1 as i64) as i32;
        sum_right = sum_right + audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].0 as i32;
        sum_left =  sum_left + audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].1 as i32;
    }
    //sum_right = sum_right / 1000;
    //sum_left = sum_left / 1000;
    audio_buf.data_filter[index] = ( (sum_right as i32), (sum_left as i32));

    
}
