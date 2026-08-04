#[allow(
    clippy::indexing_slicing,
    clippy::min_ident_chars,
    clippy::unwrap_used,
    reason = "tests"
)]
#[cfg(test)]
mod integration_tests {
    use interpreter::run;

    #[test]
    fn call_lambda() {
        let lox_script = "print fun () { return 1; }();";
        let mut output = String::new();

        let result = run(lox_script, &mut output).unwrap();

        assert_eq!(output, "1\n");
        assert!(result.is_none());
    }
}
