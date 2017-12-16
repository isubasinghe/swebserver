extern crate threadpool;

#[cfg(any(windows))]
extern crate winapi;
#[cfg(any(windows))]
extern crate user32;
#[cfg(any(windows))]
extern crate kernel32;

#[cfg(any(windows))]
use user32::MessageBoxA;
#[cfg(any(windows))]
use winapi::winuser::{MB_OK, MB_ICONINFORMATION};


use std::ffi::CString;
use std::ptr;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;

use threadpool::ThreadPool;

#[cfg(any(windows))]
fn hide_console_window() {
    let window = unsafe {kernel32::GetConsoleWindow()};
    if window != ptr::null_mut() {
        unsafe {
            user32::ShowWindow(window, winapi::SW_HIDE);
        }
    }
}

#[cfg(any(windows))]
fn show_window() {
    let text = CString::new("Running server at port 8080").unwrap();
    let caption = CString::new("Running").unwrap();

    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONINFORMATION
        );
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut response = "HTTP/1.1 200 OK\r\n\r\n".to_owned();

    {
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer);
        buffer[1023] = 0;

        let request = String::from_utf8_lossy(&buffer[..]);
        let sarray = request.split(" ");
        let vec: Vec<&str> = sarray.collect();

        let mut err: bool = false;
        let mut path = vec[1].to_string();
        if path != "/" {
            path = format!(".{}", path);
        }else {
            path = "./index.html".to_string();
        }
        
        match File::open(path) {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        response = format!("{}{}", response,contents);
                    },
                    Err(_) => {
                        err = true;
                    },
                }
            },
            Err(_) => {
                err = true;
            },
        }

        if err {
            response = format!("{}<html><p>404</p></html>", response);
        }

        
        
    }
    

    stream.write(response.as_bytes()).unwrap();
    
    stream.flush().unwrap();
}

fn spawn_thread(pool: Box<ThreadPool>, stream: TcpStream) -> Box<ThreadPool> {
    pool.execute(move || {
        handle_connection(stream);
    });
    return pool;
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut pool = Box::new(ThreadPool::new(4));

    #[cfg(any(windows))]
    hide_console_window();
    #[cfg(any(windows))]
    show_window();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool = spawn_thread(pool, stream),
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn main() {
    start_server();
    
}