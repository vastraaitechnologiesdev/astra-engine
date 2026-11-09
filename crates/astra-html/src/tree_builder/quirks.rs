//! HTML5 DOCTYPE quirks mode determination.

use crate::token::DoctypeToken;
use astra_dom::QuirksMode;

/// Determines the [`QuirksMode`] from a parsed DOCTYPE token.
#[must_use]
pub fn determine_quirks_mode(doctype: Option<&DoctypeToken>) -> QuirksMode {
    let Some(doc) = doctype else {
        return QuirksMode::Quirks;
    };

    if doc.force_quirks {
        return QuirksMode::Quirks;
    }

    if !doc.name.eq_ignore_ascii_case("html") {
        return QuirksMode::Quirks;
    }

    let public_id = doc.public_id.as_deref().unwrap_or("").to_ascii_lowercase();
    let system_id = doc.system_id.as_deref().unwrap_or("").to_ascii_lowercase();

    // Check for quirks public IDs
    let quirks_prefixes = [
        "+//silmaril//dtd html pro v0r11 19970101//",
        "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
        "-//as//dtd html 3.0 aswedit + extensions//",
        "-//ietf//dtd html 2.0 level 1//",
        "-//ietf//dtd html 2.0 level 2//",
        "-//ietf//dtd html 2.0 strict level 1//",
        "-//ietf//dtd html 2.0 strict level 2//",
        "-//ietf//dtd html 2.0 strict//",
        "-//ietf//dtd html 2.0//",
        "-//ietf//dtd html 3.0//",
        "-//ietf//dtd html 3.2 final//",
        "-//ietf//dtd html 3.2//",
        "-//ietf//dtd html 3//",
        "-//ietf//dtd html level 0//",
        "-//ietf//dtd html level 1//",
        "-//ietf//dtd html level 2//",
        "-//ietf//dtd html level 3//",
        "-//ietf//dtd html strict level 0//",
        "-//ietf//dtd html strict level 1//",
        "-//ietf//dtd html strict level 2//",
        "-//ietf//dtd html strict level 3//",
        "-//ietf//dtd html strict//",
        "-//ietf//dtd html//",
        "-//metrius//dtd metrius presentational//",
        "-//microsoft//dtd internet explorer 2.0 html strict//",
        "-//microsoft//dtd internet explorer 2.0 html//",
        "-//microsoft//dtd internet explorer 2.0 tables//",
        "-//microsoft//dtd internet explorer 3.0 html strict//",
        "-//microsoft//dtd internet explorer 3.0 html//",
        "-//microsoft//dtd internet explorer 3.0 tables//",
        "-//netscape comm. corp.//dtd html//",
        "-//netscape comm. corp.//dtd strict html//",
        "-//o'reilly and associates//dtd html 2.0//",
        "-//o'reilly and associates//dtd html extended 1.0//",
        "-//o'reilly and associates//dtd html extended relax 1.0//",
        "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
        "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
        "-//spyglass//dtd html 2.0 extended//",
        "-//sq//dtd html 2.0 hotmetal + extensions//",
        "-//sun microsystems corp.//dtd hotjava html//",
        "-//sun microsystems corp.//dtd hotjava strict html//",
        "-//w3c//dtd html 3 1995-03-24//",
        "-//w3c//dtd html 3.2 draft//",
        "-//w3c//dtd html 3.2 final//",
        "-//w3c//dtd html 3.2//",
        "-//w3c//dtd html 3.2s draft//",
        "-//w3c//dtd html 4.0 frameset//",
        "-//w3c//dtd html 4.0 transitional//",
        "-//w3c//dtd html experimental 19960712//",
        "-//w3c//dtd html experimental 970421//",
        "-//w3c//dtd w3 html//",
        "-//w3o//dtd w3 html 3.0//",
        "-/w3c/dtd html 4.0 transitional/en",
        "html",
    ];

    for prefix in quirks_prefixes {
        if public_id.starts_with(prefix) {
            return QuirksMode::Quirks;
        }
    }

    if system_id == "http://www.ibm.com/data/dtd/v11/ibmxhtml.dtd" {
        return QuirksMode::Quirks;
    }

    // HTML 4.01 Transitional without system identifier triggers quirks mode
    if doc.system_id.is_none() && public_id.starts_with("-//w3c//dtd html 4.01 transitional//") {
        return QuirksMode::Quirks;
    }

    // Limited quirks mode
    if public_id.starts_with("-//w3c//dtd xhtml 1.0 frameset//")
        || public_id.starts_with("-//w3c//dtd xhtml 1.0 transitional//")
        || (doc.system_id.is_some()
            && (public_id.starts_with("-//w3c//dtd html 4.01 frameset//")
                || public_id.starts_with("-//w3c//dtd html 4.01 transitional//")))
    {
        return QuirksMode::LimitedQuirks;
    }

    QuirksMode::NoQuirks
}
