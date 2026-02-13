//! Tag utilities for categorization and styling.
//!
//! Provides deterministic color assignment for tags based on FNV-1a hashing.

/// Tag color classes for Tailwind CSS styling.
pub struct TagColor {
    pub bg_class: &'static str,
    pub text_class: &'static str,
}

/// Color palette for tags (8 distinct colors).
const TAG_COLORS: [TagColor; 8] = [
    TagColor {
        bg_class: "bg-purple-100",
        text_class: "text-purple-700",
    },
    TagColor {
        bg_class: "bg-blue-100",
        text_class: "text-blue-700",
    },
    TagColor {
        bg_class: "bg-green-100",
        text_class: "text-green-700",
    },
    TagColor {
        bg_class: "bg-yellow-100",
        text_class: "text-yellow-700",
    },
    TagColor {
        bg_class: "bg-orange-100",
        text_class: "text-orange-700",
    },
    TagColor {
        bg_class: "bg-pink-100",
        text_class: "text-pink-700",
    },
    TagColor {
        bg_class: "bg-indigo-100",
        text_class: "text-indigo-700",
    },
    TagColor {
        bg_class: "bg-teal-100",
        text_class: "text-teal-700",
    },
];

/// Compute FNV-1a hash of a string.
///
/// This is a fast, deterministic hash function that produces consistent
/// results across Rust and JavaScript implementations.
fn fnv1a_hash(s: &str) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in s.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Get deterministic color classes for a tag.
///
/// Uses FNV-1a hash to map tag names to colors consistently.
/// The same tag will always get the same color.
pub fn get_tag_color(tag: &str) -> &'static TagColor {
    let hash = fnv1a_hash(tag);
    let index = (hash % 8) as usize;
    &TAG_COLORS[index]
}

/// Get Tailwind CSS classes for a tag (combined bg + text).
pub fn get_tag_classes(tag: &str) -> String {
    let color = get_tag_color(tag);
    format!("{} {}", color.bg_class, color.text_class)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_hash_consistency() {
        // Same input should always produce same output
        assert_eq!(fnv1a_hash("test"), fnv1a_hash("test"));
        assert_eq!(fnv1a_hash("HealthCheck"), fnv1a_hash("HealthCheck"));
    }

    #[test]
    fn test_fnv1a_hash_different_inputs() {
        // Different inputs should (usually) produce different outputs
        assert_ne!(fnv1a_hash("auth"), fnv1a_hash("network"));
        assert_ne!(fnv1a_hash("sync"), fnv1a_hash("debug"));
    }

    #[test]
    fn test_tag_color_determinism() {
        // Same tag should always get same color
        let color1 = get_tag_color("HealthCheck");
        let color2 = get_tag_color("HealthCheck");
        assert_eq!(color1.bg_class, color2.bg_class);
        assert_eq!(color1.text_class, color2.text_class);
    }

    #[test]
    fn test_tag_classes_format() {
        let classes = get_tag_classes("test");
        assert!(classes.contains("bg-"));
        assert!(classes.contains("text-"));
    }
}
