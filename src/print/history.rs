use dialoguer::History;
use std::collections::VecDeque;

#[derive(Default)]
pub struct InputHistory {
    history: VecDeque<String>,
}

impl<T: ToString> History<T> for InputHistory {
    fn read(
        &self,
        pos: usize,
    ) -> Option<String> {
        self.history
            .get(pos)
            .cloned()
    }

    fn write(
        &mut self,
        val: &T,
    ) {
        self.history
            .push_front(val.to_string());
    }
}
