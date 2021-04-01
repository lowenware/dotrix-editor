use dotrix::{
    input::{ Button, KeyCode, Mapper },
    services::{ Input },
};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// All bindable actions
pub enum Action {
    Brush,
    Move,
}

pub fn init(input: &mut Input) {
    input.mapper_mut::<Mapper<Action>>()
        .set(vec![
            (Action::Brush, Button::MouseLeft),
            (Action::Move, Button::Key(KeyCode::W)),
        ]);
}
