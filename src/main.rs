use json::JsonValue;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    let json_str = r#"
{
  "user": {
    "id": 12345,
    "name": "John Doe",
    "email": "john.doe@example.com",
    "isActive": true,
    "profile": {
      "age": 30,
      "address": {
        "street": "123 Main St",
        "city": "Anytown",
        "state": "CA",
        "postalCode": "12345",
        "coordinates": {
          "latitude": 34.052235,
          "longitude": -118.243683
        }
      },
      "phoneNumbers": [
        {
          "type": "home",
          "number": "555-555-5555"
        },
        {
          "type": "work",
          "number": "555-555-1234"
        }
      ],
      "socialMedia": {
        "twitter": "@johndoe",
        "facebook": null,
        "linkedin": "https://www.linkedin.com/in/johndoe"
      }
    },
    "preferences": {
      "notifications": {
        "email": true,
        "sms": false
      },
      "theme": "dark",
      "language": "en-US"
    },
    "roles": ["admin", "editor"],
    "createdAt": "2023-05-27T12:34:56Z",
    "lastLogin": "2024-05-25T08:00:00Z"
  },
  "products": [
    {
      "id": "prod-001",
      "name": "Laptop",
      "description": "A high-end laptop for professionals",
      "price": 1499.99,
      "currency": "USD",
      "availability": "in stock",
      "ratings": {
        "average": 4.5,
        "reviews": 120
      },
      "categories": ["electronics", "computers"],
      "tags": ["laptop", "high-end", "professional"],
      "discount": {
        "percentage": 10,
        "validUntil": "2024-06-01T00:00:00Z"
      }
    },
    {
      "id": "prod-002",
      "name": "Smartphone",
      "description": "A smartphone with a great camera",
      "price": 799.99,
      "currency": "USD",
      "availability": "pre-order",
      "ratings": {
        "average": 4.7,
        "reviews": 230
      },
      "categories": ["electronics", "mobile"],
      "tags": ["smartphone", "camera", "high-end"],
      "discount": null
    }
  ],
  "orders": [
    {
      "orderId": "order-001",
      "userId": 12345,
      "productIds": ["prod-001", "prod-002"],
      "orderDate": "2024-05-26T14:22:00Z",
      "status": "shipped",
      "totalAmount": 2199.98,
      "currency": "USD",
      "payment": {
        "method": "credit card",
        "transactionId": "txn-789",
        "billingAddress": {
          "street": "123 Main St",
          "city": "Anytown",
          "state": "CA",
          "postalCode": "12345"
        }
      }
    }
  ],
  "settings": {
    "siteTitle": "My E-commerce Site",
    "adminEmail": "admin@example.com",
    "features": {
      "enableReviews": true,
      "enableWishlist": false,
      "enableMultiCurrency": true,
      "supportedCurrencies": ["USD", "EUR", "GBP"]
    }
  }
}
    "#;

    let start = Instant::now();
    println!("Generating TypeScript interfaces from JSON...\n");

    let parsed_json = json::parse(json_str).expect("Invalid JSON");
    let mut interfaces = HashMap::new();
    generate_typescript_interfaces(&vec![parsed_json], "Root", &mut interfaces);

    for (name, interface) in interfaces {
        if interface.contains(":") {
            println!("export type {}  = {{\n{}}}\n", name, interface);
        } else {
            println!("export type {} = {}\n", name, interface);
        }
    }

    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("Time taken: {:?}", duration);
}

fn generate_typescript_interfaces(arr: &Vec<JsonValue>, type_name: &str, interfaces: &mut HashMap<String, String>) {
    let mut output = String::new();

    if arr.len() == 1 {
        match &arr[0] {
            JsonValue::Object(map) => {
                for (key, val) in map.iter() {
                    let ts_type = json_value_to_ts_type(key, val, interfaces);
                    output.push_str(&format!("    {}{}: {};\n", key, if ts_type == "any" { "?" } else { "" }, ts_type));
                }
            }
            JsonValue::Array(arr) => {
                let t = json_value_to_ts_type(type_name, &JsonValue::Array(arr.clone()), interfaces);
                output.push_str(&format!("{};", t));
            }
            _ => {}
        }
    } else {
        let mut keys = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            if let JsonValue::Object(map) = item {
                for (key, val) in map.iter() {
                    let t = json_value_to_ts_type(key, val, interfaces);

                    if i == 0 {
                        keys.insert(key, (false, vec![t]));
                        continue;
                    }

                    for (key2, _) in keys.clone() {
                        if !map.iter().any(|(k, _)| k == key2) {
                            if let Some(old_key) = keys.get_mut(key2) {
                                old_key.0 = true;
                            }
                        }
                    }

                    if let Some(old_key) = keys.get_mut(key) {
                        if t == "any" {
                            old_key.0 = true;
                        } else if !old_key.1.contains(&t) {
                            if old_key.1.contains(&"any".to_string()) {
                                old_key.1 = vec![t];
                            } else {
                                old_key.1.push(t);
                            }
                        }
                    } else {
                        keys.insert(key, (true, vec![t]));
                    }
                }
            }
        }

        for (key, (is_optional, types)) in keys {
            output.push_str(&format!("    {}{}: {};\n", key, if is_optional { "?" } else { "" }, types.join(" | ")));
        }
    }

    interfaces.insert(type_name.to_string(), output);
}

fn json_value_to_ts_type(key: &str, value: &JsonValue, interfaces: &mut HashMap<String, String>) -> String {
    match value {
        JsonValue::Null => "any".to_string(),
        JsonValue::Short(_) | JsonValue::String(_) => "string".to_string(),
        JsonValue::Number(_) => "number".to_string(),
        JsonValue::Boolean(_) => "boolean".to_string(),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "any[]".to_string()
            } else {
                let objects: Vec<JsonValue> = arr.iter().filter(|item| item.is_object()).cloned().collect();
                let others: Vec<JsonValue> = arr.iter().filter(|item| !item.is_object()).cloned().collect();

                let mut types: Vec<String> = others.iter()
                    .map(|item| json_value_to_ts_type(key, item, interfaces)).collect();

                if !objects.is_empty() {
                    let type_name = format!("{}Data", to_kebab_case(key));
                    generate_typescript_interfaces(&objects, &type_name, interfaces);
                    types.push(type_name)
                }

                types.sort();
                types.dedup();

                if types.len() == 1 {
                    format!("{}[]", types[0])
                } else {
                    format!("({})[]", types.join(" | "))
                }
            }
        }
        JsonValue::Object(_) => {
            let type_name = format!("{}Data", to_kebab_case(key));
            generate_typescript_interfaces(&vec![value.clone()], &type_name, interfaces);
            type_name
        }
    }
}

fn to_kebab_case(s: &str) -> String {
    let words: Vec<&str> = s.split(|c| c == '-' || c == '_').collect();
    let mut camel_case = String::new();

    for (_i, word) in words.iter().enumerate() {
        let mut chars = word.chars();
        match chars.next() {
            None => {}
            Some(first) => {
                camel_case.push(first.to_ascii_uppercase());
                camel_case.push_str(chars.as_str());
            }
        }
    }

    camel_case
}
