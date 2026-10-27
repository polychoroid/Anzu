use anzu_runtime::display::{DisplayAdapter, DrawBatch};

pub struct CrossTermDisplayAdapter;

impl DisplayAdapter for CrossTermDisplayAdapter {
    type Target = Option<String>;

    type Backend = Option<String>;

    fn connect_target(&mut self, _target: Self::Target) {
        ()
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        ()
    }

    fn render(&mut self, _data: Vec<DrawBatch>) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn render_if_visible(
        &mut self,
        width: u32,
        height: u32,
        data: Vec<DrawBatch>,
    ) -> Result<(), std::io::Error> {
        if width > 0 && height > 0 {
            self.render(data)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid scrren size.",
            ))
        }
    }
}
