pub struct TempCalibration {
    pub adc_value: u16,
    /// In degrees celcius.
    pub temperature: f32,
}

pub struct TempConverter {
    pull_up_r: f32,
    r0: f32,
    t0_recip: f32,
    b_recip: f32,
}

impl TempConverter {
    pub fn new(pull_up_r: f32, cal0: TempCalibration, cal1: TempCalibration) -> Self {
        let t0_recip = cal0.temperature.to_kelvin().recip();
        let r0 = adc_value_to_resistance(pull_up_r, cal0.adc_value);

        let t1_recip = cal1.temperature.to_kelvin().recip();
        let r1 = adc_value_to_resistance(pull_up_r, cal1.adc_value);

        let b_recip = (t1_recip - t0_recip) / (r1 / r0).ln();

        Self {
            pull_up_r,
            t0_recip,
            r0,
            b_recip,
        }
    }

    pub fn convert(&self, adc_value: u16) -> f32 {
        let r = adc_value_to_resistance(self.pull_up_r, adc_value);

        let t_recip = self.t0_recip + self.b_recip * (r / self.r0).ln();

        t_recip.recip().to_celcius()
    }
}

fn adc_value_to_resistance(pull_up_r: f32, adc_value: u16) -> f32 {
    let q = (adc_value as f32) / (((1u16 << 12) - 1) as f32);
    pull_up_r / q - pull_up_r
}

trait UnitConversion {
    fn to_kelvin(self) -> Self;
    fn to_celcius(self) -> Self;
}

impl UnitConversion for f32 {
    fn to_kelvin(self) -> f32 {
        self + 273.15
    }

    fn to_celcius(self) -> f32 {
        self - 273.15
    }
}
