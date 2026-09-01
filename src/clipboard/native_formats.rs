use crate::model::Representation;

pub(super) fn for_macos(representations: &[Representation]) -> Vec<Representation> {
    let mut normalized: Vec<(Representation, u8)> = Vec::new();
    for representation in representations {
        let Some((format, priority)) = macos_format(&representation.format) else {
            continue;
        };
        let candidate = Representation {
            item: representation.item,
            format: format.to_owned(),
            data: representation.data.clone(),
        };
        if let Some((existing, existing_priority)) = normalized
            .iter_mut()
            .find(|(existing, _)| existing.item == candidate.item && existing.format == candidate.format)
        {
            if priority > *existing_priority {
                *existing = candidate;
                *existing_priority = priority;
            }
        } else {
            normalized.push((candidate, priority));
        }
    }
    normalized
        .into_iter()
        .map(|(representation, _)| representation)
        .collect()
}

fn macos_format(format: &str) -> Option<(&str, u8)> {
    match format {
        "public.utf8-plain-text" => Some(("public.utf8-plain-text", 100)),
        "public.plain-text" => Some(("public.utf8-plain-text", 90)),
        "NSStringPboardType" => Some(("public.utf8-plain-text", 80)),
        "text/plain;charset=utf-8" => Some(("public.utf8-plain-text", 70)),
        "UTF8_STRING" => Some(("public.utf8-plain-text", 60)),
        "text/plain" => Some(("public.utf8-plain-text", 50)),
        "TEXT" => Some(("public.utf8-plain-text", 40)),
        "STRING" => Some(("public.utf8-plain-text", 30)),
        "public.html" => Some(("public.html", 100)),
        "text/html" => Some(("public.html", 80)),
        "public.rtf" => Some(("public.rtf", 100)),
        "text/rtf" => Some(("public.rtf", 80)),
        "public.png" => Some(("public.png", 100)),
        "image/png" => Some(("public.png", 80)),
        "public.jpeg" => Some(("public.jpeg", 100)),
        "image/jpeg" | "image/jpg" => Some(("public.jpeg", 80)),
        "public.tiff" => Some(("public.tiff", 100)),
        "image/tiff" => Some(("public.tiff", 80)),
        "com.compuserve.gif" => Some(("com.compuserve.gif", 100)),
        "image/gif" => Some(("com.compuserve.gif", 80)),
        "public.heic" => Some(("public.heic", 100)),
        "image/heic" => Some(("public.heic", 80)),
        "public.heif" => Some(("public.heif", 100)),
        "image/heif" => Some(("public.heif", 80)),
        "org.webmproject.webp" => Some(("org.webmproject.webp", 100)),
        "image/webp" => Some(("org.webmproject.webp", 80)),
        "com.adobe.pdf" => Some(("com.adobe.pdf", 100)),
        "application/pdf" => Some(("com.adobe.pdf", 80)),
        _ if is_native_type(format) => Some((format, 100)),
        _ => None,
    }
}

fn is_native_type(format: &str) -> bool {
    format.contains('.')
        && !format.contains('/')
        && format
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn representation(format: &str, data: &[u8]) -> Representation {
        Representation {
            item: 0,
            format: format.into(),
            data: data.to_vec(),
        }
    }

    #[test]
    fn translates_x11_text_to_a_macos_pasteboard_type() {
        let normalized = for_macos(&[
            representation("STRING", b"fallback"),
            representation("UTF8_STRING", b"utf8"),
            representation("text/plain;charset=utf-8", b"portable"),
        ]);

        assert_eq!(
            normalized,
            vec![representation("public.utf8-plain-text", b"portable")]
        );
    }

    #[test]
    fn translates_portable_media_types_and_preserves_native_types() {
        let normalized = for_macos(&[
            representation("text/html", b"<b>hello</b>"),
            representation("image/png", b"png"),
            representation("com.example.custom-type", b"custom"),
        ]);

        assert_eq!(
            normalized,
            vec![
                representation("public.html", b"<b>hello</b>"),
                representation("public.png", b"png"),
                representation("com.example.custom-type", b"custom"),
            ]
        );
    }

    #[test]
    fn drops_x11_atoms_and_unmapped_mime_types() {
        let normalized = for_macos(&[
            representation("chromium/x-source-url", b"url"),
            representation("SAVE_TARGETS", b"targets"),
        ]);

        assert!(normalized.is_empty());
    }
}
