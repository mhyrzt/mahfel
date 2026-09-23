use crate::ports::StoredFile;
use mahfel_domain::FileObject;
use sha2::{Digest, Sha256};

pub(crate) fn verify(file: &FileObject, stored: &StoredFile) -> Option<&'static str> {
    if stored.bytes.len() as u64 != file.size_bytes {
        return None;
    }
    let digest = Sha256::digest(&stored.bytes);
    if base64::Engine::encode(&base64::engine::general_purpose::STANDARD, digest)
        != file.checksum_sha256
    {
        return None;
    }
    let mime = match file.declared_mime.as_str() {
        "application/pdf" if valid_pdf(&stored.bytes) => "application/pdf",
        "image/png" if valid_png(&stored.bytes) => "image/png",
        "image/jpeg" if valid_jpeg(&stored.bytes) => "image/jpeg",
        _ => return None,
    };
    if stored.content_type.as_deref() != Some(mime) {
        return None;
    }
    Some(mime)
}

fn valid_pdf(bytes: &[u8]) -> bool {
    bytes.starts_with(b"%PDF-")
        && bytes
            .iter()
            .rposition(|byte| !byte.is_ascii_whitespace())
            .and_then(|end| bytes.get(..=end))
            .is_some_and(|body| body.ends_with(b"%%EOF"))
}

fn valid_png(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.ends_with(b"\0\0\0\0IEND\xaeB`\x82")
}

fn valid_jpeg(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\xff\xd8\xff") && bytes.ends_with(b"\xff\xd9")
}

#[cfg(test)]
mod tests {
    use super::{valid_jpeg, valid_pdf, valid_png, verify};
    use crate::ports::StoredFile;
    use mahfel_domain::{ActorId, FileContext, FileObject, RoomId};
    use time::OffsetDateTime;

    #[test]
    fn rejects_executable_and_appended_polyglot_markers() {
        assert!(!valid_pdf(b"MZ%PDF-1.7\n%%EOF"));
        assert!(!valid_pdf(b"%PDF-1.7\n%%EOF\nPK\x03\x04"));
        assert!(!valid_png(b"\x89PNG\r\n\x1a\nMZ"));
        assert!(!valid_jpeg(b"\xff\xd8\xff\xd9PK\x03\x04"));
    }

    #[test]
    fn wrong_checksum_and_size_never_verify() {
        let file = FileObject::pending(
            ActorId::new(),
            FileContext::Room(RoomId::new()),
            "deck.pdf".into(),
            "application/pdf".into(),
            14,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            OffsetDateTime::UNIX_EPOCH,
        )
        .expect("valid metadata");
        let stored = StoredFile {
            bytes: b"%PDF-1.7\n%%EOF".to_vec(),
            content_type: Some("application/pdf".into()),
        };
        assert!(verify(&file, &stored).is_none());
        let stored = StoredFile {
            bytes: b"%PDF-1.7\n%%EOFx".to_vec(),
            content_type: Some("application/pdf".into()),
        };
        assert!(verify(&file, &stored).is_none());
    }
}
