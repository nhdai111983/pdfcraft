//! PDF/A output-intent policy stays here, and so do the ICC v2 bytes: the shared craft-cms
//! generator names the upstream product in the profile description (see the tests below).
//! Keep v2.1 rather than substituting the shared v4 built-in profile.

fn s15f16(v: f64) -> [u8; 4] {
    ((v * 65536.0).round() as i32).to_be_bytes()
}

fn xyz(x: f64, y: f64, z: f64) -> Vec<u8> {
    let mut t = b"XYZ \0\0\0\0".to_vec();
    t.extend(s15f16(x));
    t.extend(s15f16(y));
    t.extend(s15f16(z));
    t
}

fn desc(text: &str) -> Vec<u8> {
    let mut t = b"desc\0\0\0\0".to_vec();
    let ascii: Vec<u8> = text.bytes().chain(std::iter::once(0)).collect();
    t.extend((ascii.len() as u32).to_be_bytes());
    t.extend(&ascii);
    // No Unicode or ScriptCode descriptions.
    t.extend([0u8; 4 + 4]);
    t.extend([0u8; 2 + 1 + 67]);
    t
}

fn text(text: &str) -> Vec<u8> {
    let mut t = b"text\0\0\0\0".to_vec();
    t.extend(text.bytes());
    t.push(0);
    t
}

fn curve() -> Vec<u8> {
    const N: usize = 1024;
    let mut t = b"curv\0\0\0\0".to_vec();
    t.extend((N as u32).to_be_bytes());
    for i in 0..N {
        let v = i as f64 / (N - 1) as f64;
        let lin = if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        t.extend(((lin * 65535.0).round() as u16).to_be_bytes());
    }
    t
}

pub fn srgb() -> Vec<u8> {
    // NexusOffice: the shared generator `craft_cms_v2::builtin::srgb_v2()` writes the upstream
    // product's name into the profile description and has no option for the name, so the profile
    // Nexus PDF puts in a PDF/A output intent is built here, naming Nexus PDF (spec §4.4). These
    // are the bytes the fork has always written; the tests below pin the name and the shape.
    // sRGB primaries, Bradford-adapted to the D50 profile connection space.
    let tags: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"desc", desc("sRGB (Nexus PDF)")),
        (b"cprt", text("No copyright, use freely")),
        (b"wtpt", xyz(0.9642, 1.0, 0.8249)),
        (b"rXYZ", xyz(0.4361, 0.2225, 0.0139)),
        (b"gXYZ", xyz(0.3851, 0.7169, 0.0971)),
        (b"bXYZ", xyz(0.1431, 0.0606, 0.7141)),
        (b"rTRC", curve()),
    ];
    // gTRC and bTRC share rTRC's data.
    let count = tags.len() + 2;
    let table_len = 4 + count * 12;
    let mut offset = 128 + table_len;
    let mut table = (count as u32).to_be_bytes().to_vec();
    let mut data = Vec::new();
    let mut trc = (0, 0);
    for (sig, body) in &tags {
        let len = body.len();
        table.extend(*sig);
        table.extend((offset as u32).to_be_bytes());
        table.extend((len as u32).to_be_bytes());
        if *sig == b"rTRC" {
            trc = (offset, len);
        }
        data.extend(body);
        // Tags start on 4-byte boundaries.
        let pad = (4 - len % 4) % 4;
        data.extend(std::iter::repeat_n(0u8, pad));
        offset += len + pad;
    }
    for sig in [b"gTRC", b"bTRC"] {
        table.extend(sig);
        table.extend((trc.0 as u32).to_be_bytes());
        table.extend((trc.1 as u32).to_be_bytes());
    }
    let size = 128 + table.len() + data.len();
    let mut h = Vec::with_capacity(size);
    h.extend((size as u32).to_be_bytes());
    h.extend(b"\0\0\0\0"); // preferred CMM
    h.extend([2, 0x10, 0, 0]); // version 2.1
    h.extend(b"mntr");
    h.extend(b"RGB ");
    h.extend(b"XYZ ");
    h.extend([0u8; 12]); // date
    h.extend(b"acsp");
    h.extend(b"\0\0\0\0"); // platform
    h.extend([0u8; 4]); // flags
    h.extend([0u8; 4]); // manufacturer
    h.extend([0u8; 4]); // model
    h.extend([0u8; 8]); // attributes
    h.extend([0, 0, 0, 0]); // perceptual intent
    h.extend(s15f16(0.9642));
    h.extend(s15f16(1.0));
    h.extend(s15f16(0.8249));
    h.extend([0u8; 4]); // creator
    h.resize(128, 0);
    h.extend(table);
    h.extend(data);
    h
}

#[cfg(test)]
mod tests {
    /// The profile the PDF/A output intent carries is the one the fork has always written: an ICC
    /// v2.1 matrix/TRC profile whose tag table is self-consistent.
    #[test]
    fn profile_is_well_formed() {
        let p = super::srgb();
        assert_eq!(u32::from_be_bytes(p[0..4].try_into().unwrap()) as usize, p.len());
        assert_eq!(&p[36..40], b"acsp");
        assert_eq!(&p[12..20], b"mntrRGB ");
        let n = u32::from_be_bytes(p[128..132].try_into().unwrap()) as usize;
        assert_eq!(n, 9);
        for i in 0..n {
            let e = 132 + i * 12;
            let off = u32::from_be_bytes(p[e + 4..e + 8].try_into().unwrap()) as usize;
            let len = u32::from_be_bytes(p[e + 8..e + 12].try_into().unwrap()) as usize;
            assert!(off.is_multiple_of(4) && off + len <= p.len(), "tag {i}");
        }
    }

    /// NexusOffice: a PDF/A file Nexus PDF writes carries this profile, so whoever opens the file
    /// reads its description. The shared `craft_cms_v2::builtin::srgb_v2()` hard-codes the upstream
    /// product's name there and takes no name of its own, which is why `srgb()` builds the bytes
    /// here; the shared call would put that name in our documents.
    #[test]
    fn the_output_intent_names_nexus_pdf() {
        let p = super::srgb();
        assert!(p.windows(16).any(|w| w == b"sRGB (Nexus PDF)"), "the desc tag names Nexus PDF");
        assert_ne!(p, craft_cms_v2::builtin::srgb_v2(), "not the shared profile, which carries the other name");
    }
}
