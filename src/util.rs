pub trait FirstNul {
    fn find_first_nul(&self) -> Option<usize>;
}

impl FirstNul for Vec<u8> {
    fn find_first_nul(&self) -> Option<usize> {
        for (i, b) in self.iter().enumerate() {
            if *b == 0 {
                return Some(i);
            }
        }

        return None;
    }
}
