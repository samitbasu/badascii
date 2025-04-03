use crate::lib::tc::TextCoordinate;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Rectangle {
    pub corner_1: TextCoordinate,
    pub corner_2: TextCoordinate,
}

impl Rectangle {
    pub fn new(corner_1: TextCoordinate, corner_2: TextCoordinate) -> Rectangle {
        Rectangle { corner_1, corner_2 }
    }
    pub fn contains(&self, coord: &TextCoordinate) -> bool {
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_x = self.corner_1.x.max(self.corner_2.x);
        let min_y = self.corner_1.y.min(self.corner_2.y);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        (min_x..=max_x).contains(&coord.x) && (min_y..=max_y).contains(&coord.y)
    }

    pub fn shifted(self, origin: TextCoordinate, move_pos: TextCoordinate) -> Self {
        Self {
            corner_1: self.corner_1.shifted(origin, move_pos),
            corner_2: self.corner_2.shifted(origin, move_pos),
        }
    }

    pub fn iter_interior(&self) -> impl Iterator<Item = TextCoordinate> {
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_x = self.corner_1.x.max(self.corner_2.x);
        let min_y = self.corner_1.y.min(self.corner_2.y);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        (min_y..=max_y).flat_map(move |y| (min_x..=max_x).map(move |x| TextCoordinate { x, y }))
    }

    pub fn height(&self) -> u32 {
        let y_min = self.corner_1.y.min(self.corner_2.y);
        let y_max = self.corner_1.y.max(self.corner_2.y);
        y_max - y_min + 1
    }
    pub fn width(&self) -> u32 {
        let x_min = self.corner_1.x.min(self.corner_2.x);
        let x_max = self.corner_1.x.max(self.corner_2.x);
        x_max - x_min + 1
    }

    pub fn left(&self) -> u32 {
        self.corner_1.x.min(self.corner_2.x)
    }

    pub fn top(&self) -> u32 {
        self.corner_1.y.min(self.corner_2.y)
    }
    pub fn normalize(self) -> Self {
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_x = self.corner_1.x.max(self.corner_2.x);
        let min_y = self.corner_1.y.min(self.corner_2.y);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        Self {
            corner_1: TextCoordinate { x: min_x, y: min_y },
            corner_2: TextCoordinate { x: max_x, y: max_y },
        }
    }
}
