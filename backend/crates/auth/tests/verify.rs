//! Deterministic token-verification tests with a local issuer.
//!
//! The Keycloak integration test proves real interoperability; these tests
//! pin the edge cases (expiry, audience, issuer, tampering) with controlled
//! tokens signed by a fixed test key.

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use mahfel_auth::{TokenError, TokenVerifier, TokenVerifierConfig};
use serde_json::json;
use std::{
    error::Error,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::net::TcpListener;

const ISSUER_AUDIENCE: &str = "mahfel-api";
const KEY_ID: &str = "test-key";

/// PKCS#8 RSA private key used only by this test.
const TEST_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDbmVO19GITNOuj
VpKeftrusY3jZjR+z1kOVwNwkM0avBU82CEGyPozUaQ4/eOtjOB6QOeTzayew3KC
VAH6Oj4nivrrUrF6iATLWDFyXqjnDC3lbNV85bVRdhNeozNXYHYxxrhgExoxiJwK
1FOKFXLNsOg+Hu2/UHlqNu0Y47Of7S+7lCJ+xCbxvRF8o6JSVimaHdP1y4ESwBZF
qzyxBKogPBWpRNbBehLVCTbdrIVunot/ONEFiNuposcKjh0GDJB4Ng/4WSZenAGC
jCoO26y5N27NdARErCYj047qlUteCTUBm/bs8EdsqVqmx3ODdOhsqkqiHw8htSW+
hL9vS7FnAgMBAAECggEAU1Om9BA7dtZJ47zh/cpdAN3W3UbW/JdpT4us/nWznkVo
OaI5Rf3VEswT6JxOXHhsZXdjaC5PJh87wEBoT67yDscp10mVls0uJIremGta4/E3
KStXBwGS/8oyJ2L+BFXSA6lIJQyaj9kqfOF8oZTp7ysFtSy/Qi0jJS6aYStl6XS+
6KGrSLmArXIuzNZ3/byOunBWQWbfsxSNitRiFXDqsxfpA0cbsj4adb3gpQl919jB
aZ5Fi8VzYPqP7kOE0APuI85SmWxPhX1vXbbmwqCA/JWPzXnnxU0KR+k1Npsqbg6a
ITvO0VLYHY56fQi1tFY77JepT2XQwbDPUipy1T3foQKBgQD2hQrkZ/VkupH/6NZ6
Tw8n0IMQOLyxomEQ4Lj0Ugizj7KbGyEN8w+6CSU84y9VclR/trW7laa3FMr++bC8
GY7wkONZFsdsbtLkLSlU9LILPKdAOcbBQDJaOZip5ll07JU+TMTZQpE0KfHDDCjH
4RmMKzlslpBmaY3WWDR/rz8EpQKBgQDkC0CwVO+vp8JPjze3BRViJytU1G9cKcpd
O2t1W/KINYIQ0eqkBV5l7wLq1ht8JmMRpa3gLHHxdIoxouKzM3kSqvPGkoUTO2Cj
f5HPnCBJmk7paKD6Y4FBA4SsRqYlp+uKfmzLlleQ7hIkEK8H5ur8CTzsTV4mhX08
rKrex9MkGwKBgCETPp/gltsZ8SbNvaERZG4qGrudcSvAJpfWHtqg+wvkdsk6vZea
ifyi530g2iPzwASWphFyBblAr++/3oeOQlBikC5O0RjqUZ9Ns+LzJcQ+Mg568gDR
tS/H1ITPJCd86pU+RZcEyGigTBCoW/xdZkkZL44zMVKcTOx6TyjmgWUFAoGBAOHo
T9Ye3i7Z9sFP9HanX46TlPZ9WDAp6iHACm0BgFr0/vtgqJyCAQjj5LT1E9dCQ+Qo
h/5KqTn0JC6V2RSDSk528fXmElSB/vMZHj5XwRiI9+F71/UT++iG7rinaFJnbgwN
wANS1T5qyafWqz4dwO2azTKeVkW8+drQhpkZxY2/AoGBAO8wvaUA+yeaUof6/QFx
iwz7RIX6QlztIcV6rQBXmbsS4zHdQ1aCMCMctatqozztewJk0qa8dRRm8KNjNbLr
nCg+0ezO+qF2AI+7WAaaicScxpJkW7WUXFVW3wIdKyFStuAXzY7JIH6biTxgUU1W
4fZV7NStTEG5uoe0960K55+T
-----END PRIVATE KEY-----";

const MODULUS: &str = "25lTtfRiEzTro1aSnn7a7rGN42Y0fs9ZDlcDcJDNGrwVPNghBsj6M1GkOP3jrYzgekDnk82snsNyglQB-jo-J4r661KxeogEy1gxcl6o5wwt5WzVfOW1UXYTXqMzV2B2Mca4YBMaMYicCtRTihVyzbDoPh7tv1B5ajbtGOOzn-0vu5QifsQm8b0RfKOiUlYpmh3T9cuBEsAWRas8sQSqIDwVqUTWwXoS1Qk23ayFbp6LfzjRBYjbqaLHCo4dBgyQeDYP-FkmXpwBgowqDtusuTduzXQERKwmI9OO6pVLXgk1AZv27PBHbKlapsdzg3TobKpKoh8PIbUlvoS_b0uxZw";

struct Issuer {
    url: String,
    _task: tokio::task::JoinHandle<()>,
}

async fn start_issuer() -> Result<Issuer, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let url = format!("http://{address}");
    let keys = Arc::new(RwLock::new(jwks(KEY_ID)));

    let discovery_url = url.clone();
    let discovery = axum::Router::new().route(
        "/.well-known/openid-configuration",
        axum::routing::get(move || {
            let url = discovery_url.clone();
            async move {
                axum::Json(json!({
                    "issuer": url,
                    "jwks_uri": format!("{url}/jwks"),
                }))
            }
        }),
    );
    let jwks_route = {
        let keys = Arc::clone(&keys);
        axum::Router::new().route(
            "/jwks",
            axum::routing::get(move || {
                let keys = Arc::clone(&keys);
                async move { axum::Json(keys.read().expect("lock").clone()) }
            }),
        )
    };
    let app = discovery.merge(jwks_route);
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(Issuer { url, _task: task })
}

