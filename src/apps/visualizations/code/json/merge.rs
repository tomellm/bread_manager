use itertools::Itertools;
use serde_json::{Map, Value};

pub fn merge_all_maps(
    maps: impl IntoIterator<Item = Map<String, Value>>,
) -> Map<String, Value> {
    maps.into_iter().fold(Map::default(), merge_maps)
}

pub fn merge_maps(
    mut agg_map: Map<String, Value>,
    mut to_add_map: Map<String, Value>,
) -> Map<String, Value> {
    clean_uuids_recursive(&mut agg_map);
    clean_uuids_recursive(&mut to_add_map);
    merge_maps_recursive(agg_map, to_add_map)
}

fn clean_uuids_recursive(map: &mut Map<String, Value>) {
    const UUID_PLACEHOLDER: &str = "...uuid...";

    map.into_iter().for_each(|(key, val)| match val {
        Value::Object(internal_map) => clean_uuids_recursive(internal_map),
        Value::String(str) => {
            if key.contains("uuid") {
                *str = String::from(UUID_PLACEHOLDER)
            }
        }
        Value::Array(objs) => objs
            .iter_mut()
            .filter_map(|obj| match obj {
                Value::Object(map) => Some(map),
                _ => None,
            })
            .for_each(clean_uuids_recursive),
        _ => (),
    });
}

fn merge_maps_recursive(
    agg_map: Map<String, Value>,
    mut to_add_map: Map<String, Value>,
) -> Map<String, Value> {
    let mut agg_map = agg_map
        .into_iter()
        .filter(|(key, _)| !key.contains("uuid"))
        .map(|(key, value)| {
            let other = to_add_map.remove(&key);
            let new_val = match (value, other) {
                (Value::Object(map), Some(Value::Object(other_map))) => {
                    Value::Object(merge_maps_recursive(map, other_map))
                }
                (Value::Array(values), Some(Value::Array(other))) => {
                    let values = [values, other]
                        .concat()
                        .into_iter()
                        .unique()
                        .collect_vec();
                    Value::Array(match values.len() > 10 {
                        true => values.into_iter().take(10).collect_vec(),
                        false => values,
                    })
                }
                (value, Some(Value::Null)) => value,
                (Value::Null, Some(value)) => value,
                (value, Some(Value::Array(mut values))) => {
                    if values.len() < 10 && !values.iter().any(|v| value.eq(v))
                    {
                        values.push(value);
                    }
                    Value::Array(values)
                }
                (Value::Array(mut values), Some(value)) => {
                    if values.len() < 10 && !values.iter().any(|v| value.eq(v))
                    {
                        values.push(value);
                    }
                    Value::Array(values)
                }
                (val_one, Some(val_two)) if val_one.eq(&val_two) => val_one,
                (val_one, Some(val_two)) => {
                    Value::Array(vec![val_one, val_two])
                }
                (value, None) => value,
            };

            (key, new_val)
        })
        .collect::<Map<_, _>>();

    agg_map.append(&mut to_add_map);
    agg_map
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use serde_json::{json, Value};

    use crate::apps::visualizations::code::json::{
        merge::merge_maps, merge_all_maps,
    };
    macro_rules! json_map {
        ($($json:tt)+) => {
            match json!($($json)+) {
                Value::Object(map) => map,
                _ => unreachable!(),
            }
        };
    }

    #[test]
    fn merge_two_strings_returns_as_array() {
        // Arrange
        let one = json_map!({
            "key": "val1",
        });
        let two = json_map!({
            "key": "val2"
        });

        // Act
        let new = merge_maps(one, two);

        // Assert
        assert_eq!(
            json_map!({
                "key": ["val1", "val2"]
            }),
            new
        );
    }

    #[test]
    fn merge_two_equal_strings_returns_single_string() {
        // Arrange
        let one = json_map!({
            "key": "val"
        });
        let two = json_map!({
            "key": "val"
        });

        // Act
        let new = merge_maps(one, two);

        // Assert
        assert_eq!(
            json_map!({
                "key": "val"
            }),
            new
        );
    }

    #[test]
    fn merge_two_strings_with_different_keys_returns_map_with_both() {
        // Arrange
        let one = json_map!({
            "key1": "val"
        });
        let two = json_map!({
            "key2": "val"
        });

        // Act
        let new = merge_maps(one, two);

        // Assert
        assert_eq!(
            json_map!({
                "key1": "val",
                "key2": "val",
            }),
            new
        )
    }

    #[test]
    fn merge_nested_maps_returns_nested_map_with_all_keys() {
        // Arrange
        let one = json_map!({
            "key1" : {
                "key1": "val1",
            },
            "key2" : "val2"
        });
        let two = json_map!({
            "key1" : {
                "key2": "val2"
            },
            "key3" : "val3"
        });

        // Act
        let new = merge_maps(one, two);

        // Assert
        assert_eq!(
            json_map!({
                "key1" : {
                    "key1": "val1",
                    "key2": "val2",
                },
                "key2" : "val2",
                "key3" : "val3",
            }),
            new
        );
    }

    #[test]
    fn merge_various_types_with_same_key_returns_array_with_all() {
        // Arrange
        let objs = [
            json_map!({
                "key": true
            }),
            json_map!({
                "key": 10
            }),
            json_map!({
                "key": "str"
            }),
            json_map!({
                "key": ["some", "vals"]
            }),
        ];

        // Act
        let new = merge_all_maps(objs);

        // Assert
        assert_eq!(
            json_map!({
                "key": [true, 10, "str", "some", "vals"]
            }),
            new
        );
    }

    #[test]
    fn merge_array_with_item_returns_array_with_appended_item() {
        // Arrange
        let objs = [
            json_map!({
                "key": "val1"
            }),
            json_map!({
                "key": ["val2", "val3"]
            }),
        ];

        // Act
        let new = merge_all_maps(objs);

        // Assert
        assert_eq!(
            json_map!({
                "key": ["val2", "val3", "val1"]
            }),
            new
        )
    }

    #[test]
    fn invers_merge_array_with_item_returns_array_with_appended_item() {
        // Arrange
        let objs = [
            json_map!({
                "key": ["val1", "val2"]
            }),
            json_map!({
                "key": "val3"
            }),
        ];

        // Act
        let new = merge_all_maps(objs);

        // Assert
        assert_eq!(
            json_map!({
                "key": ["val1", "val2", "val3"]
            }),
            new
        )
    }

    #[test]
    fn merge_20_different_strings_returns_array_with_first_10() {
        // Arrange
        let objs = (0..20).map(|i| {
            let val = format!("val{i}");
            json_map!({
                "key": val
            })
        });

        // Act
        let new = merge_all_maps(objs);

        // Assert
        assert_eq!(
            json_map!({
                "key": ["val0","val1","val2","val3","val4","val5","val6","val7","val8","val9"]
            }),
            new
        );
    }

    #[test]
    fn merge_arrays_returns_single_array_with_all_elements() {
        // Arrange
        let objs1 = json_map!({
            "key": (0..5).map(|i| format!("val{i}")).collect_vec()
        });
        let objs2 = json_map!({
            "key": (5..10).map(|i| format!("val{i}")).collect_vec()
        });

        // Act
        let new = merge_maps(objs1, objs2);

        // Assert
        assert_eq!(
            json_map!({
                "key": ["val0","val1","val2","val3","val4","val5","val6","val7","val8","val9"]
            }),
            new
        );
    }
}
