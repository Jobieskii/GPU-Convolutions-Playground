pub(crate) fn empty_board(width: u32, height: u32) -> Vec<Vec<f32>> {
    let mut board: Vec<Vec<f32>> = Vec::with_capacity(height.try_into().unwrap());
    for _y in 0..height {
        let mut row = Vec::with_capacity(width.try_into().unwrap());
        for _x in 0..width {
            row.push(0.);
        }
        board.push(row);
    }
    board
}

pub(crate) fn random_board(width: u32, height: u32) -> Vec<Vec<f32>> {
    let mut board: Vec<Vec<f32>> = empty_board(width, height);
    for row in board.iter_mut() {
        for cell in row {
            *cell = rand::random();
        }
    }
    board
}

pub(crate) fn random_board_binary(width: u32, height: u32) -> Vec<Vec<f32>> {
    let mut board: Vec<Vec<f32>> = empty_board(width, height);
    for row in board.iter_mut() {
        for cell in row {
            *cell = if rand::random::<f32>() > 0.5 { 1. } else { 0. };
        }
    }
    board
}
