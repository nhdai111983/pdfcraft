//! NexusOffice (QC-PDF-04): comment dates in the reader's local time. A PDF date carries its zone
//! (`D:20261026063900Z`, `…+07'00'`); the comment list showed its digits as they were, so a note
//! made at 13:39 in Hanoi and written in UTC read "06:39". A date without a zone is local already
//! and stays as written.

/// Days since 1970-01-01 of a proleptic Gregorian date (H. Hinnant's `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The date of a day count since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn digits(s: &str, at: usize, len: usize) -> Option<i64> {
    let part = s.get(at..at.checked_add(len)?)?;
    part.bytes().all(|b| b.is_ascii_digit()).then(|| part.parse().ok()).flatten()
}

/// A PDF date as "yyyy-mm-dd HH:MM" in the zone `offset` seconds east of UTC. Dates without a
/// zone, and anything that isn't a PDF date, read as [`pdfcraft_render::pretty_date`] does.
pub fn local_pretty_date(s: &str, offset: i64) -> String {
    let d = s.trim().trim_start_matches("D:");
    let (Some(y), Some(mo), Some(day), Some(h), Some(mi)) = (digits(d, 0, 4), digits(d, 4, 2), digits(d, 6, 2), digits(d, 8, 2), digits(d, 10, 2))
    else {
        return pdfcraft_render::pretty_date(s);
    };
    let sec = digits(d, 12, 2).unwrap_or(0);
    // The zone follows the seconds (or the minutes when the seconds are left out).
    let rest = d.get(14..).filter(|_| digits(d, 12, 2).is_some()).or_else(|| d.get(12..)).unwrap_or("");
    let zone = match rest.chars().next() {
        Some('Z') => Some(0),
        Some(sign @ ('+' | '-')) => {
            let zh = digits(rest, 1, 2).unwrap_or(0);
            let zm = rest.get(3..).map(|r| r.trim_start_matches('\'')).and_then(|r| digits(r, 0, 2)).unwrap_or(0);
            let z = zh.saturating_mul(3600).saturating_add(zm.saturating_mul(60));
            Some(if sign == '-' { -z } else { z })
        }
        _ => None,
    };
    let valid = (1..=12).contains(&mo) && (1..=31).contains(&day) && h < 24 && mi < 60 && sec < 61;
    let (Some(zone), true) = (zone, valid) else { return pdfcraft_render::pretty_date(s) };
    let offset = offset.clamp(-14 * 3600, 14 * 3600);
    let secs = days_from_civil(y, mo, day) * 86_400 + h * 3600 + mi * 60 - zone + offset;
    let (y, mo, day) = civil_from_days(secs.div_euclid(86_400));
    let t = secs.rem_euclid(86_400);
    format!("{y:04}-{mo:02}-{day:02} {:02}:{:02}", t / 3600, t % 3600 / 60)
}

/// [`local_pretty_date`] in this machine's zone.
pub fn local_date(s: &str) -> String {
    local_pretty_date(s, crate::local_utc_offset())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_dates_read_in_local_time() {
        let hanoi = 7 * 3600;
        assert_eq!(local_pretty_date("D:20261026063900Z", hanoi), "2026-10-26 13:39");
        assert_eq!(local_pretty_date("D:20261026063900+07'00'", hanoi), "2026-10-26 06:39");
        assert_eq!(local_pretty_date("D:20261026233000-02'30'", hanoi), "2026-10-27 09:00", "across midnight");
        assert_eq!(local_pretty_date("D:20261231200000Z", hanoi), "2027-01-01 03:00", "across the year");
        assert_eq!(local_pretty_date("D:202610260639Z", 0), "2026-10-26 06:39", "seconds left out");
        // Without a zone the digits are local already; junk reads as before.
        assert_eq!(local_pretty_date("D:20261026063900", hanoi), "2026-10-26 06:39");
        assert_eq!(local_pretty_date("yesterday", hanoi), "yesterday");
        assert_eq!(local_pretty_date("D:20261399063900Z", hanoi), "2026-13-99 06:39");
        assert_eq!(local_pretty_date("D:2026ĐĐ26063900Z", hanoi), "D:2026ĐĐ26063900Z");
        assert_eq!(local_pretty_date("D:99991231235959Z", i64::MAX), "10000-01-01 13:59");
    }
}
