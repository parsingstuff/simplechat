#![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(unused_mut)] 

extern crate itertools;

use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read};
use std::process;
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use itertools::Itertools;

use std::io::prelude::*;
use std::fs::File;
use std::io::{BufReader};

fn read_streams(sarc : &Arc<Mutex<HashMap<usize,TcpStream>>>, barc : &Arc<Mutex<HashMap<usize, Vec<u8>>>>) { 

    let mut shash = sarc.lock().unwrap();
    let mut bhash = barc.lock().unwrap();
    let mut removals = Vec::<usize>::new();

    let keys : Vec<usize> = shash.keys().map (|k| k.clone()).collect();

    keys.iter().foreach(|i| { 
        let mut bytes : [u8; 16] = [0; 16];
        let mut s = shash.get(i).unwrap();
        match s.read(&mut bytes) { 
            Ok(_)   => { 
                if bytes[0] == 0 { 
                    removals.push(*i);
                } else { 
                    bhash.insert(*i, bytes.to_vec());
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // println!("Oops, I'd block, not reading");
            },
            Err(e)  => panic!("Error in read : {:?}", e),
        }
    });

    removals.iter().foreach(|i| { 
        shash.remove(&i);
    });
                        
}

fn write_streams(sarc : &Arc<Mutex<HashMap<usize,TcpStream>>>, barc : &Arc<Mutex<HashMap<usize, Vec<u8>>>>) { 

    let mut shash = sarc.lock().unwrap();
    let mut bhash = barc.lock().unwrap();

    let mut bkeys : Vec<usize> = bhash.keys().map(|k| k.clone()).collect();

    for i in &bkeys {     
        let mut skeys : Vec<usize> = shash.keys().filter(|x| **x != *i).map(|k| k.clone()).collect();
        let result = bhash.get(&i).unwrap();
        let result_str : String = String::from_utf8_lossy(&result).to_string();
        println!("{} got {}", i, result_str);
        for x in skeys { 
            let mut cs = shash.get(&x).unwrap();
            match cs.write(result) { 
                Err(e)  => panic!("{:?}", e),
                _       => {}
            }
        }
    }

    &bkeys.iter().foreach (|i| { 
        bhash.remove(&i);
    });

}

fn main() {

    let sarc = Arc::new(Mutex::new(HashMap::<usize,TcpStream>::new()));
    let barc = Arc::new(Mutex::new(HashMap::<usize,Vec<u8>>::new()));

    let s = TcpListener::bind("0.0.0.0:9200").unwrap();

    let nsarc = sarc.clone();
    let nbarc = barc.clone();

    thread::spawn(move || { 

        // println!("In polling thread");
        // println!("After lock");
        loop { 
            // println!("Loop top");
            thread::sleep(Duration::from_millis(100));
            read_streams(&nsarc, &nbarc);
            write_streams(&nsarc, &nbarc);
            // println!("In polling thread loop");
            // println!("Loop end");
        }

    });

    s.incoming().enumerate().foreach(|(i, result)| {
        let nsarc = sarc.clone();
        let mut cs = result.unwrap();
        cs.set_nonblocking(true).expect("set to nonblocking failed");
        println!("Connection from {}:{}", &cs.peer_addr().unwrap().ip(), &cs.peer_addr().unwrap().port());

        match &cs.write(b"Welcome to jchat\n") { 
            Err(e)  => panic!("{:?}", e),
            _       => {}
        }

        thread::spawn(move || { 
            // println!("In sock thread");
            let mut shash = nsarc.lock().unwrap();
            // println!("After sock thread lock");

            shash.insert(i, cs);
            // println!("shash : {:?}", &shash);
            thread::sleep(Duration::from_millis(100));
            // println!("Thread exit");
            
        });
    });
}
