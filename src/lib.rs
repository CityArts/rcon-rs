// =======================================================================
//  Copyleft City:Arts Project 2018-∞.
//  Distributed under the terms of the 3-Clause BSD License.
//  (See accompanying file LICENSE or copy at
//   https://opensource.org/licenses/BSD-3-Clause)
// =======================================================================

#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate byteorder;

//* Constants *//
/// Declarations
const NULL: char = '\x00'; // C NULL
/*const MIN_PACKET_SIZE: u8 = 10; // 4(id) + 4(type) + 1(body) + 1(empty string)
const MIN_PACKET_ACTUAL_SIZE: u8 = 14; // 4(size) + 4(id) + 4(type) + 1(body) + 1(empty string)
const MAX_PACKET_SIZE: usize = 4096;
const MAX_BODY_SIZE: usize = 4086; // (4096 - 4(id) - 4(type) - 1(empty string) -1(body null-terminator)) = 4086*/
/// RCON command types
//const SERVERDATA_RESPONSE_VALUE: i32 = 0;
const SERVERDATA_EXECOMMAND: i32 = 2;
//const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_AUTH: i32 = 3;

//* Use from external library *//
use std::mem;
use std::str;
use std::io::prelude::*;
use std::net::TcpStream;
use rand::prelude::*;
use byteorder::{WriteBytesExt, ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Client {
    pub host: String,
    pub port: String,
    pub password: String,
    pub stream: TcpStream
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Payload {
    pub request_id: i32,
    pub ty: i32,
    pub body: Vec<u8>
}

impl Client {
    pub fn new(host: String, port: String, password: String) -> Result<Self, ()> {
        let (host_tmp, port_tmp) = { (host.clone(), port.clone()) };
        if let Ok(s) = TcpStream::connect(&format!("{}:{}", host_tmp, port_tmp)) {
            Ok(Self {
                host: host,
                port: port,
                password: password,
                stream: s
            })
        } else {
            Err(unsafe { mem::zeroed() })
        }
    }

    pub fn send_command(&mut self, command: &str) -> String {
        let payload = Payload::new(SERVERDATA_EXECOMMAND, command.as_bytes().to_vec());
        let response = self.send_payload(payload);
        let mut body = response.body;
        let _ = body.retain(|&x| x != NULL as u8);

        String::from_utf8(body).unwrap()
    }

    pub fn send_auth(&mut self) -> Result<(), ()> {
        let payload = Payload::new(SERVERDATA_AUTH, self.password.as_bytes().to_vec());
        let _ = self.send_payload(payload);
        Ok(())
    }

    pub fn send_payload(&mut self, payload: Payload) -> Payload {
        let packet = payload.create_packet();
        let mut buffer: [u8; 4096] = [0; 4096];
        self.stream.write(packet.as_slice()).unwrap();
        self.stream.read(&mut buffer).unwrap();

        let packet_size = buffer[0];
        let request_id = LittleEndian::read_i32(&buffer[1..7]);
        let ty = buffer[8] as i32;
        let body = buffer[9..(packet_size as usize)].to_vec();

        Payload {
            request_id: request_id,
            ty: ty,
            body: body
        }
    }
}

impl Payload {
    pub fn new(ty: i32, body: Vec<u8>) -> Self {
        Self {
            request_id: random(),
            ty: ty,
            body: body
        }
    }
    
    pub fn packet_size(&self) -> i32 {
        self.body.len() as i32 + 10
    }
  
    pub fn create_packet(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        res.write_i32::<LittleEndian>(self.packet_size()).unwrap();
        res.write_i32::<LittleEndian>(self.request_id).unwrap();
        res.write_i32::<LittleEndian>(self.ty).unwrap();
        res.extend_from_slice(&self.body);
        res.push(0);
        res.push(0);
        res
    }
}
