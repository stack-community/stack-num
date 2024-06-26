use clap::{App, Arg};
use gnuplot::Figure;
use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use rand::seq::SliceRandom;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Error, Read, Write};
use std::ops::{Add, Div, Mul, Sub};
use std::path::Path;
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    let app = App::new("NumStack")
        .version("0.0.1")
        .author("Stack Programming Community")
        .about("For science technology mathematics calculations of Stack programming language distribution")
        .arg(Arg::new("script")
            .index(1)
            .value_name("FILE")
            .help("Sets the script file to execution")
            .takes_value(true))
        .arg(Arg::new("debug")
            .short('d')
            .long("debug")
            .help("Enables debug mode"));
    let matches = app.clone().get_matches();

    if let Some(script) = matches.value_of("script") {
        if matches.is_present("debug") {
            let mut stack = Executor::new(Mode::Debug);
            stack.evaluate_program(match get_file_contents(Path::new(&script.to_string())) {
                Ok(code) => code,
                Err(err) => {
                    println!("Error! {err}");
                    return;
                }
            })
        } else {
            let mut stack = Executor::new(Mode::Script);
            stack.evaluate_program(match get_file_contents(Path::new(&script.to_string())) {
                Ok(code) => code,
                Err(err) => {
                    println!("Error! {err}");
                    return;
                }
            })
        }
    } else {
        // Show a title
        println!("NumStack Programming Language");
        println!("Version {}", { app.get_version().unwrap_or("unknown") });
        let mut executor = Executor::new(Mode::Debug);
        // REPL Execution
        loop {
            let mut code = String::new();
            loop {
                let enter = input("> ");
                code += &format!("{enter}\n");
                if enter.is_empty() {
                    break;
                }
            }

            executor.evaluate_program(code)
        }
    }
}

/// Read string of the file
fn get_file_contents(name: &Path) -> Result<String, Error> {
    let mut f = File::open(name)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Get standard input
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    result.trim().to_string()
}

fn convert(number: f64) -> (isize, isize) {
    const MAX_DENOMINATOR: isize = 1_000_000;

    if number == 0.0 {
        return (0, 1);
    }

    let mut best_numerator = 0;
    let mut best_denominator = 1;
    let mut best_diff = f64::MAX;

    for denominator in 1..=MAX_DENOMINATOR {
        let numerator = (number * denominator as f64).round() as isize;
        let fraction = numerator as f64 / denominator as f64;
        let diff = (number - fraction).abs();

        if diff < best_diff {
            best_diff = diff;
            best_numerator = numerator;
            best_denominator = denominator;

            if diff == 0.0 {
                break;
            }
        }
    }

    (best_numerator, best_denominator)
}

#[derive(Debug, Clone, Copy)]
struct Fraction {
    numerator: isize,
    denominator: isize,
}

impl Fraction {
    fn new(number: f64) -> Self {
        let result = convert(number);
        let mut frac = Fraction {
            numerator: result.0,
            denominator: result.1,
        };

        frac.simplify();
        frac
    }

    fn from(value: String) -> Fraction {
        let value = value.split("/").collect::<Vec<&str>>();
        let mut fraction = Fraction {
            numerator: value.get(0).unwrap_or(&"0").parse::<isize>().unwrap_or(0),
            denominator: value.get(1).unwrap_or(&"1").parse::<isize>().unwrap_or(1),
        };
        fraction.simplify();
        fraction
    }

    fn display(&mut self) -> String {
        self.simplify();
        if self.denominator == 1 {
            self.numerator.to_string()
        } else {
            format!("{}/{}", self.numerator, self.denominator)
        }
    }

    // Function to simplify the fraction
    fn simplify(&mut self) {
        // Function to find the greatest common divisor (GCD) using Euclid's algorithm
        fn gcd(mut a: isize, mut b: isize) -> isize {
            while b != 0 {
                let temp = b;
                b = a % b;
                a = temp;
            }
            a
        }

        let gcd = gcd(self.numerator.abs(), self.denominator.abs());
        self.numerator /= gcd;
        self.denominator /= gcd;
    }

    // Function to convert the fraction to a floating-point number
    fn to_f64(&self) -> f64 {
        (self.numerator / self.denominator) as f64
    }
}

impl Add for Fraction {
    type Output = Fraction;

    fn add(self, other: Fraction) -> Fraction {
        let number = other;
        fn lcm(a: isize, b: isize) -> isize {
            let gcd = |mut a: isize, mut b: isize| {
                while b != 0 {
                    let temp = b;
                    b = a % b;
                    a = temp;
                }
                a
            };

            (a * b) / gcd(a, b)
        }

        let denominator = lcm(self.denominator, number.denominator);
        let numerator1 = self.numerator * (denominator / self.denominator);
        let numerator2 = other.numerator * (denominator / other.denominator);

        let mut result = Fraction {
            numerator: numerator1 + numerator2,
            denominator,
        };
        result.simplify();
        result
    }
}

impl Sub for Fraction {
    type Output = Fraction;

