use main;

pub const AUDIO_BUF_LENGTH: usize = 64;

pub struct AudioBuffer {
    //Buffer with the filtered signals
    pub data_filter: [(i32, i32); AUDIO_BUF_LENGTH],//(right,left)
    //Buffer with the raw signals
    pub data_raw:  [(i32, i32); AUDIO_BUF_LENGTH], //(right,left)   
}

pub fn initAudioBuffer() -> AudioBuffer {
    AudioBuffer{data_filter: [(0,0); AUDIO_BUF_LENGTH], data_raw: [(0,0); AUDIO_BUF_LENGTH]}
}

//Filter the raw audio data. Filter one signal at a time
pub fn fir_filter(audio_buf: &mut AudioBuffer, index: usize) {
    //Wie viele Werte einschließlich mit dem aktuellen für die Mittelwert-Berechnung betrachetet werden sollen
    let n = 12;
    let mut sum_right : i32 = 0;
    let mut sum_left : i32 = 0;
    for j in 0..n {
        sum_right = sum_right + audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].0;
        sum_left =  sum_left + audio_buf.data_raw[((index + AUDIO_BUF_LENGTH - j) % AUDIO_BUF_LENGTH) as usize ].1;
    }
    audio_buf.data_filter[index] = ( (sum_right as i32), (sum_left as i32));
    
}
