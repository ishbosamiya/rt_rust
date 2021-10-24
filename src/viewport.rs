use std::convert::TryInto;

use crate::glm;

/// Viewport of the window in which the 2D/3D scene is
/// rendered. Locations in the viewport are considering top left is
/// (0.0, 0.0) and bottom right is (width, height)
pub struct Viewport {
    /// dimentions (width, height) of viewport
    dimensions: glm::TVec2<isize>,

    /// location of top left of viewport with respect to top left of window
    at_window_loc: glm::TVec2<isize>,
}

impl Viewport {
    pub fn new(dimensions: glm::TVec2<isize>, at_window_loc: glm::TVec2<isize>) -> Self {
        Self {
            dimensions,
            at_window_loc,
        }
    }

    /// Calculates the location of the given point in `self`
    pub fn calculate_location(
        &self,
        point_with_viewport: (&glm::TVec2<isize>, &Viewport),
    ) -> glm::TVec2<isize> {
        let point_loc_in_window =
            point_with_viewport.0 + point_with_viewport.1.get_at_window_loc_top_left();
        point_loc_in_window - self.get_at_window_loc_top_left()
    }

    pub fn get_width(&self) -> isize {
        self.dimensions[0]
    }

    pub fn get_height(&self) -> isize {
        self.dimensions[1]
    }

    pub fn get_dimensions(&self) -> glm::TVec2<isize> {
        self.dimensions
    }

    /// Get location of top left of the viewport in the window
    pub fn get_at_window_loc_top_left(&self) -> glm::TVec2<isize> {
        self.at_window_loc
    }

    /// Get location of bottom left of the viewport in the window
    pub fn get_at_window_loc_bottom_left(&self) -> glm::TVec2<isize> {
        let top_left = self.get_at_window_loc_top_left();
        glm::vec2(top_left[0], top_left[1] + self.get_height())
    }

    /// Get location of top right of the viewport in the window
    pub fn get_at_window_loc_top_right(&self) -> glm::TVec2<isize> {
        let top_left = self.get_at_window_loc_top_left();
        glm::vec2(top_left[0] + self.get_width(), top_left[1])
    }

    /// Get location of bottom right of the viewport in the window
    pub fn get_at_window_loc_bottom_right(&self) -> glm::TVec2<isize> {
        let top_left = self.get_at_window_loc_top_left();
        glm::vec2(
            top_left[0] + self.get_width(),
            top_left[1] + self.get_height(),
        )
    }

    /// Sets the opengl viewport
    ///
    /// Needs the Window's viewport since setting OpenGL's viewport
    /// requires distance from bottom left of the window.
    pub fn set_opengl_viewport(&self, window_viewport: &Viewport) {
        let bottom_left = self.get_at_window_loc_bottom_left();
        let window_height = window_viewport.get_height();
        unsafe {
            gl::Viewport(
                bottom_left[0].try_into().unwrap(),
                (window_height - bottom_left[1]).try_into().unwrap(),
                self.get_width().try_into().unwrap(),
                self.get_height().try_into().unwrap(),
            );
        }
    }
}
