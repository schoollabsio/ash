mod gpt_client;
mod settings;

use std::io::{self, Write};
use serde::Deserialize;
use std::process::Command;

use gpt_client::GptClient;
use settings::Settings;

#[derive(Deserialize)]
struct GptResponse {
    r#type: String,
    response: String,
}

async fn run(settings: Settings, model: GptClient, context: &Vec<String>, input: String) -> Result<GptResponse, String> {
    if input == "exit" {
        println!("Exiting...");
        return Err("exit".to_string());
    }

    let r = match model.send(&format!("{}\n{}", context.join("\n"), input)).await {
        Ok(response) => response,
        Err(e) => return Err(format!("Error sending request: {}", e)),
    };

    // println!("Response: {}", r);
    
    match serde_json::from_str(&r) {
        Ok(deserialized) => Ok(deserialized),
        Err(e) => Err(format!("Error deserializing response: {}", e)),
    }
}
fn main() {
    let settings = settings::Settings::new();
    let model = gpt_client::GptClient::new(
        settings.openai_api_key.clone(),
        "gpt-4-turbo".to_string(),
        "You are an intelligent command prompt. You will receive english-language instructions from the user, and then turn those instructions into executable shell commands or scripts. You are running on macOS. You will also summarize and interpret command output for the user. Return all of your responses in JSON format, with the following structure: { type: \"command\" | \"response\", response: string }. type indicates whether your response is an executable set of instructions, or human-readable text output addressing the user's most recent prompt.".to_string()
    );

    let mut conversation_history = Vec::new();

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        loop {
            // Prompt the user for input
            print!("ash: ");
            io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input

            // Read and trim the input
            let input = read_and_trim_input();

            // Add user input to conversation history
            &conversation_history.push(format!("User: {}", input));

            // Run the command and get the result
            match run(settings.clone(), model.clone(), &conversation_history, input).await {
                Ok(result) => {
                    // Add AI response to conversation history
                    conversation_history.push(format!("AI: {}", result.response));

                    if result.r#type != "command" {
                        println!("gpt> {}", result.response);
                    } else {
                        println!("run? (y/i/n)> {}", result.response);
                        let user_input = read_and_trim_input();
                        if user_input.to_lowercase() == "y" {
                            let output = execute_command(&result.response);
                            println!("{}", String::from_utf8_lossy(&output.stdout));
                            if !output.stderr.is_empty() {
                                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                            }
                        } else if user_input.to_lowercase() == "i" {
                            let output = execute_command(&result.response);
                            let command_output = String::from_utf8_lossy(&output.stdout).to_string();
                            if !output.stderr.is_empty() {
                                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                            }
                            match run(settings.clone(), model.clone(), &conversation_history, command_output).await {
                                Ok(new_result) => {
                                    println!("{}", new_result.response);
                                    // Add AI interpretation to conversation history
                                    conversation_history.push(format!("AI Interpretation: {}", new_result.response));
                                },
                                Err(e) => {
                                    eprintln!("Error interpreting result: {}", e);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
        }
    });
}

fn read_and_trim_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn execute_command(command: &str) -> std::process::Output {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command")
}