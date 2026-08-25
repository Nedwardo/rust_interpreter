#[allow(
    clippy::indexing_slicing,
    clippy::min_ident_chars,
    clippy::unwrap_used,
    reason = "tests"
)]
#[cfg(test)]
mod integration_tests {
    use interpreter::expressions::Value;
    use interpreter::run;

    #[test]
    fn call_lambda() {
        let lox_script = "print fun () { return 1; }();";
        let mut output = String::new();

        let result = run(lox_script, &mut output);

        assert_eq!(output, "1\n");

        println!("{result:?}");

        assert!(result.unwrap().is_none());
    }

    #[test]
    fn fibonacci() {
        let lox_script = "fun fib(n) { \
            if (n <= 1) return n; \
            return fib(n - 2) + fib(n - 1); \
        } \
        \
        for (var i = 0; i < 20; i = i + 1) { \
            print fib(i); \
        }";

        let mut output = String::new();

        let result = run(lox_script, &mut output);

        println!("{result:?}");

        assert!(result.unwrap().is_none());
    }

    #[test]
    fn closures() {
        let lox_script = "fun makeCounter() { \
                var i = 0; \
                fun count() { \
                    i = i + 1; \
                    print i; \
                } \
            \
            return count; \
        } \
        \
        var counter = makeCounter(); \
        counter(); \
        counter(); \
        ";

        let mut output = String::new();

        let result = run(lox_script, &mut output);
        println!("Printing output !!!!: {:?}", output);

        assert_eq!(output, "1\n2\n");
        assert!(matches!(result.unwrap().unwrap(), Value::Nil));
    }

    #[test]
    fn function_test() {
        let lox_script = "fun sayHi(first, last) { \
            print \"Hi, \" + first + \" \" + last + \"!\"; \
        } \
        \
        sayHi(\"Dear\", \"Reader\");";
        let mut output = String::new();

        let result = run(lox_script, &mut output);

        assert_eq!(output, "\"Hi, Dear Reader!\"\n");
        assert!(matches!(result.unwrap().unwrap(), Value::Nil));
    }

    #[test]
    fn lambda() {
        let lox_script = "fun thrice(fn) { \
            for (var i = 1; i <= 3; i = i + 1) { \
                fn(i); \
            } \
        } \
        \
        thrice(fun (a) { \
            print a; \
        });";

        let mut output = String::new();

        let result = run(lox_script, &mut output);

        assert_eq!(output, "1\n2\n3\n");

        println!("{result:?}");

        assert!(matches!(result.unwrap().unwrap(), Value::Nil));
    }

    #[test]
    fn for_statment() {
        let lox_script = "var a = 0; \
        var temp; \
        \
        for (var b = 1; a < 20; b = temp + b) { \
            print a; \
            temp = a; \
            a = b; \
        }";

        let mut output = String::new();

        let result = run(lox_script, &mut output);

        assert_eq!(output, "0\n1\n1\n2\n3\n5\n8\n13\n");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn unary_and_binary() {
        let lox_script = "print -1 + 3";

        let mut output = String::new();

        let result = run(lox_script, &mut output);

        assert_eq!(output, "2\n");
        assert!(result.unwrap().is_none());
    }
}
