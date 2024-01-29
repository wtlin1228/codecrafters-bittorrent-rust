use serde_json;

pub fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next().unwrap() {
        'l' => {
            return decode_array(encoded_value).1;
        }
        'i' => {
            return decode_integer(encoded_value).1;
        }
        'd' => {
            return decode_dictionary(encoded_value).1;
        }
        _ => {
            return decode_string(encoded_value).1;
        }
    }
}

fn _decode_string(encoded_value: &str) -> (&str, String) {
    // string is encoded as <number>:<string>
    //                              |        |
    //                         colon_index   |
    //                                    end_index
    let colon_index = encoded_value.find(':').unwrap();
    let string_length = encoded_value[..colon_index].parse::<u64>().unwrap();
    let end_index = colon_index + 1 + string_length as usize;
    let string = &encoded_value[colon_index + 1..end_index];
    (&encoded_value[end_index..], string.to_string())
}

fn decode_string(encoded_value: &str) -> (&str, serde_json::Value) {
    // string is encoded as <number>:<string>
    //                              |        |
    //                         colon_index   |
    //                                    end_index
    let (remaining, s) = _decode_string(encoded_value);
    (remaining, serde_json::Value::String(s))
}

fn decode_integer(encoded_value: &str) -> (&str, serde_json::Value) {
    // integer is encoded as i<number>e
    //                                |
    //                             end_index
    let end_index = encoded_value.find('e').unwrap();
    let integer = encoded_value[1..end_index].parse::<i64>().unwrap();
    (
        &encoded_value[end_index + 1..],
        serde_json::Value::Number(serde_json::Number::from(integer)),
    )
}

fn decode_array(encoded_value: &str) -> (&str, serde_json::Value) {
    // array is encoded as l<inner_encoded_value>e
    //                                           |
    //                                        end_index
    let mut encoded_value = &encoded_value[1..];
    let mut items: Vec<serde_json::Value> = vec![];
    loop {
        match encoded_value.chars().next().unwrap() {
            'e' => return (&encoded_value[1..], serde_json::Value::Array(items)),
            'l' => {
                let res = decode_array(encoded_value);
                encoded_value = res.0;
                items.push(res.1);
            }
            'i' => {
                let res = decode_integer(encoded_value);
                encoded_value = res.0;
                items.push(res.1);
            }
            'd' => {
                let res = decode_dictionary(encoded_value);
                encoded_value = res.0;
                items.push(res.1);
            }
            _ => {
                let res = decode_string(encoded_value);
                encoded_value = res.0;
                items.push(res.1);
            }
        }
    }
}

fn decode_dictionary(encoded_value: &str) -> (&str, serde_json::Value) {
    // array is encoded as d<key1><value1>...<keyN><valueN>e
    //                                                     |
    //                                                  end_index
    let mut encoded_value = &encoded_value[1..];
    let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    loop {
        let key: String;
        let val: serde_json::Value;
        match encoded_value.chars().next().unwrap() {
            'e' => return (&encoded_value[1..], serde_json::Value::Object(map)),
            _ => {
                let res = _decode_string(encoded_value);
                encoded_value = res.0;
                key = res.1;
            }
        }
        match encoded_value.chars().next().unwrap() {
            'l' => {
                let res = decode_array(encoded_value);
                encoded_value = res.0;
                val = res.1;
            }
            'i' => {
                let res = decode_integer(encoded_value);
                encoded_value = res.0;
                val = res.1;
            }
            'd' => {
                let res = decode_dictionary(encoded_value);
                encoded_value = res.0;
                val = res.1;
            }
            _ => {
                let res = decode_string(encoded_value);
                encoded_value = res.0;
                val = res.1;
            }
        }
        map.insert(key, val);
    }
}