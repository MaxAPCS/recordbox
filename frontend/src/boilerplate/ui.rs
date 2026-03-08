pub trait Scene {
    fn update(&self);
    fn elements(&self) -> Vec<&Element>;
}

pub struct Element {
    pub shape: Trapezoid,
    pub image: image::RgbaImage,
}

impl Element {
    pub fn corners(&self) -> [[f32; 2]; 4] {
        self.shape.corners()
    }
}

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
