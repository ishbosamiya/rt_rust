pub trait Drawable<'a> {
    type ExtraData;
    type Error;

    fn draw(&'a self, extra_data: &mut Self::ExtraData) -> Result<(), Self::Error>;
    fn draw_wireframe(&'a self, _extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        eprintln!("error: draw_wireframe() not implemented but called");
        Ok(())
    }
}
