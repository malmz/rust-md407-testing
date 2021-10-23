use nb::block;
struct Keypad {
    p0: PE0<Input<PullDown>>,
    p1: PE1<Input<PullDown>>,
    p2: PE2<Input<PullDown>>,
    p3: PE3<Input<PullDown>>,

    p4: PE4<Output<PushPull>>,
    p5: PE5<Output<PushPull>>,
    p6: PE6<Output<PushPull>>,
    p7: PE7<Output<PushPull>>,

    pressed_keys: [[bool; 4]; 4],
    current_frame: [[bool; 4]; 4],
}

const KEYS: [[char; 4]; 4] = [
    ['1', '2', '3', 'A'],
    ['4', '5', '6', 'B'],
    ['7', '8', '9', 'C'],
    ['*', '0', '#', 'D'],
];

impl Keypad {
    fn new(
        p0: PE0<Input<PullDown>>,
        p1: PE1<Input<PullDown>>,
        p2: PE2<Input<PullDown>>,
        p3: PE3<Input<PullDown>>,

        p4: PE4<Output<PushPull>>,
        p5: PE5<Output<PushPull>>,
        p6: PE6<Output<PushPull>>,
        p7: PE7<Output<PushPull>>,
    ) -> Self {
        Self {
            p0,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
            pressed_keys: Default::default(),
            current_frame: Default::default(),
        }
    }

    fn activate_row(&mut self, row: usize) {
        match row {
            0 => self.p4.set_high(),
            1 => self.p5.set_high(),
            2 => self.p6.set_high(),
            3 => self.p7.set_high(),
            _ => {}
        }
    }

    fn read_col(&self, col: usize) -> bool {
        match col {
            0 => self.p0.is_high(),
            1 => self.p1.is_high(),
            2 => self.p2.is_high(),
            3 => self.p3.is_high(),
            _ => false,
        }
    }

    fn clear_rows(&mut self) {
        self.p4.set_low();
        self.p5.set_low();
        self.p6.set_low();
        self.p7.set_low();
    }

    fn read_key(&mut self) -> nb::Result<char, Infallible> {
        let mut key_count = 0;
        let mut val = '\0';

        for row in 0..4 {
            self.clear_rows();
            self.activate_row(row);
            for col in 0..4 {
                if self.read_col(col) {
                    self.current_frame[row][col] = true;
                    if !self.pressed_keys[row][col] {
                        self.pressed_keys[row][col] = true;
                        key_count += 1;
                        val = KEYS[row][col];
                    }
                } else {
                    self.current_frame[row][col] = true;
                    self.pressed_keys[row][col] = false;
                }
            }
        }

        let mut debounce = false;
        'outer: for row in 0..4 {
            self.clear_rows();
            self.activate_row(row);
            for col in 0..4 {
                if self.current_frame[row][col] != self.read_col(col) {
                    debounce = true;
                    break 'outer;
                }
            }
        }

        if debounce {
            return Ok('-');
        }

        if key_count == 1 && !debounce {
            Ok(val)
        } else {
            Err(Error::WouldBlock)
        }
    }
}

fn init_usart(dp: Peripherals) -> Tx<USART1> {
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).pclk1(8.mhz()).freeze();

    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa9;
    Serial::tx(dp.USART1, tx_pin, Config::default(), clocks).unwrap()
}

fn run2() -> ! {
    //static BUF: &[u8] = "Hello world".as_bytes();

    let dp = Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).pclk1(8.mhz()).freeze();
    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa9;

    let mut tx: Tx<USART1> = Serial::tx(dp.USART1, tx_pin, Config::default(), clocks).unwrap();

    /* let dma2_streams = StreamsTuple::new(dp.DMA2);
       let conf = DmaConfig::default()
           .memory_increment(true)
           .transfer_complete_interrupt(true);

       let mut transfer = Transfer::init_memory_to_peripheral(dma2_streams.7, tx, BUF, None, conf);
    */

    let gpioe = dp.GPIOE.split();

    let mut keypad = Keypad::new(
        gpioe.pe0.into(),
        gpioe.pe1.into(),
        gpioe.pe2.into(),
        gpioe.pe3.into(),
        gpioe.pe4.into(),
        gpioe.pe5.into(),
        gpioe.pe6.into(),
        gpioe.pe7.into(),
    );

    writeln!(tx, "\nInitialized").unwrap();

    loop {
        let key = block!(keypad.read_key()).unwrap();
        writeln!(tx, "{}", key).unwrap();
    }
}
