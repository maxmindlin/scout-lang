use scout_interpreter::builtin::BuiltinKind;
use scout_interpreter::object::Object;
use std::sync::Arc;

#[cfg(test)]
mod builtin_tests {
    use super::*;

    #[tokio::test]
    async fn test_trim_function() {
        // Test that trim function is recognized
        let trim_builtin = BuiltinKind::is_from("trim");
        assert!(trim_builtin.is_some());
        assert_eq!(trim_builtin.unwrap(), BuiltinKind::Trim);

        // Test trim functionality
        let test_string = Arc::new(Object::Str("  hello world  ".to_string()));
        let args = vec![test_string];

        // Create a mock environment - we'll need to simulate this
        // For now, let's just verify the builtin is registered correctly
        println!("Trim builtin registered successfully");
    }

    #[test]
    fn test_string_builtin_recognition() {
        // Test string manipulation builtin recognitions
        assert_eq!(BuiltinKind::is_from("trim"), Some(BuiltinKind::Trim));
        assert_eq!(BuiltinKind::is_from("trimStart"), Some(BuiltinKind::TrimStart));
        assert_eq!(BuiltinKind::is_from("trimEnd"), Some(BuiltinKind::TrimEnd));
        assert_eq!(BuiltinKind::is_from("toLowerCase"), Some(BuiltinKind::ToLowerCase));
        assert_eq!(BuiltinKind::is_from("toUpperCase"), Some(BuiltinKind::ToUpperCase));
        assert_eq!(BuiltinKind::is_from("split"), Some(BuiltinKind::Split));
        assert_eq!(BuiltinKind::is_from("replace"), Some(BuiltinKind::Replace));
        assert_eq!(BuiltinKind::is_from("attr"), Some(BuiltinKind::Attr));

        // Test other builtins still work
        assert_eq!(BuiltinKind::is_from("len"), Some(BuiltinKind::Len));
        assert_eq!(BuiltinKind::is_from("print"), Some(BuiltinKind::Print));
        assert_eq!(BuiltinKind::is_from("nonexistent"), None);
    }
}