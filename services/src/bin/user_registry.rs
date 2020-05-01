use rand::{RngCore as _, SeedableRng as _};
use sha2::Digest as _;

use oasis_std::{
    abi::*,
    collections::map::{Entry, Map},
    Address, Context,
};

/// A user registry and sign-in service.
#[derive(oasis_std::Service)]
struct UserRegistry {
    users: Map<String /* name */, User>,

    /// The key used to sign JWTs that can be used to authenticate users.
    jwt_secret: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    name: String,
    login_credential: LoginCredential,
}

/// User login credential. For now, password because it's easy.
/// A public key would be ideal, but storing the private key is a bit tricky–especially
/// for casual users.
type LoginCredential = String;

impl UserRegistry {
    pub fn new(_ctx: &Context) -> Self {
        let mut jwt_secret = vec![0u8; 32];
        rand_pcg::Pcg64::from_entropy().fill_bytes(&mut jwt_secret);
        Self {
            users: Map::new(),
            jwt_secret,
        }
    }

    /// Registers a new user.
    pub fn register(
        &mut self,
        _ctx: &Context,
        name: String,
        login_credential: LoginCredential,
    ) -> Result<(), Error> {
        match self.users.entry(name.clone()) {
            Entry::Occupied(_) => Err(Error::UsernameTaken),
            Entry::Vacant(ve) => {
                ve.insert(User {
                    name,
                    login_credential,
                });
                Ok(())
            }
        }
    }

    /// Returns an auth token that can be used to authenticate a user.
    /// Probably don't share this unless you want people to be able to use services as you.
    ///
    /// `audience` is the address of the service that will eventually consume this token.
    pub fn sign_in(
        &self,
        _ctx: &Context,
        name: String,
        login_credential: LoginCredential,
        audience: Address,
    ) -> Result<String, Error> {
        match self.users.get(&name) {
            Some(u) if u.login_credential == login_credential => {
                let token = jwt::Token::new(
                    jwt::Header::default(),
                    jwt::Registered {
                        sub: Some(u.name.to_string()),
                        aud: Some(format!("{:x}", audience)),
                        ..Default::default()
                    },
                );
                Ok(token.signed(&self.jwt_secret, sha2::Sha256::new()).unwrap())
            }
            _ => Err(Error::PermissionDenied),
        }
    }

    /// Checks whether the provided token is valid. Returns the verified claims.
    pub fn verify_token(&self, ctx: &Context, token: String) -> Result<UserInfo, Error> {
        let token = jwt::Token::<jwt::Header, jwt::Registered>::parse(&token)
            .map_err(|_| Error::InvalidToken)?;

        let (sub, aud) = match &token.claims {
            jwt::Registered {
                sub: Some(sub),
                aud: Some(aud),
                ..
            } => (sub, aud),
            _ => return Err(Error::InvalidToken),
        };

        if *aud != format!("{:x}", ctx.sender())
            || !token.verify(&self.jwt_secret, sha2::Sha256::new())
        {
            return Err(Error::PermissionDenied);
        }

        Ok(UserInfo { name: sub.clone() })
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Error {
    PermissionDenied,
    UsernameTaken,
    InvalidToken,
}

fn main() {
    oasis_std::service!(UserRegistry);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a new account and a `Context` with the new account as the sender.
    fn create_account_ctx() -> (Address, Context) {
        let addr = oasis_test::create_account(0 /* initial balance */);
        let ctx = Context::default().with_sender(addr).with_gas(100_000);
        (addr, ctx)
    }

    #[test]
    fn test_register_signin() {
        let (aud_addr, ctx) = create_account_ctx();

        let mut registry = UserRegistry::new(&ctx);

        let (user_a, pw_a) = ("A", "passwordA");
        let (user_b, pw_b) = ("B", "passwordB");

        registry
            .register(&ctx, user_a.to_string(), pw_a.to_string())
            .unwrap();

        // can't create with existing username
        assert_eq!(
            registry.register(&ctx, user_a.to_string(), pw_b.to_string()),
            Err(Error::UsernameTaken)
        );

        // ensure sign-ins fail when without credentials
        let sign_in_bad = |user: &str, pass: &str| {
            registry.sign_in(&ctx, user.to_string(), pass.to_string(), aud_addr)
        };
        assert_eq!(sign_in_bad(user_a, pw_b), Err(Error::PermissionDenied));
        assert_eq!(sign_in_bad(user_b, pw_a), Err(Error::PermissionDenied));

        // test results of successful sign-ins
        let sign_in = |aud_addr: Address| {
            registry
                .sign_in(&ctx, user_a.to_string(), pw_a.to_string(), aud_addr)
                .unwrap()
        };
        let good_token = sign_in(aud_addr);
        let bad_aud_token = sign_in(oasis_test::create_account(0));
        let bad_sig_token = good_token[..good_token.len() - 1].to_string();

        assert_eq!(
            registry.verify_token(&ctx, good_token),
            Ok(Claims {
                sub: user_a.to_string(),
                aud: format!("{:x}", aud_addr),
            })
        );
        assert_eq!(
            registry.verify_token(&ctx, bad_aud_token),
            Err(Error::PermissionDenied)
        );
        assert_eq!(
            registry.verify_token(&ctx, bad_sig_token),
            Err(Error::PermissionDenied)
        );
    }
}
