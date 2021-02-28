use {
    std::io::{
        prelude::*,
        stdin,
        stdout,
    },
    rand::Rng,
};

pub trait QwwIteratorExt: Iterator + Sized {
    fn rand<R: Rng>(self, rng: &mut R) -> Option<Self::Item> {
        let mut v = self.collect::<Vec<_>>();
        if v.is_empty() {
            None
        } else {
            let len = v.len();
            Some(v.swap_remove(rng.gen_range(0..len)))
        }
    }
}

impl<T: Iterator> QwwIteratorExt for T {}

pub fn input(msg: &str) -> String {
    print!("[ ?? ] {}: ", msg);
    stdout().flush().expect("failed to flush stdout");
    let mut result = String::new();
    stdin().read_line(&mut result).expect("failed to read mod input");
    assert_eq!(result.pop(), Some('\n'));
    result
}
