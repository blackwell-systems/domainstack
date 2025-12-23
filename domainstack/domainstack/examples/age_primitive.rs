use domainstack::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Age(u8);

impl Age {
    pub fn new(value: u8) -> Result<Self, ValidationError> {
        let rule = rules::range(18, 120);
        validate("age", &value, &rule)?;
        Ok(Self(value))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Validate for Age {
    fn validate(&self) -> Result<(), ValidationError> {
        let rule = rules::range(18, 120);
        validate("age", &self.0, &rule)
    }
}

fn main() {
    println!("=== Age Primitive Example ===\n");

    println!("1. Valid ages:");
    for age in [18, 25, 65, 120] {
        match Age::new(age) {
            Ok(a) => println!("   Age {} is valid ({})", age, a.value()),
            Err(e) => println!("   Error: {}", e),
        }
    }

    println!("\n2. Invalid ages:");
    for age in [0, 17, 121, 255] {
        match Age::new(age) {
            Ok(_) => println!("   Unexpected success for {}", age),
            Err(e) => {
                println!("   Age {} rejected:", age);
                for v in &e.violations {
                    println!("     [{} {}] {}", v.path, v.code, v.message);
                    if let Some(min) = v.meta.get("min") {
                        println!("       - Minimum: {}", min);
                    }
                    if let Some(max) = v.meta.get("max") {
                        println!("       - Maximum: {}", max);
                    }
                }
            }
        }
    }
}
