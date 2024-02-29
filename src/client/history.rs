#[derive(Debug, Clone)]
pub struct History {
    pub index: i32,
    pub histo: Vec<String>,
}

impl History {
    pub fn initialize() -> Self {
        let str = String::new();

        History {
            index: 0,
            histo: vec![str],
        }
    }

    pub fn add(&mut self, command: String) {
        let size =  self.histo.len();

        self.index = size as i32;
        self.histo[size - 1] = command.clone();
        self.histo.push(String::new());
    }

    pub fn get(&mut self, index: i32) -> &String {
        let histo_len = self.histo.len();

        if index < 0 {
            return &self.histo[0]
        } else if index >= histo_len as i32 {
            return &self.histo[histo_len - 1]
        } else if index >= 0 {
            self.index = index;
        }
        &self.histo[index as usize]
    }

    #[warn(dead_code)]
    pub fn debug(&self) {
        let mut index = 0;

        for line in self.histo.clone() {
            index += 1;
            print!("[{}]{}", index, line);
        }
    }

}