    fn sub(self, other: Fraction) -> Fraction {
        let number = other;
        fn lcm(a: isize, b: isize) -> isize {
            let gcd = |mut a: isize, mut b: isize| {
                while b != 0 {
                    let temp = b;
                    b = a % b;
                    a = temp;
                }
                a
            };

            (a * b) / gcd(a, b)
        }

        let denominator = lcm(self.denominator, number.denominator);
        let numerator1 = self.numerator * (denominator / self.denominator);
        let numerator2 = other.numerator * (denominator / other.denominator);

        let mut result = Fraction {
            numerator: numerator1 - numerator2,
            denominator,
        };
        result.simplify();
        result
    }
}

impl Mul for Fraction {
    type Output = Fraction;

    fn mul(self, other: Fraction) -> Fraction {
        let number = other;
        let mut result = Fraction {
            numerator: self.numerator * number.numerator,
            denominator: self.denominator * number.denominator,
        };
        result.simplify();
        result
    }
}

impl Div for Fraction {
    type Output = Fraction;

    fn div(self, other: Fraction) -> Fraction {
        let mut number = other;
        (number.denominator, number.numerator) = (number.numerator, number.denominator);

        let mut result = self * number;
        result.simplify();
        result
    }
}

/// Execution Mode
#[derive(Clone, Debug)]
enum Mode {
    Script, // Script execution
    Debug,  // Debug execution
}

/// Data type
#[derive(Clone, Debug)]
enum Type {
    Number(Fraction),
    String(String),
    Bool(bool),
    List(Vec<Type>),
    Matrix(Vec<Fraction>, (usize, usize)),
    Error(String),
}

/// Implement methods
impl Type {
    fn to_matrix(mx: &Vec<Fraction>, length: usize) -> String {
        let mut matrix: Vec<Vec<Fraction>> = Vec::new();
        let mut buffer: Vec<Fraction> = Vec::new();

        let mut count = 0;
        for i in mx {
            if count < length {
                buffer.push(i.clone());
                count += 1;
            } else {
                matrix.push(buffer.clone());
                buffer.clear();
                count = 1;
                buffer.push(i.clone());
            }
        }
        matrix.push(buffer.clone());

        let mut text = "{".to_string();

        for i in matrix.iter() {
            for j in i.iter() {
                text += &format!(" {},", j.clone().display())
            }
            text.remove(text.len() - 1);
            text += ";"
        }

        text.remove(text.len() - 1);
        text += " }";
        text
    }

    /// Show data to display
    fn display(&self) -> String {
        match self {
            Type::Number(num) => num.clone().display(),
            Type::String(s) => format!("({})", s),
            Type::Bool(b) => b.to_string(),
            Type::List(list) => {
                let result: Vec<String> = list.iter().map(|token| token.display()).collect();
                format!("[{}]", result.join(" "))
            }
            Type::Error(err) => format!("error:{err}"),
            Type::Matrix(mx, (_, length)) => Type::to_matrix(mx, *length),
        }
    }

    /// Get string form data
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.display(),
            Type::Bool(b) => b.to_string(),
            Type::List(l) => Type::List(l.to_owned()).display(),
            Type::Error(err) => format!("error:{err}"),
            Type::Matrix(mx, (_, length)) => Type::to_matrix(mx, *length),
        }
    }

    /// Get number from data
    fn get_number(&mut self) -> Fraction {
        match self {
            Type::String(s) => Fraction::from(s.to_owned()),
            Type::Number(i) => i.clone(),
            Type::Bool(b) => {
                if *b {
                    Fraction::new(1.0)
                } else {
                    Fraction::new(0.0)
                }
            }
            Type::List(l) => Fraction::new(l.len() as f64),
            Type::Error(e) => Fraction::new(e.parse().unwrap_or(0f64)),
            _ => Fraction::new(0f64),
        }
    }

    /// Get bool from data
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => !s.is_empty(),
            Type::Number(i) => i.to_f64() != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => !l.is_empty(),
            Type::Error(e) => e.parse().unwrap_or(false),
            _ => false,
        }
    }

    /// Get list form data
    fn get_list(&mut self) -> Vec<Type> {
        match self {
            Type::String(s) => s
                .to_string()
                .chars()
                .map(|x| Type::String(x.to_string()))
                .collect::<Vec<Type>>(),
            Type::Number(i) => vec![Type::Number(i.to_owned())],
            Type::Bool(b) => vec![Type::Bool(*b)],
            Type::List(l) => l.to_vec(),
            Type::Error(e) => vec![Type::Error(e.to_string())],
            Type::Matrix(l, _) => l
                .to_owned()
                .iter()
                .map(|x| Type::Number(x.clone()))
                .collect(),
        }
    }

    fn get_matrix(&mut self) -> (Vec<Fraction>, (usize, usize)) {
        match self {
            Type::Matrix(mx, size) => (mx.to_vec(), *size),
            _ => (vec![], (0, 0)),
        }
    }
}

