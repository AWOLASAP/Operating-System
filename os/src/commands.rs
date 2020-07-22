use crate::println;


pub struct CommandRunner{
    command_buffer: str,
}

impl CommandRunner {
    fn new() -> CommandRunner {
        CommandRunner{
            command_buffer: "",
        }

    }

    pub fn add_to_buffer(&mut self, c: char){
        self.command_buffer = self.command_buffer.to_owned();
        let borrowed_string: &str = c.to_string();

        self.command_buffer.push_str(borrowed_string);
    }

    pub fn echo(&mut self, string: &str) {
        println!("\n{}", string);
    }
}
