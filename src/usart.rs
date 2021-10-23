type LedPin = gpiob::PB0<Output<PushPull>>;

static LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));
static TIM: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));

// u32 0xE000ED14
// clear bit 3

#[interrupt]
fn TIM2() {
    static mut L_LED: Option<LedPin> = None;
    static mut L_TIM: Option<CountDownTimer<TIM2>> = None;

    let led = L_LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| LED.borrow(cs).replace(None).unwrap())
    });

    let tim = L_TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| TIM.borrow(cs).replace(None).unwrap())
    });

    let _ = led.toggle();
    let _ = tim.wait();
}

/* #[interrupt]
fn DMA */

fn run() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).pclk1(8.mhz()).freeze();

    let gpiob = dp.GPIOB.split();
    let mut led = gpiob.pb0.into_push_pull_output();
    let _ = led.set_high();

    irq::free(|cs| *LED.borrow(&cs).borrow_mut() = Some(led));

    let mut timer = Timer::new(dp.TIM2, &clocks).start_count_down(1.hz());

    timer.listen(Event::TimeOut);

    irq::free(|cs| *TIM.borrow(&cs).borrow_mut() = Some(timer));

    unsafe {
        peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);

    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa9;

    // configure serial
    let mut tx = Serial::tx(dp.USART1, tx_pin, Config::default(), clocks).unwrap();

    let mut value: u8 = 0;

    /* loop {
        wfi();
    } */

    loop {
        //wfi();
        // print some value every 500 ms, value will overflow after 255
        value += 1;
        writeln!(tx, "Values: {}", value).unwrap();
        delay.delay_ms(500_u32);
    }
}
