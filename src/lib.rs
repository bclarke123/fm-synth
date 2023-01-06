pub mod octave {

    pub type Note = i32;

    const BASE_FREQ: f64 = 220.0;

    pub const A: Note = 0;
    pub const A_SHARP: Note = 1;
    pub const B: Note = 2;
    pub const C: Note = 3;
    pub const C_SHARP: Note = 4;
    pub const D: Note = 5;
    pub const D_SHARP: Note = 6;
    pub const E: Note = 7;
    pub const F: Note = 8;
    pub const F_SHARP: Note = 9;
    pub const G: Note = 10;
    pub const A_FLAT: Note = 11;

    pub fn freq(note: Note, octave: i32) -> f64 {
        let octave_notes = octave * 12;
        let notes = (octave_notes + note) as f64;
        let pow = notes / 12.0;

        BASE_FREQ * 2_f64.powf(pow)
    }
}
