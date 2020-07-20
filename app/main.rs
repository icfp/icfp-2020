use std::env;
use std::process;

use hyper::StatusCode;

use icfp::ast::{demodulate_string, modulate_to_string, Symbol};
use icfp::client::Client as AlienClient;
use icfp::stack_interpreter::{Effects, Resolve, VM};
use image::{GrayImage, ImageFormat};
use std::ops::Deref;
use std::time::SystemTime;

struct CliEffects {}

impl Effects for CliEffects {
    fn send(&self, content: String) -> String {
        unimplemented!()
    }

    fn display(&self, image: &GrayImage) {
        let name = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        image
            .save_with_format(format!("/tmp/{}.png", name.as_secs()), ImageFormat::Png)
            .unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);
    let client = AlienClient::new(server_url, player_key);

    let mut program = Symbol::List(vec![Symbol::Lit(0)]);

    let vm = VM::new_effects(Box::new(CliEffects {}));

    for _i in 0..50 {
        dbg!(&program);
        let response = dbg!(send_program(&client, &program).await);
        program = vm
            .run_symbols(&[Symbol::Ap, Symbol::Inc, Symbol::Ap, Symbol::Car, response])
            .deref()
            .clone();
    }

    Ok(())
}

async fn send_program(client: &AlienClient, program: &Symbol) -> Symbol {
    let program_string = modulate_to_string(&program);

    match client.send(program_string).await {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                print!("Server response: ");
                let text = res.text().await;
                match text {
                    Ok(content) => demodulate_string(content.as_str()),
                    Err(why) => panic!("error reading body: {:?}", why),
                }
            }
            _ => {
                println!("Unexpected server response:");
                println!("HTTP code: {}", res.status());
                print!("Response body: ");

                let text = res.text().await;

                match text {
                    Ok(content) => println!("{:?}", content),
                    Err(why) => println!("error reading body: {:?}", why),
                }

                process::exit(2)
            }
        },
        Err(err) => {
            println!("Unexpected server response:\n{}", err);
            process::exit(1)
        }
    }
}
