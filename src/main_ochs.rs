#![no_std]
#![no_main]
#[warn(unused_unsafe)]


mod filter;

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
    touch::check_family_id(&mut i2c_3).unwrap();
    
    lcd.clear_screen();
    lcd.set_background_color(lcd::Color::from_hex(BACKGROUND_COLOR as u32));

    //init audio_main_vec, set anchor, print audio_main_vec
    let aud_main_vec_anchor = gui::init_aud_main_vec_anchor();
    let mut audio_main_vec = gui::init_audio_main_vec();
    print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, FIRST_COLOR);

    //init center_box above aud_main_vec_anchor of audio_main_vec
    let center_box = gui::init_center_box();
    
    //init smoothing_box and print
    let smoothing_box =  gui::init_smoothing_box();
    print_box(&smoothing_box, &mut lcd);
    let mut smooth_strength: u16 = 1;
    let smooth_multiplier: u16 = 20;

    //init view mode and view_toogle_box and print
    let mut view_actual_waves: bool = false;
    let view_toogle_box = gui::init_view_toggle_box();
    print_box(&view_toogle_box, &mut lcd);

    loop {
        //enviro updates
        print_box(&view_toogle_box, &mut lcd);
        if !view_actual_waves {
            print_box(&center_box, &mut lcd);
            print_box(&smoothing_box, &mut lcd);
        }

        //check for touches
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            
            //remove old audio_main_vec
            print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, BACKGROUND_COLOR);
            //update deltas to difference between anchor and touch position and print audio_main_vec again
            audio_main_vec.delta_x = touch.x as i16 - aud_main_vec_anchor.x;
            audio_main_vec.delta_y = touch.y as i16 - aud_main_vec_anchor.y;
            print_vector_reposition(&mut audio_main_vec, aud_main_vec_anchor.x, aud_main_vec_anchor.y, &mut lcd, FIRST_COLOR);

            //check box touch
            if (touch.y >= 20) && (touch.y <= 70) {
                if (touch.y >= smoothing_box.start.y as u16) && (touch.y < smoothing_box.start.y as u16 + smoothing_box.length_y) {
                    smooth_strength = (touch.y - smoothing_box.start.y as u16) * smooth_multiplier;
                } else if (touch.y >= view_toogle_box.start.y as u16) && (touch.y < view_toogle_box.start.y as u16 + view_toogle_box.length_y) {
                    view_actual_waves = true;
                }
            }
        }
    }
}