fn jwks(kid: &str) -> serde_json::Value {
    json!({
        "keys": [{
            "kty": "RSA",
            "kid": kid,
            "alg": "RS256",
            "use": "sig",
            "n": MODULUS,
            "e": "AQAB",
        }]
    })
}

fn mint(claims: serde_json::Value) -> Result<String, Box<dyn Error>> {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KEY_ID.to_owned());
    let key = EncodingKey::from_rsa_pem(TEST_KEY_PEM.as_bytes())?;
    Ok(jsonwebtoken::encode(&header, &claims, &key)?)
}

fn claims(issuer: &str, offset_seconds: i64) -> serde_json::Value {
    let now = time::OffsetDateTime::now_utc().unix_timestamp() + offset_seconds;
    json!({
        "iss": issuer,
        "aud": ISSUER_AUDIENCE,
        "sub": "user-1",
        "email": "user@mahfel.local",
        "email_verified": true,
        "preferred_username": "user",
        "name": "Test User",
        "iat": now,
        "nbf": now,
        "exp": now + 300,
    })
}

fn verifier(issuer: &Issuer) -> TokenVerifier {
    let mut config = TokenVerifierConfig::new(issuer.url.clone(), ISSUER_AUDIENCE);
    // Keep the leeway small so an expired token is immediately rejected.
    config.leeway = Duration::from_secs(1);
    TokenVerifier::new(config)
}

#[tokio::test]
async fn verifies_a_valid_token_and_normalizes_claims() -> Result<(), Box<dyn Error>> {
    let issuer = start_issuer().await?;
    let token = mint(claims(&issuer.url, 0))?;

    let identity = verifier(&issuer).verify(&token).await?;

    assert_eq!(identity.subject, "user-1");
    assert_eq!(identity.issuer, issuer.url);
    assert_eq!(identity.email.as_deref(), Some("user@mahfel.local"));
    assert!(identity.email_verified);
    Ok(())
}

#[tokio::test]
async fn rejects_an_expired_token() -> Result<(), Box<dyn Error>> {
    let issuer = start_issuer().await?;
    // Issued well beyond the leeway in the past.
    let token = mint(claims(&issuer.url, -600))?;

    assert!(matches!(
        verifier(&issuer).verify(&token).await,
        Err(TokenError::Expired)
    ));
    Ok(())
}

#[tokio::test]
async fn rejects_a_token_for_another_audience() -> Result<(), Box<dyn Error>> {
    let issuer = start_issuer().await?;
    let mut token_claims = claims(&issuer.url, 0);
    token_claims["aud"] = json!("another-api");
    let token = mint(token_claims)?;

    assert!(matches!(
        verifier(&issuer).verify(&token).await,
        Err(TokenError::WrongAudience)
    ));
    Ok(())
}

#[tokio::test]
async fn rejects_a_token_from_another_issuer() -> Result<(), Box<dyn Error>> {
    let issuer = start_issuer().await?;
    let token = mint(claims("https://evil.example/realms/rastai", 0))?;

    assert!(matches!(
        verifier(&issuer).verify(&token).await,
        Err(TokenError::WrongIssuer)
    ));
    Ok(())
}

#[tokio::test]
async fn rejects_a_tampered_signature() -> Result<(), Box<dyn Error>> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let issuer = start_issuer().await?;
    let token = mint(claims(&issuer.url, 0))?;

    // Flip a bit in the decoded signature so the encoding cannot mask it.
    let mut segments: Vec<&str> = token.split('.').collect();
    let mut signature = URL_SAFE_NO_PAD.decode(segments[2])?;
    signature[0] ^= 0x01;
    let reencoded = URL_SAFE_NO_PAD.encode(&signature);
    segments[2] = &reencoded;
    let tampered = segments.join(".");

    assert!(matches!(
        verifier(&issuer).verify(&tampered).await,
        Err(TokenError::InvalidSignature)
    ));
    Ok(())
}
