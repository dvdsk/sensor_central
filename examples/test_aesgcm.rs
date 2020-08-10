use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::Aes128Gcm;
use rand::Rng;

fn main() {
    let key = [0u8; 16];
    const NONCE_SIZE: usize = 12;
    const MAC_SIZE: usize = 6;

    let mut rng = rand::thread_rng();

    let cipher = Aes128Gcm::new(GenericArray::from_slice(&key));

    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce[..]);
    let nonce = GenericArray::from_slice(&nonce);

    //let pin: u32 = 28;
    let pin: u32 = rng.gen_range(0, 999999);
    let mut pin_array = [0u8; 4];
    pin_array[..4].copy_from_slice(&pin.to_be_bytes());
    dbg!(pin_array);

    let ciphertext = cipher
        .encrypt(nonce, pin_array.as_ref())
        .expect("encryption failure!"); // NOTE: handle this error to avoid panics!
    dbg!(&ciphertext);
    dbg!(&ciphertext.len());

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failure!"); // NOTE: handle this error to avoid panics!
    dbg!(&plaintext);
}