/// Manage program execution
#[derive(Clone, Debug)]
struct Executor {
    stack: Vec<Type>,              // Data stack
    memory: HashMap<String, Type>, // Variable's memory
    mode: Mode,                    // Execution mode
}

impl Executor {
    /// Constructor
    fn new(mode: Mode) -> Executor {
        Executor {
            stack: Vec::new(),
            memory: HashMap::new(),
            mode,
        }
    }

    /// Output log
    fn log_print(&mut self, msg: String) {
        if let Mode::Debug = self.mode {
            print!("{msg}");
        }
    }

    /// Show variable inside memory
    fn show_variables(&mut self) {
        self.log_print("Variables {\n".to_string());
        let max = self.memory.keys().map(|s| s.len()).max().unwrap_or(0);
        for (name, value) in self.memory.clone() {
            self.log_print(format!(
                " {:>width$}: {}\n",
                name,
                value.display(),
                width = max
            ))
        }
        self.log_print("}\n".to_string())
    }

    /// Show inside the stack
    fn show_stack(&mut self) -> String {
        format!(
            "Stack〔 {} 〕",
            self.stack
                .iter()
                .map(|x| x.display())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }

    /// Parse token by analyzing syntax
    fn analyze_syntax(&mut self, code: String) -> Vec<String> {
        // Convert tabs, line breaks, and full-width spaces to half-width spaces
        let code = code.replace(['\n', '\t', '\r', '　'], " ");

        let mut syntax = Vec::new(); // Token string
        let mut buffer = String::new(); // Temporary storage
        let mut brackets = 0; // String's nest structure
        let mut parentheses = 0; // List's nest structure
        let mut braces = 0; // Matrix's nest structure
        let mut hash = false; // Is it Comment
        let mut escape = false; // Flag to indicate next character is escaped

        for c in code.chars() {
            match c {
                '\\' if !escape => {
                    escape = true;
                }
                '(' if !hash && !escape => {
                    brackets += 1;
                    buffer.push('(');
                }
                ')' if !hash && !escape => {
                    brackets -= 1;
                    buffer.push(')');
                }
                '{' if !hash && brackets == 0 && !escape => {
                    braces += 1;
                    buffer.push('{');
                }
                '}' if !hash && brackets == 0 && !escape => {
                    braces -= 1;
                    buffer.push('}');
                }
                '#' if !hash && !escape => {
                    hash = true;
                    buffer.push('#');
                }
                '#' if hash && !escape => {
                    hash = false;
                    buffer.push('#');
                }
                '[' if !hash && brackets == 0 && !escape => {
                    parentheses += 1;
                    buffer.push('[');
                }
                ']' if !hash && brackets == 0 && !escape => {
                    parentheses -= 1;
                    buffer.push(']');
                }
                ' ' if !hash && parentheses == 0 && brackets == 0 && !escape && braces == 0 => {
                    if !buffer.is_empty() {
                        syntax.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => {
                    if parentheses == 0 && brackets == 0 && !hash {
                        if escape {
                            match c {
                                'n' => buffer.push_str("\\n"),
                                't' => buffer.push_str("\\t"),
                                'r' => buffer.push_str("\\r"),
                                _ => buffer.push(c),
                            }
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        if escape {
                            buffer.push('\\');
                        }
                        buffer.push(c);
                    }
                    escape = false; // Reset escape flag for non-escape characters
                }
            }
        }

        if !buffer.is_empty() {
            syntax.push(buffer);
        }
        syntax
    }

    /// evaluate string as program
    fn evaluate_program(&mut self, code: String) {
        // Parse into token string
        let syntax: Vec<String> = self.analyze_syntax(code);

        for token in syntax {
            // Show inside stack to debug
            let stack = self.show_stack();
            self.log_print(format!("{stack} ←  {token}\n"));

            // Character vector for token processing
            let chars: Vec<char> = token.chars().collect();

            // Judge what the token is
            if let Ok(i) = token.parse::<f64>() {
                // Push number value on the stack
                self.stack.push(Type::Number(Fraction::new(i)));
            } else if token == "true" || token == "false" {
                // Push bool value on the stack
                self.stack.push(Type::Bool(token.parse().unwrap_or(true)));
            } else if chars[0] == '(' && chars[chars.len() - 1] == ')' {
                // Processing string escape
                let string = {
                    let mut buffer = String::new(); // Temporary storage
                    let mut brackets = 0; // String's nest structure
                    let mut parentheses = 0; // List's nest structure
                    let mut hash = false; // Is it Comment
                    let mut escape = false; // Flag to indicate next character is escaped

                    for c in token[1..token.len() - 1].to_string().chars() {
                        match c {
                            '\\' if !escape => {
                                escape = true;
                            }
                            '(' if !hash && !escape => {
                                brackets += 1;
                                buffer.push('(');
                            }
                            ')' if !hash && !escape => {
                                brackets -= 1;
                                buffer.push(')');
                            }
                            '#' if !hash && !escape => {
                                hash = true;
                                buffer.push('#');
                            }
                            '#' if hash && !escape => {
                                hash = false;
                                buffer.push('#');
                            }
                            '[' if !hash && brackets == 0 && !escape => {
                                parentheses += 1;
                                buffer.push('[');
                            }
                            ']' if !hash && brackets == 0 && !escape => {
                                parentheses -= 1;
                                buffer.push(']');
                            }
                            _ => {
                                if parentheses == 0 && brackets == 0 && !hash {
                                    if escape {
                                        match c {
                                            'n' => buffer.push_str("\\n"),
                                            't' => buffer.push_str("\\t"),
                                            'r' => buffer.push_str("\\r"),
                                            _ => buffer.push(c),
                                        }
                                    } else {
                                        buffer.push(c);
                                    }
                                } else {
                                    if escape {
                                        buffer.push('\\');
                                    }
                                    buffer.push(c);
                                }
                                escape = false; // Reset escape flag for non-escape characters
                            }
                        }
                    }
                    buffer
                }; // Push string value on the stack
                self.stack.push(Type::String(string));
            } else if chars[0] == '[' && chars[chars.len() - 1] == ']' {
                // Push list value on the stack
                let old_len = self.stack.len(); // length of old stack
                let slice = &token[1..token.len() - 1];
                self.evaluate_program(slice.to_string());
                // Make increment of stack an element of list
                let mut list = Vec::new();
                for _ in old_len..self.stack.len() {
                    list.push(self.pop_stack());
                }
                list.reverse(); // reverse list
                self.stack.push(Type::List(list));
            } else if chars[0] == '{' && chars[chars.len() - 1] == '}' {
                let text = token[1..token.len() - 1].to_string();

                let row = text.split(";").collect::<Vec<&str>>().len();
                let col = text.split(";").collect::<Vec<&str>>()[0]
                    .split(",")
                    .collect::<Vec<&str>>()
                    .len();

                let value = text
                    .split(|c| c == ',' || c == ';')
                    .map(|x| {
                        self.evaluate_program(x.to_string());
                        self.pop_stack().get_number()
                    })
                    .collect::<Vec<Fraction>>();
                self.stack.push(Type::Matrix(value, (row, col)))
            } else if token.starts_with("error:") {
                // Push error value on the stack
                self.stack.push(Type::Error(token.replace("error:", "")))
            } else if let Some(i) = self.memory.get(&token) {
                // Push variable's data on stack
                self.stack.push(i.clone());
            } else if chars[0] == '#' && chars[chars.len() - 1] == '#' {
                // Processing comments
                self.log_print(format!("* Comment \"{}\"\n", token.replace('#', "")));
            } else if token.contains("/") {
                // Push fraction number from literal
                self.stack.push(Type::Number(Fraction::from(token)))
            } else {
                // Else, execute as command
                self.execute_command(token);
            }
        }

        // Show inside stack, after execution
        let stack = self.show_stack();
        self.log_print(format!("{stack}\n"));
    }

    /// execute string as commands
    fn execute_command(&mut self, command: String) {
        match command.as_str() {
            // Commands of calculation

            // Addition
            "add" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a + b));
            }

            // Subtraction
            "sub" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a - b));
            }

