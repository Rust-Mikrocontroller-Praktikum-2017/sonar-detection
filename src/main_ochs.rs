#![no_std]
#![no_main]
#[warn(unused_unsafe)]

extern crate stm32f7_discovery as stm32f7;
// initialization routines for .data and .bss
extern crate r0;
//extern crate std::f32;

use stm32f7::{system_clock, sdram, lcd, i2c, touch, board, embedded};


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
    lcd.set_background_color(lcd::Color::from_hex(gui::BACKGROUND_COLOR as u32));

    let aud_main_vec_anchor = gui::init_point(gui::X_DIM_RES/2, gui::Y_DIM_RES/2);
    let mut audio_main_vec = gui::init_vector(126, 0);
    let center_box = gui::init_box(gui::init_point(aud_main_vec_anchor.x - 5, aud_main_vec_anchor.y - 5), 10, 10, gui::FIRST_COLOR);
    let smoothing_box =  gui::init_box(gui::init_point(20, 5), 20, 200, gui::SECOND_COLOR);
    let view_mode_toggle_box = gui::init_box(gui::init_point(20, 217), 50, 50, gui::THIRD_COLOR);

    let mut smooth_strength: u16 = 1;
    let mut waves_mode_activated: bool = false;

    let mut sinus_alpha = 0; //alpha = 0

    loop {
        //GUI ENVIRO SETUPS AND UPDATES
        gui::print_box(&view_mode_toggle_box, &mut lcd);
        if !waves_mode_activated {
            gui::print_box(&center_box, &mut lcd);
            gui::print_box(&smoothing_box, &mut lcd);
        }

        //CHECK USER INTERACTION (TOUCH)
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            match waves_mode_activated {
                true => { //check if display should change mode again
                    if gui::is_in_box(touch.x, touch.y, &view_mode_toggle_box) {
                        waves_mode_activated = false;
                    }
                }
                false => { //check if new smooth_strength is requested or view_mode_toggle_box is pressed
                    if gui::is_in_box(touch.x, touch.y, &smoothing_box) {
                        smooth_strength = (touch.y - smoothing_box.start.y as u16) * smooth_multiplier;
                    } else if gui::is_in_box(touch.x, touch.y, &view_mode_toggle_box) {
                        waves_mode_activated = true;
                        lcd.clear_screen();
                    }
                }
            }
        }


        //DISPLAY COMPUTED AUDIO DATA
        if !waves_mode_activated { //displaying for vector mode
            //remove old vector
            gui::remove_vector(&mut audio_main_vec, &mut lcd);
            //calculate updated vector
            audio_main_vec = gui::calculate_vector(&audio_main_vec, sinus_alpha);
            //print updated vector
            gui::print_vector(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, gui::FIRST_COLOR);
        } else {} //NOTE: Displaying for waves mode is implemented directly at audio data poll section

    }
}