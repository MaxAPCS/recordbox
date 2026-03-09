pub trait Scene {
    fn update(&self);
    fn handle_key(&self, key: winit::keyboard::KeyCode, pressed: bool);
    fn handle_mouse(&self, button: winit::event::MouseButton, location: (f32, f32), pressed: bool);
    fn elements(&self) -> Vec<std::borrow::Cow<'_, Element>>;
}

#[derive(Clone)]
pub struct Element {
    pub shape: Trapezoid,
    pub image: image::RgbaImage,
}

impl Element {
    pub fn corners(&self) -> [[f32; 2]; 4] {
        self.shape.corners()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Trapezoid {
    l_x: f32,
    r_x: f32,
    bl_y: f32,
    tl_y: f32,
    br_y: f32,
    tr_y: f32,
}

impl Trapezoid {
    fn corners(&self) -> [[f32; 2]; 4] {
        [
            [self.l_x, self.bl_y], // Bottom Left
            [self.r_x, self.br_y], // Bottom Right
            [self.l_x, self.tl_y], // Top Left
            [self.r_x, self.tr_y], // Top Right
        ]
    }

    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        if x < self.l_x || x > self.r_x {
            return false;
        }
        let min_y = self.bl_y.min(self.br_y);
        let max_y = self.tl_y.max(self.tr_y);
        if y < min_y || y > max_y {
            return false;
        }

        let dx = self.r_x - self.l_x;
        if dx == 0.0 {
            return false;
        }
        let y_bottom = self.bl_y + (self.br_y - self.bl_y) * (x - self.l_x) / dx;
        let y_top = self.tl_y + (self.tr_y - self.tl_y) * (x - self.l_x) / dx;

        y >= y_bottom && y <= y_top
    }

    pub fn from_square(cx: f32, cy: f32, s: f32) -> Self {
        Trapezoid {
            l_x: cx - s,
            r_x: cx + s,
            bl_y: cy - s,
            tl_y: cy + s,
            br_y: cy - s,
            tr_y: cy + s,
        }
    }
}

impl Default for Trapezoid {
    fn default() -> Self {
        Self::from_square(0., 0., 1.)
    }
}
