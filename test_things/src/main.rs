use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn handle_read(mut stream: TcpStream, tx: mpsc::Sender<Vec<u8>>) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let message = buffer[..size].to_vec();
                println!("Mesaj primit: {:?}", String::from_utf8_lossy(&message));
                // Trimitem mesajul către thread-ul de scriere
                if tx.send(message).is_err() {
                    eprintln!("Canalul de scriere a fost închis.");
                    break;
                }
            }
            Ok(_) => {
                println!("Clientul s-a deconectat.");
                break;
            }
            Err(e) => {
                eprintln!("Eroare la citire: {}", e);
                break;
            }
        }
    }
}

fn handle_write(mut stream: TcpStream, rx: mpsc::Receiver<Vec<u8>>) {
    loop {
        match rx.recv() {
            Ok(message) => {
                // Trimitem mesajul înapoi clientului
                if let Err(e) = stream.write_all(&message) {
                    eprintln!("Eroare la scriere: {}", e);
                    break;
                }
                println!("Mesaj trimis: {:?}", String::from_utf8_lossy(&message));
            }
            Err(_) => {
                eprintln!("Canalul de citire a fost închis.");
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server pornit la adresa 127.0.0.1:7878");

    // Acceptăm o singură conexiune
    if let Ok((stream, addr)) = listener.accept() {
        println!("Client conectat: {}", addr);

        // Canal pentru comunicare între thread-uri
        let (tx, rx) = mpsc::channel();

        // Clonăm stream-ul pentru thread-urile de citire și scriere
        let read_stream = stream.try_clone()?;
        let write_stream = stream.try_clone()?;

        // Pornim thread-ul pentru citire
        thread::spawn(move || handle_read(read_stream, tx));

        // Pornim thread-ul pentru scriere
        thread::spawn(move || handle_write(write_stream, rx));
    }

    loop{

    }

    Ok(())
}
