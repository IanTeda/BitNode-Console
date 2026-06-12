# Comprehensive Unit Testing with Fake Crate

This guide provides instructions for writing comprehensive unit tests using the `fake` crate to generate realistic random test data. Follow these patterns to ensure your tests are robust, maintainable, and cover edge cases effectively.

## Overview

The `fake` crate provides deterministic random data generation for testing. Use it to create realistic test data that follows the same constraints as production data, enabling property-based testing approaches.

## Test Data Generation Patterns

### SQLX Test

When writing unit tests for database functions use `#[sqlx::test]` derive macro to stub the test with a database

### Mock Data Methods

Implement mock data generation methods on your types for consistent test data creation:

```rust
impl MyStruct {
    /// Generates a mock instance with realistic random test data.
    ///
    /// Uses the fake crate to create varied test data that follows
    /// the same validation rules as production data.
    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            id: lib_domain::RowID::mock(),
            name: Self::generate_mock_name(),
            email: Self::generate_mock_email(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Generates a realistic name for testing.
    #[cfg(test)]
    fn generate_mock_name() -> String {
        use fake::Fake;
        use fake::faker::name::en::Name;

        Name().fake()
    }

    /// Generates a valid email address for testing.
    #[cfg(test)]
    fn generate_mock_email() -> String {
        use fake::Fake;
        use fake::faker::internet::en::SafeEmail;

        SafeEmail().fake()
    }
}
```

### Optional Field Randomisation

Use probabilistic generation for optional fields to test both `Some` and `None` cases:

```rust
/// Generates a mock description with 60% probability.
/// Tests both present and absent description scenarios.
#[cfg(test)]
fn generate_mock_description() -> Option<String> {
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::lorem::en::Words;

    let has_description: bool = Boolean(60).fake(); // 60% chance of Some
    if has_description {
        let words: Vec<String> = Words(3..10).fake();
        Some(words.join(" "))
    } else {
        None
    }
}
```

### Enum Value Distribution

Distribute enum values realistically rather than uniformly:

```rust
/// Generates category types with realistic distribution.
/// Most categories should be Income or Expense types.
#[cfg(test)]
fn generate_mock_category_type() -> CategoryType {
    use fake::Fake;
    use fake::faker::number::en::NumberWithFormat;

    let distribution: u8 = NumberWithFormat("##").fake::<String>().parse().unwrap_or(50);

    match distribution {
        0..=10 => CategoryType::Asset,      // 10% - rare
        11..=20 => CategoryType::Liability, // 10% - rare
        21..=30 => CategoryType::Equity,    // 10% - rare
        31..=70 => CategoryType::Expense,   // 40% - common
        _ => CategoryType::Income,          // 30% - common
    }
}
```

## Comprehensive Test Patterns

### Property-Based Testing with Randomisation

Test that your code works across many random inputs:

```rust
#[test]
fn mock_data_generation_produces_valid_instances() {
    // Test that mock data always produces valid instances
    for _ in 0..100 {
        let instance = MyStruct::mock();
        assert!(instance.is_valid(), "Mock data should always be valid");
    }
}

#[test]
fn optional_fields_are_properly_randomised() {
    let mut has_some_field = false;
    let mut has_none_field = false;

    // Run enough iterations to catch randomisation issues
    for _ in 0..100 {
        let instance = MyStruct::mock();
        if instance.optional_field.is_some() {
            has_some_field = true;
        } else {
            has_none_field = true;
        }
    }

    assert!(has_some_field && has_none_field,
        "Optional field should be randomised between Some and None");
}
```

### Edge Case Testing

Generate edge cases systematically:

```rust
#[test]
fn handles_edge_case_names() {
    let edge_cases = vec![
        "",  // Empty string
        "A", // Single character
        "A".repeat(100), // Very long name
        "Name with special chars: !@#$%", // Special characters
        "Jose Maria", // Unicode characters
        "  Leading spaces", // Leading whitespace
        "Trailing spaces  ", // Trailing whitespace
    ];

    for name in edge_cases {
        let instance = MyStruct {
            name: name.clone(),
            ..MyStruct::mock()
        };

        // Test your validation or processing logic
        let result = instance.validate_name();
        // Assert expected behaviour for each case
    }
}
```

### Validation Testing

Test that generated data passes your validation rules:

```rust
#[test]
fn mock_data_passes_validation_rules() {
    for _ in 0..50 {
        let category = Categories::mock();

        // Test code format (XXX.XXX.XXX)
        assert!(category.code.contains('.'));
        let parts: Vec<&str> = category.code.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.len() == 3));

        // Test name is not empty
        assert!(!category.name.is_empty());
        assert!(category.name.len() <= 100); // Assuming max length

        // Test timestamps are reasonable
        assert!(category.created_on <= chrono::Utc::now());
        assert!(category.updated_on >= category.created_on);
    }
}
```

## Fake Crate Usage Patterns

### Text Generation

```rust
use fake::Fake;
use fake::faker::lorem::en::{Word, Words, Sentence, Sentences, Paragraph, Paragraphs};

// Single word
let word: String = Word().fake();

// Multiple words
let words: Vec<String> = Words(3..8).fake();

// Sentence with random length
let sentence: String = Sentence(5..15).fake();

// Multiple sentences
let sentences: Vec<String> = Sentences(2..5).fake();

// Paragraph
let paragraph: String = Paragraph(3..7).fake();

// Multiple paragraphs
let paragraphs: Vec<String> = Paragraphs(2..4).fake();
```

