#![no_std]
#![no_main]
#![warn(unused_unsafe)]
#![feature(compiler_builtins_lib)]


mod filter;
mod gui;
mod sonar_localization;

extern crate stm32f7_discovery as stm32f7;
// initialization routines for .data and .bss
extern crate r0;
//extern crate std::f32;
extern crate compiler_builtins;
use stm32f7::{system_clock, sdram, lcd, i2c, audio, touch, board, embedded};

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
    
    //Floatingpoint-arithmetic activation
    let scb = stm32f7::cortex_m::peripheral::scb_mut();
    scb.cpacr.modify(|v| v | 0b1111 << 20);
    

    main(board::hw());
}

#[inline(never)]
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

    //sdram
    sdram::init(rcc,fmc,&mut gpio);
    
    //i2c
    i2c::init_pins_and_clocks(rcc,&mut gpio);
    let mut i2c_3 = i2c::init(i2c_3);

    // sai and stereo microphone
    audio::init_sai_2_pins(&mut gpio);
    audio::init_sai_2(sai_2, rcc);
    assert!(audio::init_wm8994(&mut i2c_3).is_ok());

    //Init lcd controller,touch and sram
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    touch::check_family_id(&mut i2c_3).unwrap();
    
    lcd.clear_screen();
    lcd.set_background_color(lcd::Color::from_hex(gui::BACKGROUND_COLOR as u32));

    //Audio buffer
    let mut audio_buf = filter::init_audio_buffer();

    //Set clock divider
    let mut acr1 = sai_2.acr1.read();
    let mut bcr1 = sai_2.bcr1.read();
    acr1.set_mcjdiv(2 as u8);
    bcr1.set_mcjdiv(2 as u8);
    sai_2.acr1.write(acr1);
    sai_2.bcr1.write(bcr1);
    //Threshold for relevant data
    let threshold = 400;
    //specifies if sampled audio values are valid
    let mut active;


    //GUI stuff
    let aud_main_vec_anchor = gui::init_point((gui::X_DIM_RES/2) as i16, (gui::Y_DIM_RES/2) as i16);
    let mut audio_main_vec = gui::init_vector(126, 0);
    let mut sinus_alpha:f32; //angle for vector mode
    let mut smoothing_data_counter: i16 = 0;
    let mut smoothing_data_sumup: f32 = 0.0;
    let mut smoothing_data_avg: f32;
    let mut smoothing_data_strength = gui::SMOOTHING_STRENGTH_LOW;
    
    //NOT NEEDED, but ready to be implemented ;)
    
    //let smoothing_box =  gui::init_box(gui::init_point(20, 5), 20, 200, gui::SECOND_COLOR);
    //let view_mode_toggle_box = gui::init_box(gui::init_point(20, 217), 50, 50, gui::THIRD_COLOR);
    //let mut view_mode_last_toggled: usize = system_clock::ticks();
    //---

    let center_box = gui::init_box(gui::init_point(aud_main_vec_anchor.x - 5, aud_main_vec_anchor.y - 5), 10, 10, gui::FIRST_COLOR);
    let mut waves_mode_activated: bool = false;
    
    loop{

        //POLL FOR NEW AUDIO DATA //FILTERING
        let mut i = 0;
        active = false;
        while i < filter::AUDIO_BUF_LENGTH + filter::FILTER_OFFSET {

            //Write data from mics in data_raw puffer
            while !sai_2.bsr.read().freq() {} // fifo_request_flag
            audio_buf.data_raw[i].0 = (sai_2.bdr.read().data() as i16) as i32;
            while !sai_2.bsr.read().freq() {} // fifo_request_flag
            audio_buf.data_raw[i].1 = (sai_2.bdr.read().data() as i16) as i32;
            //Only filter relevant data above a threshold
            if (audio_buf.data_raw[i].0 >= threshold || audio_buf.data_raw[i].0 <= ((-1) * threshold) || audio_buf.data_raw[i].1 >= threshold || audio_buf.data_raw[i].1 <= ((-1) * threshold)) || active {
                active = true;
                if i >= filter::FILTER_OFFSET {
                     filter::fir_filter(&mut audio_buf, i);
                }

                if waves_mode_activated { //interface to display
                    lcd.set_next_col((((audio_buf.data_filter[i].0))) as u16 as u32, (((audio_buf.data_filter[i].1))) as u16 as u32);
                }
                i += 1;
            }
        }

        //COMPUTE SINE OF COLLECTED AUDIO DATA FOR DISPLAYING
        sinus_alpha = sonar_localization::get_sound_source_direction_sin(&audio_buf.data_filter);
        
        //GUI ENVIRO SETUPS AND UPDATES
        //gui::print_box(&view_mode_toggle_box, &mut lcd);
        if !waves_mode_activated {
            gui::print_box(&center_box, &mut lcd); //currently not needed
            //gui::print_box(&smoothing_box, &mut lcd); //currently not needed
        }

        //NOTE: user interaction due to unsolved problems in the stm32f7_discovery api not possible :(

        //CHECK USER INTERACTION (TOUCH)
//        for touch in &touch::touches(&mut i2c_3).unwrap() {
//            match waves_mode_activated {
//                true => { //check if display should change mode again
//                    if gui::is_in_box(touch.x, touch.y, &view_mode_toggle_box) && gui::is_cooled_down(view_mode_last_toggled) {
//                        waves_mode_activated = false;
//                        lcd.clear_screen();
//                        view_mode_last_toggled = system_clock::ticks();
//                    }
//                }
//                false => { //check if new smooth_strength is updated or view_mode_toggle_box is pressed
//                    if gui::is_in_box(touch.x, touch.y, &smoothing_box) {
//                        //smooth_strength = (touch.y - smoothing_box.start.y as u16) * gui::SMOOTH_MULTIPLIER; //not in use
//                    } else if gui::is_in_box(touch.x, touch.y, &view_mode_toggle_box) && gui::is_cooled_down(view_mode_last_toggled) {
//                        waves_mode_activated = true;
//                        lcd.clear_screen();
//                        view_mode_last_toggled = system_clock::ticks();
//                    }
//                }
//            }
//        }

        //---

        //DISPLAY COMPUTED AUDIO DATA
        if !waves_mode_activated { //displaying for vector mode
            //remove old vector
            gui::remove_vector(&mut audio_main_vec, &mut lcd);
            //smooth sinus_alpha until smoothing_data_strength, then update audio_main_vec
            if sinus_alpha < 1.0 && sinus_alpha > -1.0 && sinus_alpha != 0.0 {
                smoothing_data_sumup += sinus_alpha;
                smoothing_data_counter += 1;
                if smoothing_data_counter == smoothing_data_strength { //now update audio_main_vec
                    smoothing_data_avg = smoothing_data_sumup / smoothing_data_strength as f32;
                    smoothing_data_sumup = smoothing_data_avg;
                    smoothing_data_counter = 0;
                    audio_main_vec = *(gui::calculate_vector(&mut audio_main_vec, smoothing_data_avg));
                }
            }    
            //print vector
            gui::print_vector(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, gui::FIRST_COLOR);            
        } else {} //NOTE: Displaying for waves mode is implemented directly at audio data poll section  
    }  
}