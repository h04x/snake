extern crate console;
use console::{style, Style, StyledObject, Term};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct SnakeChain {
    x: usize,
    y: usize,
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Top,
    Down,
}

struct Snake {
    move_direction: Arc<Mutex<Direction>>,
    body: VecDeque<SnakeChain>,
    chain_symbol: console::StyledObject<char>,
}

struct SnakeGame {
    width: usize,
    height: usize,
    speed_msec: u64,
    term: Term,
    snake: Snake,
    playground_symbol: char,
    playground_color: console::Style,
}

impl Direction {
    fn turn_left(&mut self) {
        use Direction::*;
        *self = match *self {
            Right => Top,
            Left => Down,
            Top => Left,
            Down => Right,
        }
    }    
    fn turn_right(&mut self) {
        use Direction::*;
        *self = match *self {
            Right => Down,
            Left => Top,
            Top => Right,
            Down => Left,
        }
    }    
} 

impl Snake {
    fn new(left: usize, top: usize, len: usize, move_direction: Direction) -> Snake {
        let mut body = VecDeque::new();
        match move_direction {
            Direction::Left => {
                for i in 0..len {
                    body.push_front(SnakeChain {
                        x: left - i,
                        y: top,
                    });
                }
            }
            Direction::Right => {
                for i in 0..len {
                    body.push_front(SnakeChain {
                        x: left + i,
                        y: top,
                    });
                }
            }
            Direction::Top => {
                for i in 0..len {
                    body.push_front(SnakeChain {
                        x: left,
                        y: top - i,
                    });
                }
            }
            Direction::Down => {
                for i in 0..len {
                    body.push_front(SnakeChain {
                        x: left - i,
                        y: top + i,
                    });
                }
            }
        }
        let move_direction = Arc::new(Mutex::new(move_direction));
        Snake {
            move_direction,
            body,
            chain_symbol: style('o').cyan(),
        }
    }

    fn draw(&self, term: &Term) {
        for snake_chain in &self.body {
            term.move_cursor_to(snake_chain.x, snake_chain.y);
            print!("{}", self.chain_symbol);
        }
        io::stdout().flush().unwrap();
    }

    fn _move(&mut self, term: &Term, hide: &console::StyledObject<char>) {
        let tail = self.body.pop_back().unwrap();
        term.move_cursor_to(tail.x, tail.y);
        print!("{}", hide);
        io::stdout().flush().unwrap();
        let mut head = self.body[0].clone();
        //let mutex_guard = self.move_direction.lock();
       // println!("{:?}", self.move_direction.lock().unwrap().unwrap());
        match *self.move_direction.lock().unwrap() {               
            Direction::Left => {
                head.x = head.x - 1;
            }
            Direction::Right => {
                head.x = head.x + 1;
            }
            Direction::Top => {
                head.y = head.y - 1;
            }
            Direction::Down => {
                head.y = head.y + 1;
            }
        }
        term.move_cursor_to(head.x, head.y);
        print!("{}", self.chain_symbol);
        io::stdout().flush().unwrap();
        self.body.push_front(head);
    }
}

impl SnakeGame {
    fn new(width: usize, height: usize, speed_msec: u64) -> SnakeGame {
        let snake = Snake::new(0, height / 2, 7, Direction::Right);
        SnakeGame {
            width,
            height,
            speed_msec,
            term: Term::stdout(),
            snake,
            playground_symbol: '.',
            playground_color: Style::new().white(),
        }
    }

    fn start_key_press_handler(&self) {
        let rt = self.term.clone();
        let dr = self.snake.move_direction.clone();
        thread::spawn(move || loop {
            match rt.read_key() {
                Ok(r) => {
                    match r {
                        console::Key::ArrowLeft => {
                            dr.lock().unwrap().turn_left();
                        },
                        console::Key::ArrowRight => {
                            dr.lock().unwrap().turn_right();
                        },
                        _ => {}
                    }
                },                
                _ => {}
            }
        });
    }

    fn play(&mut self) {
        self.draw_playground();
        self.snake.draw(&self.term);
        
        self.start_key_press_handler();

        
        let hide = self.playground_color.apply_to(self.playground_symbol);
        loop {
            self.snake._move(&self.term, &hide);
            thread::sleep(Duration::from_millis(self.speed_msec));
        }
        
        /*let hide = self.playground_color.apply_to(self.playground_symbol);
        loop {
            self.snake._move(&self.term, &hide);
            if rand::thread_rng().gen_range(0.0, 1.0) < 0.1 {
                match self.snake.move_direction {
                    Direction::Left | Direction::Right => {
                        if rand::thread_rng().gen_range(0, 2) == 0{
                            self.snake.move_direction = Direction::Top;
                        } else {
                            self.snake.move_direction = Direction::Down;
                        }
                    },
                    Direction::Top | Direction::Down => {
                        if rand::thread_rng().gen_range(0, 2) == 0{
                            self.snake.move_direction = Direction::Right;
                        } else {
                            self.snake.move_direction = Direction::Left;
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(self.speed_msec));
        }*/
    }

    fn draw_playground(&self) {
        self.term.clear_screen();
        let r = self.playground_symbol.to_string().repeat(self.width);
        for _ in 0..self.height {
            println!("{}", self.playground_color.apply_to(&r));
        }
    }
}

fn main() {
    let mut game = SnakeGame::new(100, 15, 100);
    game.play();
}
