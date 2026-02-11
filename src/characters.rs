pub struct Border {
    pub tl: char, pub tr: char, pub bl: char, pub br: char,
    pub h: char,  pub v: char,
}

pub const BORDER_SINGLE: Border = Border { tl:'┌', tr:'┐', bl:'└', br:'┘', h:'─', v:'│' };
pub const BORDER_DOUBLE: Border = Border { tl:'╔', tr:'╗', bl:'╚', br:'╝', h:'═', v:'║' };
pub const BORDER_HEAVY:  Border = Border { tl:'┏', tr:'┓', bl:'┗', br:'┛', h:'━', v:'┃' };