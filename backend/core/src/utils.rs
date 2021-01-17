use jsonwebtoken::{DecodingKey, EncodingKey, errors::Error as JWTErrors, Header, Validation};
pub use jsonwebtoken::errors::ErrorKind as JWTErrorKind;
use ring::hmac;
use serde::{de::DeserializeOwned, Serialize};
pub use jsonwebtoken::Algorithm;

#[derive(Clone)]
pub struct Hash {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey<'static>,
    hmac256_key: hmac::Key,
    hmac512_key: hmac::Key,
    crypto_algorithm: Algorithm,
    header: Header,
    validation: Validation,
    secret: String,
}

impl Hash {
    pub fn new(secret: &'static String, crypto_algorithm: Algorithm) -> Hash {
        Hash {
            encoding_key: EncodingKey::from_secret(secret.as_str().as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_str().as_bytes()),
            hmac256_key: hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes()),
            hmac512_key: hmac::Key::new(hmac::HMAC_SHA512, secret.as_bytes()),
            crypto_algorithm,
            header: Header::new(crypto_algorithm),
            validation: Validation::new(crypto_algorithm),
            secret: secret.clone(),
        }
    }

    pub fn sign512(&self, message: &str) -> String {
        base64::encode(hmac::sign(&self.hmac512_key, message.as_bytes()))
    }

    pub fn sign256(&self, message: &str) -> String {
        base64::encode(hmac::sign(&self.hmac256_key, message.as_bytes()))
    }

    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String, JWTErrors> {
        jsonwebtoken::encode(&self.header, claims, &self.encoding_key)
    }

    pub fn decode<T: DeserializeOwned>(&self, token: &str) -> Result<T, JWTErrors> {
        jsonwebtoken::decode::<T>(token, &self.decoding_key, &self.validation)
            .map(|t| t.claims)
    }
}