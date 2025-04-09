#[derive(Copy, Clone, Debug, PartialEq, Default, Eq, Hash)]
pub struct TextCoordinate {
    pub x: u32,
    pub y: u32,
}

impl TextCoordinate {
    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }
    pub fn left(&self) -> Self {
        Self {
            x: self.x.saturating_sub(1),
            y: self.y,
        }
    }
    pub fn up(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.saturating_sub(1),
        }
    }
    pub fn down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    pub fn shifted(self, origin: TextCoordinate, move_pos: TextCoordinate) -> TextCoordinate {
        let delta_x = move_pos.x as i32 - origin.x as i32;
        let delta_y = move_pos.y as i32 - origin.y as i32;
        TextCoordinate {
            x: self.x.saturating_add_signed(delta_x),
            y: self.y.saturating_add_signed(delta_y),
        }
    }
}
