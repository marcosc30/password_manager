extern crate crypto;
extern crate hex;
extern crate rand;

use sha2::{Sha256, Digest};
use crypto::aes::KeySize::KeySize256;
use crypto::aes::cbc_encryptor;
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use rand::Rng;
use std::str;
use crypto::aes::cbc_decryptor;

pub fn hash_master(password: &str, salt: [u8; 32]) -> [u8; 32] {
    // If I was making this on the cloud, or to make an improvement, I'd add a salt to the password
    let converted_password = password.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(converted_password);
    hasher.update(&salt);
    let result = hasher.finalize();
    let mut hashed_password = [0; 32];
    hashed_password.copy_from_slice(&result[..]);
    hashed_password
}

// 2. Encrypt a given password using the master password

pub fn encrypt_password(password: &str, key: &[u8; 32]) -> Vec<u8> {
    // Generate a random IV
    let iv: [u8; 16] = rand::thread_rng().gen();

    // Create an encryptor instance
    let mut encryptor = cbc_encryptor(KeySize256, key, &iv, PkcsPadding);

    // Buffer setup
    let mut read_buffer = RefReadBuffer::new(password.as_bytes());
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    // Perform encryption
    let mut encrypted_data = Vec::new();
    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true).unwrap();
        encrypted_data.extend(write_buffer.take_read_buffer().take_remaining().iter().copied());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    // Combine IV and ciphertext
    [iv.to_vec(), encrypted_data].concat()
}

// Decrypt the password

pub fn decrypt_password(encrypted_data: &[u8], key: &[u8]) -> String {
    let (iv, ciphertext) = encrypted_data.split_at(16);

    let mut decryptor = cbc_decryptor(KeySize256, key, iv, PkcsPadding);

    let mut read_buffer = RefReadBuffer::new(ciphertext);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    let mut decrypted_data = Vec::new();
    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true).unwrap();
        decrypted_data.extend(write_buffer.take_read_buffer().take_remaining().iter().copied());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    String::from_utf8(decrypted_data).unwrap()
}