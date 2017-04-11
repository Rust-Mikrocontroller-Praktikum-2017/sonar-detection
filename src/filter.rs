use main;

pub const AUDIO_BUF_LENGTH: usize = 256;
pub const MULTIPLIER: [f32; 33] = [-0.399,-0.046,0.407,0.486,0.046,-0.525,-0.626,-0.046,0.731,0.885,0.046,-1.185,-1.532,-0.046,2.997,6.062,7.349,6.062,2.997,-0.046,-1.532,-1.185,0.046,0.885,0.731,-0.046,-0.626,-0.525,0.046,0.486,0.407,-0.046,-0.399];

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
        //sum_right = sum_right + (MULTIPLIER[j] * audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].0 as f32) as i32;
        //sum_left =  sum_left + (MULTIPLIER[j] * audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].1 as f32) as i32;
        sum_right = sum_right + audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].0 as i32;
        sum_left =  sum_left +audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].1 as i32;
    }
    audio_buf.data_filter[index] = ( (sum_right as i32), (sum_left as i32));
    
}
