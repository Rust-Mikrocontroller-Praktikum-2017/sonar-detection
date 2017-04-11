#![no_std]
#![no_main]
#[inline(never)]
#[warn(unused_unsafe)]


mod filter;

extern crate stm32f7_discovery as stm32f7;
// initialization routines for .data and .bss
extern crate r0;
//extern crate std::f32;
use stm32f7::{system_clock, sdram, lcd, i2c,audio, touch, board, embedded};

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
    stm32f7::heap::init();
    
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
                            sai_2,
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
    //i2c
    i2c::init_pins_and_clocks(rcc,&mut gpio);
    let mut i2c_3 = i2c::init(i2c_3);

    // sai and stereo microphone
    audio::init_sai_2_pins(&mut gpio);
    audio::init_sai_2(sai_2, rcc);
    assert!(audio::init_wm8994(&mut i2c_3).is_ok());

    //Init lcd controller and sram
    sdram::init(rcc,fmc,&mut gpio);
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    let mut layer1 = lcd.layer_1().unwrap();
    let mut layer2 = lcd.layer_2().unwrap();
    layer1.clear();
    layer2.clear();
    stm32f7::init_stdout(layer2);

    //Audio buffer
    let mut audio_buf = filter::init_audio_buffer();

    //Set clock divider
    let mut acr1 = sai_2.acr1.read();
    let mut bcr1 = sai_2.bcr1.read();
    acr1.set_mcjdiv(6 as u8);
    bcr1.set_mcjdiv(6 as u8);
    sai_2.acr1.write(acr1);
    sai_2.bcr1.write(bcr1);
    //Thereshold for relevant data
    let threshold = 2048;
    loop{
       //Poll for new audio data until the audio buffer for filterd data is full
        let mut i = 0;
        while i < filter::AUDIO_BUF_LENGTH {
            //Write data from mics in data_raw puffer
            while !sai_2.bsr.read().freq() {} // fifo_request_flag
            audio_buf.data_raw[i].0 = (sai_2.bdr.read().data() as i32);
            while !sai_2.bsr.read().freq() {} // fifo_request_flag
            audio_buf.data_raw[i].1 = (sai_2.bdr.read().data() as i32);  

            //Only filter relevant data above a threshold
            if audio_buf.data_raw >= threshold || audio_buf.data_raw[i].1 >= threshold {
                filter::fir_filter(&mut audio_buf, i);
                i += 1;
            }
        }
        //Get sinus for displaying audio direction
        let mut sin_a = sonar_localization::get_sound_source_direction_sin(&audio_buf.data_filter);
    }  

}