use crate::error::Error;

use openssl::rsa::{Padding};
use openssl::pkey::{PKeyRef, Public, Private};

pub trait Sponge {
    fn absorb<T: AsRef<[u8]>>(&mut self, t: T) -> Result<(), Error>;
}

pub struct Verifier<'a> { v: openssl::sign::Verifier<'a> }

impl<'a> Verifier<'a> {
    pub fn new(pk: &'a PKeyRef<Public>) -> Result<Verifier<'a>, Error> {
        let mut v = openssl::sign::Verifier::new_without_digest(pk)?;
        v.set_rsa_padding(Padding::PKCS1_PSS)?;
        Ok(Verifier { v })
    }

    pub fn verify<S: AsRef<[u8]>>(self, sig: S) -> Result<bool, Error> {
        Ok(self.v.verify(sig.as_ref())?)
    }
}

impl Sponge for Verifier<'_> {
    fn absorb<T: AsRef<[u8]>>(&mut self, t: T) -> Result<(), Error> {
        self.v.update(t.as_ref()).map_err(Error::from)
    }
}

pub struct Signer<'a> { s: openssl::sign::Signer<'a> }

impl<'a> Signer<'a> {
    pub fn new(pk: &'a PKeyRef<Private>) -> Result<Self, Error> {
        let mut s = openssl::sign::Signer::new_without_digest(pk)?;
        s.set_rsa_padding(Padding::PKCS1_PSS)?;
        Ok(Signer { s })
    }

    pub fn sign(self) -> Result<Vec<u8>, Error> {
        self.s.sign_to_vec().map_err(Error::from)
    }
}

impl Sponge for Signer<'_> {
    fn absorb<T: AsRef<[u8]>>(&mut self, t: T) -> Result<(), Error> {
        self.s.update(t.as_ref()).map_err(Error::from)
    }
}

pub trait Absorbable {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error>;
}
