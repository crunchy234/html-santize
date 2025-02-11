#[cfg(test)]
mod tests {
    use html_sanitize::HtmlSanitize;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, HtmlSanitize, PartialEq, Eq)]
    struct TestStruct {
        #[serde(rename = "test-field")]
        test_field: String,
        #[serde(rename = "test-optional-field")]
        test_optional_field: Option<String>,
    }

    #[test]
    fn test_sanitize() {
        let test_struct = TestStruct {
            test_field: "<script>alert('test');</script>".to_string(),
            test_optional_field: Some("<script>alert('test');</script>".to_string()),
        };
        let sanitized = test_struct.sanitize();
        assert_eq!(sanitized.test_field, "");
        assert_eq!(sanitized.test_optional_field, Some("".to_string()));
        let test_struct = TestStruct {
            test_field: "<img src=x onerror='alert(1)'>".to_string(),
            test_optional_field: None,
        };
        let sanitized = test_struct.sanitize();
        assert_eq!(sanitized.test_field, "<img src=\"x\">");
        assert_eq!(sanitized.test_optional_field, None);
        let test_struct = TestStruct {
            test_field: "<img src=x onerror='alert(1)'>".to_string(),
            test_optional_field: Some("<img src=x onerror='alert(1)'>".to_string()),
        };
        let sanitized = test_struct.sanitize();
        assert_eq!(sanitized.test_field, "<img src=\"x\">");
        assert_eq!(sanitized.test_optional_field, Some("<img src=\"x\">".to_string()));
    }
    
    #[derive(Debug, Serialize, Deserialize, HtmlSanitize, PartialEq)]
    struct TestStruct2 {
        other_field: u64,
        test_field: String,
        test_optional_field: Option<String>,
        other_optional_field: Option<u64>,
        vec_field: Vec<String>,
        vec_field2: Vec<f32>,
        vec_optional_field2: Option<Vec<f32>>,
        double_vec_field2: Vec<Vec<f32>>,
        double_vec_optional_field2: Option<Vec<Vec<f32>>>,
    }
    
    #[test]
    fn test_sanitize2() {
        let test_struct = TestStruct2 {
            other_field: 1,
            test_field: "<script>alert('test');</script>".to_string(),
            test_optional_field: Some("<img src=x onerror='alert(1)'>".to_string()),
            other_optional_field: Some(2),
            vec_field: vec!["<img src=x onerror='alert(1)'>".to_string()],
            vec_field2: vec![1.0, 2.0],
            vec_optional_field2: Some(vec![3.0, 4.0]),
            double_vec_field2: vec![vec![1.0, 2.0]],
            double_vec_optional_field2: Some(vec![vec![3.0, 4.0]]),
        };
        let sanitized = test_struct.sanitize();
        assert_eq!(sanitized.other_field, 1);
        assert_eq!(sanitized.test_field, "");
        assert_eq!(sanitized.test_optional_field, Some("<img src=\"x\">".to_string()));
        assert_eq!(sanitized.other_optional_field, Some(2));
        assert_eq!(sanitized.vec_field, vec!["<img src=\"x\">"]);
        assert_eq!(sanitized.vec_field2, vec![1.0, 2.0]);
        assert_eq!(sanitized.vec_optional_field2, Some(vec![3.0, 4.0]));
        assert_eq!(sanitized.double_vec_field2, vec![vec![1.0, 2.0]]);
        assert_eq!(sanitized.double_vec_optional_field2, Some(vec![vec![3.0, 4.0]]));
    }
}