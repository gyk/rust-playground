/// Makes some noise when dropped.
pub struct Bomb {
    label: String,
}

impl Bomb {
    pub fn new(label: String) -> Bomb {
        Bomb {
            label
        }
    }
}

impl Drop for Bomb {
    fn drop(&mut self) {
        println!("Boom! from {}", self.label);
    }
}
