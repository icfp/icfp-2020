use std::env;
use std::process;

use hyper::StatusCode;

use icfp::ast::{demodulate_string, modulate_to_string, Symbol};
use icfp::client::Client as AlienClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);
    let client = AlienClient::new(server_url, player_key);

    let mut program = Symbol::List(vec![Symbol::Lit(0)]);

    for _i in 0..50 {
        dbg!(&program);
        let response = dbg!(send_program(&client, &program).await);
        program = icfp::stack_interpreter::eval_instructions(&[
            Symbol::Ap,
            Symbol::Inc,
            Symbol::Ap,
            Symbol::Car,
            response,
        ])
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
