use serialport;
use std::io;
use std::io::Write;
use std::thread;
use std::time;
use std::str;
use colored::Colorize;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

fn write_handler(port: &mut Box<dyn serialport::SerialPort>, tx: Sender<i32>) {
    loop {
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Could not read line.");

        if command.contains("exit") {
            let _ = tx.send(1);
            break;
        }

        if command == "\r\n" { continue };

        port.write(command.as_bytes()).expect("Could not send...");
    }
}

fn read_handler(port: &mut Box<dyn serialport::SerialPort>, rx: Receiver<i32>) {
    loop {
        let mut _result: Vec<u8> = vec![0; 10000];

        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => { break; }
            Err(TryRecvError::Empty) => {}
        }

        match port.read(_result.as_mut_slice()) {
            Ok(t) => {
                let string = match str::from_utf8(&_result[..t]) {
                    Ok(v) => v,
                    Err(_) => "ERROR - conversion\r\n",
                };

                let split = string.split("ERROR");
                let length = string.split("ERROR").count();

                for (i, s) in split.enumerate() {
                    print!("{}", s.green().bold());

                    if i == (length - 1) { break; }
                    print!("{}", &"ERROR".red().bold());
                }

                io::stdout().flush().expect("Couldn't flush stdout");
            }
            Err(_) => continue,
        }
    }
}

fn main() {
    loop {
        let ports = serialport::available_ports().unwrap();
        println!("Choose a port to connect to:");

        let mut counter = 1;
        for port in &ports {
            println!("{}: {}", counter, port.port_name);
            counter += 1;
        }
        println!("0: To exit...\n");

        let ans = loop {
            let mut ans = String::new();
            io::stdin().read_line(&mut ans).expect("Failed to read.");
            let ans: usize = match ans.trim().parse() {
                Ok(num) => num,
                Err(_) => continue,
            };
            break ans;
        };
        if ans == 0 {
            return;
        }
        let ans = ans - 1;

        let mut port = serialport::new(&ports[ans].port_name, 115200).timeout(time::Duration::from_millis(10)).open()
            .expect("Could not open serial port.");
        let mut clone = port.try_clone().expect("Could not create a clone.");

        println!("Connected to {:?}\nWrite your command below:", &port.name());

        let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();
        thread::spawn(move || { read_handler(&mut clone, rx); });
        write_handler(&mut port, tx);
    }
}
