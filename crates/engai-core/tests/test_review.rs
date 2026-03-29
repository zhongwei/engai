use engai_core::review::calculate_next_review;

#[test]
fn test_quality_0_resets() {
    let result = calculate_next_review(0, 10, 2.5);
    assert_eq!(result.familiarity, 0);
    assert_eq!(result.interval, 1);
}

#[test]
fn test_quality_2_resets() {
    let result = calculate_next_review(2, 10, 2.5);
    assert_eq!(result.familiarity, 0);
    assert_eq!(result.interval, 1);
}

#[test]
fn test_quality_3_no_change() {
    let result = calculate_next_review(3, 5, 2.5);
    assert_eq!(result.familiarity, 1);
    assert_eq!(result.interval, 5);
    assert_eq!(result.ease_factor, 2.5);
}

#[test]
fn test_quality_4_increases() {
    let result = calculate_next_review(4, 5, 2.5);
    assert_eq!(result.familiarity, 2);
    assert!(result.interval > 1);
    assert!(result.ease_factor > 2.5);
}

#[test]
fn test_quality_5_increases_more() {
    let result_q4 = calculate_next_review(4, 5, 2.5);
    let result_q5 = calculate_next_review(5, 5, 2.5);
    assert!(result_q5.interval >= result_q4.interval);
    assert_eq!(result_q5.familiarity, 3);
}

#[test]
fn test_ease_factor_floor() {
    let result = calculate_next_review(0, 1, 1.3);
    assert_eq!(result.ease_factor, 1.3);

    let result2 = calculate_next_review(0, 1, 1.2);
    assert_eq!(result2.ease_factor, 1.3);
}