### Numeric Generation

```rust
use fake::Fake;
use fake::faker::number::en::{Digit, NumberWithFormat};

// Single digit
let digit: String = Digit().fake();

// Formatted numbers
let phone: String = NumberWithFormat("(###) ###-####").fake();
let ssn: String = NumberWithFormat("###-##-####").fake();
let percentage: String = NumberWithFormat("##.#").fake();

// Random integers in range
use fake::rand::Rng;
let mut rng = fake::rand::rng();
let random_int = rng.random_range(1..=100);
```

### Internet Data

```rust
use fake::Fake;
use fake::faker::internet::en::{Username, SafeEmail, DomainSuffix, IPv4, IPv6};

// User data
let username: String = Username().fake();
let email: String = SafeEmail().fake();

// Network data
let domain: String = DomainSuffix().fake();
let ipv4: String = IPv4().fake();
let ipv6: String = IPv6().fake();
```

### Boolean and Choice Generation

```rust
use fake::Fake;
use fake::faker::boolean::en::Boolean;

// Probabilistic boolean
let is_active: bool = Boolean(80).fake(); // 80% chance of true

// Random choice from collection
use fake::faker::company::en::CompanyName;
let companies = vec!["Apple", "Google", "Microsoft", "Amazon"];
let random_company = fake::faker::random::en::Random::new().fake::<&str>();
```

### Date and Time Generation

```rust
use fake::Fake;
use fake::faker::chrono::en::DateTime;

// Random datetime
let random_datetime: chrono::DateTime<chrono::Utc> = DateTime().fake();

// Date in range
use chrono::{DateTime, Utc, Duration};
let start = Utc::now() - Duration::days(365);
let end = Utc::now();
let random_date = DateTime().fake_with_rng(&mut rng, start..end);
```

## Test Organization

### Test Module Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    mod mock_data_generation {
        // Tests for mock data generation methods
    }

    mod validation {
        // Tests for validation logic
    }

    mod business_logic {
        // Tests for business rules
    }

    mod edge_cases {
        // Tests for edge cases and error conditions
    }
}
```

### Helper Functions

Create reusable test helpers:

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_valid_instance() -> MyStruct {
        MyStruct::mock()
    }

    pub fn create_invalid_instance() -> MyStruct {
        MyStruct {
            name: "", // Invalid empty name
            ..MyStruct::mock()
        }
    }

    pub fn generate_test_cases() -> Vec<TestCase> {
        (0..20)
            .map(|_| TestCase {
                input: MyStruct::mock(),
                expected: compute_expected_result(),
            })
            .collect()
    }
}
```

## Integration with Existing Tests

### Database Tests

For database integration tests, use mock data to create test scenarios:

```rust
#[cfg(test)]
mod database_tests {
    use super::*;
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn create_category_with_mock_data() {
        let pool = setup_test_database().await;

        // Generate realistic test data
        let category = Categories::mock();

        // Test database insertion
        let result = category.create(&pool).await;
        assert!(result.is_ok());

        // Verify data was stored correctly
        let stored = Categories::find_by_id(&pool, category.id).await.unwrap().unwrap();
        assert_eq!(stored.name, category.name);
    }
}
```

## Best Practices

### Deterministic Seeds

For reproducible tests, use seeded random generation:

```rust
#[test]
fn reproducible_random_data() {
    use fake::rand::{Rng, SeedableRng};
    use fake::rand::rngs::StdRng;

    let seed = [42u8; 32]; // Fixed seed for reproducibility
    let mut rng = StdRng::from_seed(seed);

    let value1: String = fake::faker::lorem::en::Word().fake_with_rng(&mut rng);
    let value2: String = fake::faker::lorem::en::Word().fake_with_rng(&mut rng);

    // Same seed should produce same results
    let mut rng2 = StdRng::from_seed(seed);
    let value1_again: String = fake::faker::lorem::en::Word().fake_with_rng(&mut rng2);

    assert_eq!(value1, value1_again);
}
```

### Performance Considerations

- Generate mock data once per test, not in loops
- Use smaller iteration counts for expensive operations
- Cache expensive fake data generation

### Test Coverage Goals

- **Happy path**: Normal operation with valid data
- **Edge cases**: Boundary conditions and unusual inputs
- **Error conditions**: Invalid data and failure scenarios
- **Randomisation**: Property-based testing with varied inputs

## Quality Checklist

- [ ] Mock data generation methods exist for all major types
- [ ] Optional fields are properly randomised (Some/None)
- [ ] Generated data passes validation rules
- [ ] Edge cases are tested systematically
- [ ] Property-based tests use sufficient iterations
- [ ] Tests are deterministic (use seeds when needed)
- [ ] Test helpers are reusable across test modules
- [ ] Database tests use realistic mock data
- [ ] Performance impact of fake data generation is minimal

Remember: The goal is to test your code's behaviour across a wide range of realistic inputs, not just the happy path. Use the fake crate to generate varied, realistic test data that exposes edge cases and validates your code's robustness.