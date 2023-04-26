#![feature(test)]

extern crate test;

mod hitori_examples {
    include!("../../hitori-examples/src/putting_everything_together/email.rs");
    include!("../../hitori-examples/src/putting_everything_together/ipv4.rs");
    include!("../../hitori-examples/src/putting_everything_together/uri.rs");
}

use hitori::Expr;
use hitori_examples::{Email as HitoriEmail, IpV4 as HitoriIpV4, Uri as HitoriUri};
use regex::Regex;
use test::Bencher;

const TEXT: &str = include_str!("regex-benchmark/input-text.txt");

const REGEX_EMAIL_PATTERN: &str = r"[\w\.+-]+@[\w\.-]+\.[\w\.-]+";

const REGEX_URI_PATTERN: &str = r"[\w]+://[^/\s?#][^\s?#]+(?:\?[^\s#]*)?(?:#[^\s]*)?";

const REGEX_IPV4_PATTERN: &str =
    r"(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])";

fn hitori_find_count<E: Expr<usize, char>>(expr: &E) -> usize {
    hitori::string::FindIter::new(expr, TEXT).count()
}

fn regex_find_count(re: &Regex) -> usize {
    re.find_iter(TEXT).count()
}

#[test]
fn hitori_email_count_eq_regex_email_count() {
    assert_eq!(
        hitori_find_count(&HitoriEmail),
        regex_find_count(&Regex::new(REGEX_EMAIL_PATTERN).unwrap())
    );
}

#[test]
fn hitori_uri_count_eq_regex_uri_count() {
    assert_eq!(
        hitori_find_count(&HitoriUri),
        regex_find_count(&Regex::new(REGEX_URI_PATTERN).unwrap())
    );
}

#[test]
fn hitori_ipv4_count_eq_regex_ipv4_count() {
    assert_eq!(
        hitori_find_count(&HitoriIpV4),
        regex_find_count(&Regex::new(REGEX_IPV4_PATTERN).unwrap())
    );
}

#[bench]
fn hitori_email(b: &mut Bencher) {
    b.iter(|| hitori_find_count(&HitoriEmail));
}

#[bench]
fn regex_email(b: &mut Bencher) {
    b.iter(|| regex_find_count(&Regex::new(REGEX_EMAIL_PATTERN).unwrap()));
}

#[bench]
fn regex_email_precompiled(b: &mut Bencher) {
    let re = Regex::new(REGEX_EMAIL_PATTERN).unwrap();
    b.iter(|| regex_find_count(&re));
}

#[bench]
fn hitori_uri(b: &mut Bencher) {
    b.iter(|| hitori_find_count(&HitoriUri));
}

#[bench]
fn regex_uri(b: &mut Bencher) {
    b.iter(|| regex_find_count(&Regex::new(REGEX_URI_PATTERN).unwrap()));
}

#[bench]
fn regex_uri_precompiled(b: &mut Bencher) {
    let re = Regex::new(REGEX_URI_PATTERN).unwrap();
    b.iter(|| regex_find_count(&re));
}

#[bench]
fn hitori_ipv4(b: &mut Bencher) {
    b.iter(|| hitori_find_count(&HitoriIpV4));
}

#[bench]
fn regex_ipv4(b: &mut Bencher) {
    b.iter(|| regex_find_count(&Regex::new(REGEX_IPV4_PATTERN).unwrap()));
}

#[bench]
fn regex_ipv4_precompiled(b: &mut Bencher) {
    let re = Regex::new(REGEX_IPV4_PATTERN).unwrap();
    b.iter(|| regex_find_count(&re));
}