            // Multiplication
            "mul" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a * b));
            }

            // Division
            "div" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a / b));
            }

            // Remainder of division
            "mod" => {
                let b = self.pop_stack().get_number().to_f64();
                let a = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(a % b)));
            }

            // Exponentiation
            "pow" => {
                let b = self.pop_stack().get_number().to_f64();
                let a = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(a.powf(b))));
            }

            // Rounding off
            "round" => {
                let a = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(a.round())));
            }

            // Trigonometric sine
            "sin" => {
                let number = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(number.sin())))
            }

            // Trigonometric cosine
            "cos" => {
                let number = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(number.cos())))
            }

            // Trigonometric tangent
            "tan" => {
                let number = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(number.tan())))
            }

            // Exponential function
            "exp" => {
                let number = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Number(Fraction::new(number.exp())))
            }

            // Logical operations of AND
            "and" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a && b));
            }

            // Logical operations of OR
            "or" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a || b));
            }

            // Logical operations of NOT
            "not" => {
                let b = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(!b));
            }

            // Judge is it equal
            "equal" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::Bool(a == b));
            }

            // Judge is it less
            "less" => {
                let b = self.pop_stack().get_number().to_f64();
                let a = self.pop_stack().get_number().to_f64();
                self.stack.push(Type::Bool(a < b));
            }

            // Get random value from list
            "rand" => {
                let list = self.pop_stack().get_list();
                let result = match list.choose(&mut rand::thread_rng()) {
                    Some(i) => i.to_owned(),
                    None => Type::List(list),
                };
                self.stack.push(result);
            }

            // Shuffle list by random
            "shuffle" => {
                let mut list = self.pop_stack().get_list();
                list.shuffle(&mut rand::thread_rng());
                self.stack.push(Type::List(list));
            }

            // Commands of string processing

            // Repeat string a number of times
            "repeat" => {
                let count = self.pop_stack().get_number().to_f64(); // Count
                let text = self.pop_stack().get_string(); // String
                self.stack.push(Type::String(text.repeat(count as usize)));
            }

            // Get unicode character form number
            "decode" => {
                let code = self.pop_stack().get_number().to_f64();
                let result = char::from_u32(code as u32);
                match result {
                    Some(c) => self.stack.push(Type::String(c.to_string())),
                    None => {
                        self.log_print("Error! failed of number decoding\n".to_string());
                        self.stack.push(Type::Error("number-decoding".to_string()));
                    }
                }
            }

            // Encode string by UTF-8
            "encode" => {
                let string = self.pop_stack().get_string();
                if let Some(first_char) = string.chars().next() {
                    self.stack
                        .push(Type::Number(Fraction::new((first_char as u32) as f64)));
                } else {
                    self.log_print("Error! failed of string encoding\n".to_string());
                    self.stack.push(Type::Error("string-encoding".to_string()));
                }
            }

            // Concatenate the string
            "concat" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::String(a + &b));
            }

            // Replacing string
            "replace" => {
                let after = self.pop_stack().get_string();
                let before = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::String(text.replace(&before, &after)))
            }

            // Split string by the key
            "split" => {
                let key = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::List(
                    text.split(&key)
                        .map(|x| Type::String(x.to_string()))
                        .collect::<Vec<Type>>(),
                ));
            }

            // Change string style case
            "case" => {
                let types = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();

                self.stack.push(Type::String(match types.as_str() {
                    "lower" => text.to_lowercase(),
                    "upper" => text.to_uppercase(),
                    _ => text,
                }));
            }

            // Generate a string by concat list
            "join" => {
                let key = self.pop_stack().get_string();
                let mut list = self.pop_stack().get_list();
                self.stack.push(Type::String(
                    list.iter_mut()
                        .map(|x| x.get_string())
                        .collect::<Vec<String>>()
                        .join(&key),
                ))
            }

            // Judge is it find in string
            "find" => {
                let word = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::Bool(text.contains(&word)))
            }

            // Search by regular expression
            "regex" => {
                let pattern = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();

                let pattern: Regex = match Regex::new(pattern.as_str()) {
                    Ok(i) => i,
                    Err(e) => {
                        self.log_print(format!("Error! {}\n", e.to_string().replace("Error", "")));
                        self.stack.push(Type::Error("regex".to_string()));
                        return;
                    }
                };

                let mut list: Vec<Type> = Vec::new();
                for i in pattern.captures_iter(text.as_str()) {
                    list.push(Type::String(i[0].to_string()))
                }
                self.stack.push(Type::List(list));
            }

            // Commands of I/O

            // Write string in the file
            "write-file" => {
                let mut file = match File::create(Path::new(&self.pop_stack().get_string())) {
                    Ok(file) => file,
                    Err(e) => {
                        self.log_print(format!("Error! {e}\n"));
                        self.stack.push(Type::Error("create-file".to_string()));
                        return;
                    }
                };
                if let Err(e) = file.write_all(self.pop_stack().get_string().as_bytes()) {
                    self.log_print(format!("Error! {}\n", e));
                    self.stack.push(Type::Error("write-file".to_string()));
                }
            }

            // Read string in the file
            "read-file" => {
                let name = Path::new(&self.pop_stack().get_string()).to_owned();
                match get_file_contents(&name) {
                    Ok(s) => self.stack.push(Type::String(s)),
                    Err(e) => {
                        self.log_print(format!("Error! {}\n", e));
                        self.stack.push(Type::Error("read-file".to_string()));
                    }
                };
            }

            // Standard input
            "input" => {
                let prompt = self.pop_stack().get_string();
                self.stack.push(Type::String(input(prompt.as_str())));
            }

            // Standard output
            "print" => {
                let a = self.pop_stack().get_string();

                let a = a.replace("\\n", "\n");
                let a = a.replace("\\t", "\t");
                let a = a.replace("\\r", "\r");

                if let Mode::Debug = self.mode {
                    println!("[Output]: {a}");
                } else {
                    print!("{a}");
                }
            }

            // Standard output with new line
            "println" => {
                let a = self.pop_stack().get_string();

                let a = a.replace("\\n", "\n");
                let a = a.replace("\\t", "\t");
                let a = a.replace("\\r", "\r");

                if let Mode::Debug = self.mode {
                    println!("[Output]: {a}");
                } else {
                    println!("{a}");
                }
            }

            // Get command-line arguments
            "args-cmd" => self.stack.push(Type::List(
                env::args()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|x| Type::String(x.to_string()))
                    .collect::<Vec<Type>>(),
            )),

            // Commands of control

            // Evaluate string as program
            "eval" => {
                let code = self.pop_stack().get_string();
                self.evaluate_program(code)
            }

            // Conditional branch
            "if" => {
                let condition = self.pop_stack().get_bool(); // Condition
                let code_else = self.pop_stack().get_string(); // Code of else
                let code_if = self.pop_stack().get_string(); // Code of If
                if condition {
                    self.evaluate_program(code_if)
                } else {
                    self.evaluate_program(code_else)
                };
            }

            // Loop while condition is true
            "while" => {
                let cond = self.pop_stack().get_string();
                let code = self.pop_stack().get_string();
                while {
                    self.evaluate_program(cond.clone());
                    self.pop_stack().get_bool()
                } {
                    self.evaluate_program(code.clone());
                }
            }

            // Generate a thread
            "thread" => {
                let code = self.pop_stack().get_string();
                let mut executor = self.clone();
                thread::spawn(move || executor.evaluate_program(code));
            }

            // Exit a process
            "exit" => {
                let status = self.pop_stack().get_number().to_f64();
                std::process::exit(status as i32);
            }

            // Commands of list processing

            // Get list value by index
            "get" => {
                let index = self.pop_stack().get_number().to_f64() as usize;
                let list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    self.stack.push(list[index].clone());
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Set list value by index
            "set" => {
                let value = self.pop_stack();
                let index = self.pop_stack().get_number().to_f64() as usize;
                let mut list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    list[index] = value;
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Delete list value by index
            "del" => {
                let index = self.pop_stack().get_number().to_f64() as usize;
                let mut list = self.pop_stack().get_list();
                if list.len() > index {
                    list.remove(index);
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Append value in the list
            "append" => {
                let data = self.pop_stack();
                let mut list = self.pop_stack().get_list();
                list.push(data);
                self.stack.push(Type::List(list));
            }

            // Insert value in the list
            "insert" => {
                let data = self.pop_stack();
                let index = self.pop_stack().get_number().to_f64();
                let mut list = self.pop_stack().get_list();
                list.insert(index as usize, data);
                self.stack.push(Type::List(list));
            }

            // Get index of the list
            "index" => {
                let target = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                for (index, item) in list.iter().enumerate() {
                    if target == item.clone().get_string() {
                        self.stack.push(Type::Number(Fraction::new(index as f64)));
                        return;
                    }
                }
                self.log_print(String::from("Error! item not found in the list\n"));
                self.stack.push(Type::Error(String::from("item-not-found")));
            }

            // Sorting in the list
            "sort" => {
                let mut list: Vec<String> = self
                    .pop_stack()
                    .get_list()
                    .iter()
                    .map(|x| x.to_owned().get_string())
                    .collect();
                list.sort();
                self.stack.push(Type::List(
                    list.iter()
                        .map(|x| Type::String(x.to_string()))
                        .collect::<Vec<_>>(),
                ));
            }

            // reverse in the list
            "reverse" => {
                let mut list = self.pop_stack().get_list();
                list.reverse();
                self.stack.push(Type::List(list));
            }

            // Iteration for the list
            "for" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                list.iter().for_each(|x| {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());
                    self.evaluate_program(code.clone());
                });
            }

            // Generate a range
            "range" => {
                let step = self.pop_stack().get_number();
                let max = self.pop_stack().get_number();
                let min = self.pop_stack().get_number();

                let mut range: Vec<Type> = Vec::new();
                let mut i = min;

                while i.to_f64() < max.to_f64() {
                    range.push(Type::Number(i));
                    i = step + i;
                }

                self.stack.push(Type::List(range));
            }

            // Get length of list
            "len" => {
                let data = self.pop_stack().get_list();
                self.stack
                    .push(Type::Number(Fraction::new(data.len() as f64)));
            }

            // Commands of functional programming

            // Mapping a list
            "map" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                let mut result_list = Vec::new();
                for x in list.iter() {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    result_list.push(self.pop_stack());
                }

                self.stack.push(Type::List(result_list));
            }

            // Filtering a list value
            "filter" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                let mut result_list = Vec::new();

                for x in list.iter() {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    if self.pop_stack().get_bool() {
                        result_list.push(x.clone());
                    }
                }

                self.stack.push(Type::List(result_list));
            }

            // Generate value from list
            "reduce" => {
                let code = self.pop_stack().get_string();
                let now = self.pop_stack().get_string();
                let init = self.pop_stack();
                let acc = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                self.memory
                    .entry(acc.clone())
                    .and_modify(|value| *value = init.clone())
                    .or_insert(init);

                for x in list.iter() {
                    self.memory
                        .entry(now.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    let result = self.pop_stack();

                    self.memory
                        .entry(acc.clone())
                        .and_modify(|value| *value = result.clone())
                        .or_insert(result);
                }

                let result = self.memory.get(&acc);
                self.stack
                    .push(result.unwrap_or(&Type::String("".to_string())).clone());

                self.memory
                    .entry(acc.clone())
                    .and_modify(|value| *value = Type::String("".to_string()))
                    .or_insert(Type::String("".to_string()));
            }

            // Commands of memory manage

            // Pop in the stack
            "pop" => {
                self.pop_stack();
            }

            // Get size of stack
            "size-stack" => {
                let len: f64 = self.stack.len() as f64;
                self.stack.push(Type::Number(Fraction::new(len)));
            }

            // Get Stack as List
            "get-stack" => {
                self.stack.push(Type::List(self.stack.clone()));
            }

            // Define variable at memory
            "var" => {
                let name = self.pop_stack().get_string();
                let data = self.pop_stack();
                self.memory
                    .entry(name)
                    .and_modify(|value| *value = data.clone())
                    .or_insert(data);
                self.show_variables()
            }

            // Get data type of value
            "type" => {
                let result = match self.pop_stack() {
                    Type::Number(_) => "number".to_string(),
                    Type::String(_) => "string".to_string(),
                    Type::Bool(_) => "bool".to_string(),
                    Type::List(_) => "list".to_string(),
                    Type::Error(_) => "error".to_string(),
                    Type::Matrix(_, _) => "matrix".to_string(),
                };

                self.stack.push(Type::String(result));
            }

            // Explicit data type casting
            "cast" => {
                let types = self.pop_stack().get_string();
                let mut value = self.pop_stack();
                match types.as_str() {
                    "number" => self.stack.push(Type::Number(value.get_number())),
                    "string" => self.stack.push(Type::String(value.get_string())),
                    "bool" => self.stack.push(Type::Bool(value.get_bool())),
                    "list" => self.stack.push(Type::List(value.get_list())),
                    "error" => self.stack.push(Type::Error(value.get_string())),
                    _ => self.stack.push(value),
                }
            }

            // Get memory information
            "mem" => {
                let mut list: Vec<Type> = Vec::new();
                for (name, _) in self.memory.clone() {
                    list.push(Type::String(name))
                }
                self.stack.push(Type::List(list))
            }

            // Free up memory space of variable
            "free" => {
                let name = self.pop_stack().get_string();
                self.memory.remove(name.as_str());
                self.show_variables();
            }

            // Copy stack's top value
            "copy" => {
                let data = self.pop_stack();
                self.stack.push(data.clone());
                self.stack.push(data);
            }

            // Swap stack's top 2 value
            "swap" => {
                let b = self.pop_stack();
                let a = self.pop_stack();
                self.stack.push(b);
                self.stack.push(a);
            }

            // Commands of times

            // Get now time as unix epoch
            "now-time" => {
                self.stack.push(Type::Number(Fraction::new(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                )));
            }

            // Sleep fixed time
            "sleep" => sleep(Duration::from_secs_f64(
                self.pop_stack().get_number().to_f64(),
            )),

            // Commands of matrix
            "scalar-mul" => {
                let number = self.pop_stack().get_number().to_f64();

                let (matrix, (rows, cols)) = self.pop_stack().get_matrix();

                let matrix = nalgebra::DMatrix::from_row_slice(
                    rows,
                    cols,
                    &matrix.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );
                let result: Vec<f64> = (matrix * number).iter().cloned().collect();

                self.stack.push(Type::Matrix(
                    result.iter().map(|x| Fraction::new(*x)).collect(),
                    (rows, cols),
                ))
            }

            "add-matrix" => {
                let (matrix1, (rows1, cols1)) = self.pop_stack().get_matrix();
                let matrix1 = nalgebra::DMatrix::from_row_slice(
                    rows1,
                    cols1,
                    &matrix1.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let (matrix2, (rows2, cols2)) = self.pop_stack().get_matrix();
                let matrix2 = nalgebra::DMatrix::from_row_slice(
                    rows2,
                    cols2,
                    &matrix2.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let result: Vec<f64> = (matrix1 + matrix2).iter().cloned().collect();

                self.stack.push(Type::Matrix(
                    result.iter().map(|x| Fraction::new(*x)).collect(),
                    (rows1, cols1),
                ))
            }

            "sub-matrix" => {
                let (matrix1, (rows1, cols1)) = self.pop_stack().get_matrix();
                let matrix1 = nalgebra::DMatrix::from_row_slice(
                    rows1,
                    cols1,
                    &matrix1.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let (matrix2, (rows2, cols2)) = self.pop_stack().get_matrix();
                let matrix2 = nalgebra::DMatrix::from_row_slice(
                    rows2,
                    cols2,
                    &matrix2.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let result: Vec<f64> = (matrix2 - matrix1).iter().cloned().collect();

                self.stack.push(Type::Matrix(
                    result.iter().map(|x| Fraction::new(*x)).collect(),
                    (rows1, cols1),
                ))
            }

            "mul-matrix" => {
                let (matrix1, (rows1, cols1)) = self.pop_stack().get_matrix();
                let matrix1 = nalgebra::DMatrix::from_row_slice(
                    rows1,
                    cols1,
                    &matrix1.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let (matrix2, (rows2, cols2)) = self.pop_stack().get_matrix();
                let matrix2 = nalgebra::DMatrix::from_row_slice(
                    rows2,
                    cols2,
                    &matrix2.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let result: Vec<f64> = (matrix1 * matrix2).iter().cloned().collect();

                self.stack.push(Type::Matrix(
                    result.iter().map(|x| Fraction::new(*x)).collect(),
                    (rows1, cols1),
                ))
            }

            "transpose" => {
                let (matrix, (rows, cols)) = self.pop_stack().get_matrix();
                let matrix = nalgebra::DMatrix::from_row_slice(
                    rows,
                    cols,
                    &matrix.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );
                let transposed_matrix = matrix.transpose();

                let mut transposed_data = Vec::new();
                for i in 0..transposed_matrix.nrows() {
                    for j in 0..transposed_matrix.ncols() {
                        transposed_data.push(Fraction::new(transposed_matrix[(i, j)]));
                    }
                }

                self.stack.push(Type::Matrix(transposed_data, (cols, rows)))
            }

            "inverse" => {
                let (matrix, (rows, cols)) = self.pop_stack().get_matrix();
                let matrix = nalgebra::DMatrix::from_row_slice(
                    rows,
                    cols,
                    &matrix.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );
                let inversed_matrix = if let Some(i) = matrix.try_inverse() {
                    i
                } else {
                    self.stack.push(Type::Error("no-inverse".to_string()));
                    return;
                };

                self.stack.push(Type::Matrix(
                    inversed_matrix.iter().map(|x| Fraction::new(*x)).collect(),
                    (inversed_matrix.nrows(), inversed_matrix.ncols()),
                ))
            }

            "sim-equation" => {
                let (matrix, (rows, cols)) = self.pop_stack().get_matrix();
                let constants: Vec<Fraction> = {
                    let temp_constants = self.pop_stack().get_list();
                    let constants: Vec<Fraction> = temp_constants
                        .iter()
                        .map(|x| x.to_owned().get_number())
                        .collect();
                    constants
                };

                // Rounding error
                let decimal_places = {
                    let mut array = [matrix.clone(), constants.to_vec()].concat();
                    let mut temp: usize = 0;
                    for i in &mut array {
                        let i = &mut i
                            .display()
                            .split(".")
                            .collect::<Vec<&str>>()
                            .get(1)
                            .unwrap_or(&"")
                            .chars()
                            .count();
                        if temp < *i {
                            temp = *i
                        }
                    }
                    temp
                };

                let coefficients = nalgebra::DMatrix::from_row_slice(
                    rows,
                    cols,
                    &matrix.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );
                let constants = nalgebra::DVector::from_row_slice(
                    &constants.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let solution = if let Some(i) = coefficients.lu().solve(&constants) {
                    i
                } else {
                    self.stack.push(Type::Error("no-solution".to_string()));
                    return;
                };

                self.stack.push(Type::Matrix(
                    solution
                        .as_slice()
                        .iter()
                        .map(|value| {
                            let factor = 10_f64.powi(decimal_places as i32);
                            Fraction::new((value * factor).round() / factor)
                        })
                        .collect(),
                    (solution.nrows(), solution.ncols()),
                ));
            }

            "graph" => {
                let (data, (row, col)) = self.pop_stack().get_matrix();
                let adjacency_matrix = nalgebra::DMatrix::<f64>::from_row_slice(
                    row,
                    col,
                    &data.iter().map(|x| x.to_f64()).collect::<Vec<f64>>(),
                );

                let mut graph = Graph::<f64, ()>::new();
                let mut node_indices = Vec::new();

                for value in &data.clone() {
                    let node_index = graph.add_node(value.to_f64());
                    node_indices.push(node_index);
                }
                for (i, row) in adjacency_matrix.row_iter().enumerate() {
                    for (j, &value) in row.iter().enumerate() {
                        if value != 0.0 {
                            graph.add_edge(node_indices[i], node_indices[j], ());
                        }
                    }
                }
                let dot = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
                self.stack.push(Type::String(dot))
            }

            "bar-chart" => {
                let data: Vec<f64> = self
                    .pop_stack()
                    .get_list()
                    .iter_mut()
                    .map(|x| x.get_number().to_f64())
                    .collect();
                let mut figure = Figure::new();
                figure.axes2d().boxes(1..=data.len(), data, &[]);
                figure.set_title("Bar Chart - NumStack");
                figure.show().unwrap();
            }

            "line-chart" => {
                let data: Vec<f64> = self
                    .pop_stack()
                    .get_list()
                    .iter_mut()
                    .map(|x| x.get_number().to_f64())
                    .collect();
                let mut figure = Figure::new();
                figure.axes2d().lines(1..=data.len(), data, &[]);
                figure.set_title("Line Chart - NumStack");
                figure.show().unwrap();
            }

            // If it is not recognized as a command, use it as a string.
            _ => self.stack.push(Type::String(command)),
        }
    }

    /// Pop stack's top value
    fn pop_stack(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            self.log_print(
                "Error! There are not enough values on the stack. returns default value\n"
                    .to_string(),
            );
            Type::String("".to_string())
        }
    }
}
