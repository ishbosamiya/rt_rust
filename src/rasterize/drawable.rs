pub trait Drawable {
    type ExtraData;
    type Error;

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), Self::Error>;
    fn draw_wireframe(&self, _extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        eprintln!("error: draw_wireframe() not implemented but called");
        Ok(())
    }
}
