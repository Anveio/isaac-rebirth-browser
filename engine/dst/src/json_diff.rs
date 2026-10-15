//! First-difference reports for JSON values, so a failed equality property
//! names the field that differs instead of only reporting that one does.
use serde_json::Value;

/// The JSON pointer of the first value where `left` and `right` differ, with
/// both sides' values there.
pub fn first<'a>(left: &'a Value, right: &'a Value) -> Option<(String, &'a Value, &'a Value)> {
    let mut path = String::new();
    difference(left, right, &mut path).map(|(left, right)| (path, left, right))
}

/// Depth-first search for the first differing value; `path` becomes its JSON
/// pointer.
fn difference<'a>(
    left: &'a Value,
    right: &'a Value,
    path: &mut String,
) -> Option<(&'a Value, &'a Value)> {
    let length = path.len();
    let found = match (left, right) {
        (Value::Object(left_fields), Value::Object(right_fields))
            if left_fields.keys().eq(right_fields.keys()) =>
        {
            left_fields
                .iter()
                .zip(right_fields.values())
                .find_map(|((key, left), right)| {
                    path.truncate(length);
                    path.push('/');
                    path.push_str(key);
                    difference(left, right, path)
                })
        }
        (Value::Array(left_items), Value::Array(right_items))
            if left_items.len() == right_items.len() =>
        {
            left_items
                .iter()
                .zip(right_items)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    path.truncate(length);
                    path.push_str(&format!("/{index}"));
                    difference(left, right, path)
                })
        }
        _ => return (left != right).then_some((left, right)),
    };
    if found.is_none() {
        path.truncate(length);
    }
    found
}
