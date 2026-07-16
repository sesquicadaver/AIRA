fn main() {
    let msg = std::env::args().nth(1).unwrap_or_else(|| {
        String::from_utf8_lossy(aira_object::LOCAL_TEST_DOMAIN_MSG).into_owned()
    });
    println!(
        "{}",
        aira_object::local_test_signature(msg.as_bytes()).signature_value
    );
}
