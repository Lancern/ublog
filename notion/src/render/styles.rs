/// Get the CSS class name corresponding to the given Notion color.
pub fn get_color_style<T>(color: T) -> String
where
    T: AsRef<str>,
{
    let color = color.as_ref();
    format!("color-{}", color.replace('_', "-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_color_style() {
        assert_eq!("color-gray-background", get_color_style("gray_background"));
    }
}
