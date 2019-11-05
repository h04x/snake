//extern crate console;
//use console::{style, Style, Term};
use rand::Rng;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::{self, Write, stdout, Stdout};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crossterm::{
    QueueableCommand, 
    cursor, 
    screen::RawScreen,
    terminal::{
        Clear,
        ClearType
    }, 
    style::{
        ContentStyle,
        StyledContent,
        style,
        Color,
        Attribute,
        Styler
    },
    input::{
        input,
        KeyEvent,
        InputEvent
    }
    };

enum MoveAct {
    Move,
    Grow,
}

#[derive(Debug, Clone, PartialEq)]
struct SnakeChain {
    x: isize,
    y: isize,
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Snake {
    move_direction: Arc<Mutex<Direction>>,
    body: VecDeque<SnakeChain>,
    chain_symbol: StyledContent<char>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Cookie {
    x: isize,
    y: isize,
}

struct Cookies {
    cookies: BTreeSet<Cookie>,
    cookie_symbol: StyledContent<char>,
    percent: f32,
}

struct SnakeGame {
    width: isize,
    height: isize,
    speed_msec: u64,
    term: Stdout,
    snake: Snake,
    cookies: Cookies,
    playground_symbol: char,
    playground_color: ContentStyle,
}

impl Direction {
    fn turn_left(&mut self) {
        use Direction::*;
        *self = match *self {
            Right => Up,
            Left => Down,
            Up => Left,
            Down => Right,
        }
    }
    fn turn_right(&mut self) {
        use Direction::*;
        *self = match *self {
            Right => Down,
            Left => Up,
            Up => Right,
            Down => Left,
        }
    }
}

impl Cookies {
    fn new(percent: f32, width: isize, height: isize, snake: &Snake) -> Cookies {
        let mut cookies = BTreeSet::new();
        let cookies_cnt = (width + height - snake.body.len() as isize) as f32 * percent;

        let mut r = rand::thread_rng();
        for _ in 0..cookies_cnt as usize {
            loop {
                let x = r.gen_range(0, width);
                let y = r.gen_range(0, height);
                let c = Cookie { x, y };
                let s = SnakeChain { x, y };
                if snake.body.contains(&s) == false && cookies.contains(&c) == false {
                    cookies.insert(c);
                    break;
                }
            }
        }

        Cookies {
            cookie_symbol: style('x').with(Color::Red),
            cookies,
            percent,
        }
    }

    fn draw(&self, term: &mut Stdout) {
        for cookie in &self.cookies {
            term.queue(cursor::MoveTo(cookie.x as u16, cookie.y as u16));
            print!("{}", self.cookie_symbol);
            term.flush();

        }
    }

    fn remove(&mut self, cookie: Cookie) {
        self.cookies.remove(&cookie);
    }

    fn add(&mut self, term: &mut Stdout, width: isize, height: isize, snake: &Snake) {
        let mut cookies_cnt =
            ((width + height - snake.body.len() as isize) as f32 * self.percent) as usize;
        if cookies_cnt < 1 {
            cookies_cnt = 1;
        }

        if cookies_cnt <= self.cookies.len() {
            return;
        }

        let mut r = rand::thread_rng();
        loop {
            let x = r.gen_range(0, width);
            let y = r.gen_range(0, height);
            let c = Cookie { x, y };
            let s = SnakeChain { x, y };
            if snake.body.contains(&s) == false && self.cookies.contains(&c) == false {
                self.cookies.insert(c);
                term.queue(cursor::MoveTo(x as u16, y as u16));
                print!("{}", self.cookie_symbol);
                term.flush();


                break;
            }
        }
    }
}

impl Snake {
    fn new(left: isize, top: isize, len: usize, move_direction: Direction) -> Snake {
        let mut body = VecDeque::new();
        match move_direction {
            Direction::Left => {
                for i in 0..len as isize {
                    body.push_front(SnakeChain {
                        x: left - i,
                        y: top,
                    });
                }
            }
            Direction::Right => {
                for i in 0..len as isize {
                    body.push_front(SnakeChain {
                        x: left + i,
                        y: top,
                    });
                }
            }
            Direction::Up => {
                for i in 0..len as isize {
                    body.push_front(SnakeChain {
                        x: left,
                        y: top - i,
                    });
                }
            }
            Direction::Down => {
                for i in 0..len as isize {
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
            chain_symbol: style('o').with(Color::Cyan),
        }
    }

    fn draw(&self, term: &mut Stdout) {
        for snake_chain in &self.body {
            term.queue(cursor::MoveTo(snake_chain.x as u16, snake_chain.y as u16));
            print!("{}", self.chain_symbol);
        }
        term.flush();

    }

    fn cut_tail(&mut self, term: &mut Stdout, hide: &StyledContent<char>) {
        let tail = self.body.pop_back().unwrap();
        term.queue(cursor::MoveTo(tail.x as u16, tail.y as u16));
        print!("{}", hide);
        term.flush();

    }

    fn add_head(&mut self, term: &mut Stdout) {
        let mut head = self.body[0].clone();
        match *self.move_direction.lock().unwrap() {
            Direction::Left => {
                head.x -= 1;
            }
            Direction::Right => {
                head.x += 1;
            }
            Direction::Up => {
                head.y -= 1;
            }
            Direction::Down => {
                head.y += 1;
            }
        }
        term.queue(cursor::MoveTo(head.x as u16, head.y as u16));
        print!("{}", self.chain_symbol);
        term.flush();
        self.body.push_front(head);
    }

    fn _move(&mut self, mut term: &mut Stdout, hide: &StyledContent<char>, act: MoveAct) {
        if let MoveAct::Move = act {
            self.cut_tail(&mut term, hide);
        }
        self.add_head(&mut term);
    }
}

impl SnakeGame {
    fn new(width: isize, height: isize, speed_msec: u64) -> SnakeGame {
        let snake = Snake::new(
            0,          // tail pos x
            height / 2, // tail pos y
            7,          // len
            Direction::Right,
        );

        let cookies = Cookies::new(
            0.3, //cookies fill percent of free space
            width, height, &snake,
        );

        let term = stdout();
        let _raw = RawScreen::into_raw_mode();
        
        SnakeGame {
            width,
            height,
            speed_msec,
            term,
            snake,
            cookies,
            playground_symbol: '.',
            playground_color: ContentStyle::new(),
        }
    }

    fn start_key_press_handler(&self) {
        let mut rt = input().read_sync();
        let dr = self.snake.move_direction.clone();
        thread::spawn(move || loop {
            if let Some(r) = rt.next() {
                match r {
                    InputEvent::Keyboard(KeyEvent::Left) => {
                        dr.lock().unwrap().turn_left();
                    }
                    InputEvent::Keyboard(KeyEvent::Right) => {
                        dr.lock().unwrap().turn_right();
                    }
                    _ => {}
                }
            }
        });
    }

    #[allow(dead_code)]
    fn crazy_ivan(&self) {
        if rand::thread_rng().gen_range(0.0, 1.0) < 0.1 {
            if rand::thread_rng().gen_range(0, 2) == 1 {
                self.snake.move_direction.lock().unwrap().turn_left();
            } else {
                self.snake.move_direction.lock().unwrap().turn_right();
            }
        }
    }

    fn snake_meet_barrier(&mut self) -> bool {
        let head = self.snake.body.pop_front().unwrap();
        let mut ret = false;
        if head.x > self.width - 1 || head.x < 0 {
            ret = true;
        }
        if head.y > self.height - 1 || head.y < 0 {
            ret = true;
        }

        if self.snake.body.contains(&head) {
            ret = true;
        }

        self.snake.body.push_front(head);
        return ret;
    }

    fn snake_meet_cookie(&self) -> Option<Cookie> {
        let head = self.snake.body[0].clone();
        let cookie = Cookie {
            x: head.x,
            y: head.y,
        };
        if self.cookies.cookies.contains(&cookie) {
            return Some(cookie);
        }
        None
    }

    fn play(&mut self) {
        self.draw_playground();
        self.snake.draw(&mut self.term);
        self.cookies.draw(&mut self.term);

        self.start_key_press_handler();

        let hide = self.playground_color.apply(self.playground_symbol);
        let mut act = MoveAct::Move;
        loop {
            self.snake._move(&mut self.term, &hide, act);
            act = MoveAct::Move;
            if self.snake_meet_barrier() {
                break;
            }
            if let Some(cookie) = self.snake_meet_cookie() {
                act = MoveAct::Grow;
                self.cookies.remove(cookie);
                self.cookies
                    .add(&mut self.term, self.width, self.height, &self.snake);
            }
            thread::sleep(Duration::from_millis(self.speed_msec));
        }
    }

    fn draw_playground(&mut self) {        
        self.term.queue(Clear(ClearType::All));
        self.term.queue(cursor::MoveTo(0, 0));
        self.term.queue(cursor::Hide);
        self.term.flush();
        let r = self
            .playground_symbol
            .to_string()
            .repeat(self.width as usize);
        for _ in 0..self.height {
            println!("{}", self.playground_color.apply(&r));
            self.term.flush();
        }
    }
}

fn main() {
    let mut game = SnakeGame::new(
        100, //playgroud width
        15,  //playground height
        100, //speed delay (ms)
    );
    game.play();
}
