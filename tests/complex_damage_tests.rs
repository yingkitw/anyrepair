//! Complex damage scenario tests for comprehensive format coverage
//! Tests realistic and complex damage patterns from LLM outputs

use anyrepair::repair;

#[test]
fn test_complex_json_deeply_nested_with_multiple_errors() {
    let input = r#"{
  "users": [
    {
      "id": 1,
      "profile": {
        "name": "Alice",
        "settings": {
          "notifications": true,
          "preferences": {
            "theme": "dark",
            "language": "en",
            "advanced": {
              "experimental": false,
              "debug": true,
            }
          },
        },
      },
    },
    {
      "id": 2,
      "profile": {
        "name": "Bob",
        "settings": {
          "notifications": false,
          "preferences": {
            "theme": "light",
            "language": "fr",
          },
        },
      },
    },
  ],
}"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("\"Alice\""));
    assert!(repaired.contains("\"Bob\""));
}

#[test]
fn test_complex_json_mixed_quote_styles_and_missing_quotes() {
    let input = r#"{
  'name': "John",
  age: 30,
  "email": 'john@example.com',
  active: true,
  tags: ['admin', "user",],
  metadata: {
    created: "2024-01-01",
    'updated': 2024-01-15,
  },
}"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("John"));
}

#[test]
fn test_complex_yaml_indentation_and_list_mixing() {
    let input = r#"application:
  name: MyApp
  version: 1.0.0
  services:
    - name: api
      port: 8080
      endpoints:
        - /health
        - /status
        - /metrics
      config:
        timeout: 30
        retries: 3
        backoff:
          initial: 1
          max: 60
          multiplier: 2
    - name: database
      port: 5432
      config:
        pool_size: 10
        timeout: 5
  features:
    - authentication
    - logging
    - monitoring
  environment:
    production:
      debug: false
      log_level: error
    development:
      debug: true
      log_level: debug"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("MyApp"));
    assert!(repaired.contains("api"));
}

#[test]
fn test_complex_markdown_mixed_formatting_and_nested_structures() {
    let input = r#"# Main Documentation

## Overview
This is a **complex** document with *various* formatting issues.

### Code Examples
```python
def hello_world():
    print("Hello, World!")
```

### Lists and Nesting
- Item 1
  - Subitem 1.1
    - Subitem 1.1.1
  - Subitem 1.2
- Item 2
  - Subitem 2.1

### Tables
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

### Links and Images
[Link Text](https://example.com)
![Alt Text](https://example.com/image.png)

### Inline Code
Use `variable_name` in your code.

### Blockquote
> This is a blockquote
> with multiple lines
> of text

### Bold and Italic
***Bold and italic*** text
__Bold__ and _italic_"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Main Documentation"));
}

#[test]
fn test_complex_xml_nested_with_attributes_and_entities() {
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <company name="TechCorp" founded="2020">
    <departments>
      <department id="1" name="Engineering">
        <teams>
          <team lead="Alice" size="5">
            <members>
              <member id="101" name="Bob" role="Senior Developer"/>
              <member id="102" name="Carol" role="Developer"/>
              <member id="103" name="Dave" role="Junior Developer"/>
            </members>
          </team>
          <team lead="Eve" size="3">
            <members>
              <member id="201" name="Frank" role="QA Engineer"/>
              <member id="202" name="Grace" role="QA Engineer"/>
            </members>
          </team>
        </teams>
      </department>
      <department id="2" name="Sales">
        <teams>
          <team lead="Henry" size="4">
            <members>
              <member id="301" name="Iris" role="Sales Manager"/>
              <member id="302" name="Jack" role="Sales Rep"/>
            </members>
          </team>
        </teams>
      </department>
    </departments>
  </company>
</root>"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("TechCorp"));
}

#[test]
fn test_complex_csv_with_quoted_fields_and_special_chars() {
    let input = r#"id,name,email,phone,address,notes
1,"Smith, John","john@example.com","+1-555-0100","123 Main St, Apt 4B, New York, NY 10001","Regular customer, prefers email"
2,"Doe, Jane","jane@example.com","+1-555-0101","456 Oak Ave, Suite 200, Los Angeles, CA 90001","VIP customer, high volume"
3,"Johnson, Bob","bob@example.com","+1-555-0102","789 Pine Rd, Building C, Chicago, IL 60601","New customer, requires follow-up"
4,"Williams, Alice","alice@example.com","+1-555-0103","321 Elm St, Floor 5, Houston, TX 77001","Inactive for 6 months"
5,"Brown, Charlie","charlie@example.com","+1-555-0104","654 Maple Dr, Unit 10, Phoenix, AZ 85001","Prefers phone contact""#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Smith"));
}

#[test]
fn test_complex_toml_with_nested_tables_and_arrays() {
    let input = r#"[package]
name = "myapp"
version = "0.1.0"
edition = "2021"
authors = ["Alice", "Bob"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
regex = "1.10"

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[[bin]]
name = "myapp"
path = "src/main.rs"

[[bin]]
name = "cli"
path = "src/cli.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("myapp"));
}

#[test]
fn test_complex_ini_with_multiple_sections_and_comments() {
    let input = r#"; Application Configuration File
; Last updated: 2024-01-15

[database]
host = localhost
port = 5432
user = admin
password = secret123
database = myapp_db
pool_size = 10
timeout = 30

[server]
host = 0.0.0.0
port = 8080
workers = 4
timeout = 60
ssl_enabled = true
ssl_cert = /etc/ssl/certs/server.crt
ssl_key = /etc/ssl/private/server.key

[logging]
level = info
format = json
file = /var/log/myapp.log
max_size = 100MB
max_backups = 5

[features]
authentication = true
caching = true
rate_limiting = true
compression = true"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("database"));
}

#[test]
fn test_complex_json_api_response_with_nested_errors() {
    let input = r#"{
  "status": "success",
  "data": {
    "users": [
      {
        "id": 1,
        "username": "alice",
        "email": "alice@example.com",
        "roles": ["admin", "user",],
        "permissions": {
          "read": true,
          "write": true,
          "delete": false,
        },
        "metadata": {
          "created_at": "2024-01-01T00:00:00Z",
          "last_login": "2024-01-15T12:30:00Z",
          "login_count": 42,
        },
      },
      {
        "id": 2,
        "username": "bob",
        "email": "bob@example.com",
        "roles": ["user",],
        "permissions": {
          "read": true,
          "write": false,
          "delete": false,
        },
        "metadata": {
          "created_at": "2024-01-05T00:00:00Z",
          "last_login": "2024-01-14T18:45:00Z",
          "login_count": 15,
        },
      },
    ],
    "pagination": {
      "page": 1,
      "per_page": 10,
      "total": 2,
      "pages": 1,
    },
  },
  "errors": [],
}"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("alice"));
    assert!(repaired.contains("bob"));
}

#[test]
fn test_complex_yaml_config_with_anchors_and_references() {
    let input = r#"defaults: &defaults
  timeout: 30
  retries: 3
  backoff: exponential

services:
  api:
    <<: *defaults
    port: 8080
    endpoints:
      - /api/v1/users
      - /api/v1/products
      - /api/v1/orders
    database:
      host: localhost
      port: 5432
      credentials:
        username: api_user
        password: secret
  
  cache:
    <<: *defaults
    port: 6379
    ttl: 3600
    max_connections: 100
  
  queue:
    <<: *defaults
    port: 5672
    workers: 4
    prefetch: 10"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("services"));
}

#[test]
fn test_complex_markdown_with_code_blocks_and_formatting() {
    let input = r#"# API Documentation

## Authentication

### Bearer Token
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" https://api.example.com/v1/users
```

### API Key
```python
import requests

headers = {
    'X-API-Key': 'your_api_key'
}
response = requests.get('https://api.example.com/v1/users', headers=headers)
```

## Endpoints

### GET /users
Returns a list of all users.

**Parameters:**
- `page` (integer): Page number (default: 1)
- `limit` (integer): Items per page (default: 10)

**Response:**
```json
{
  "data": [
    {
      "id": 1,
      "name": "John",
      "email": "john@example.com"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 100
  }
}
```

### POST /users
Creates a new user.

**Request Body:**
```json
{
  "name": "Jane",
  "email": "jane@example.com",
  "password": "secure_password"
}
```"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("API Documentation"));
}

#[test]
fn test_complex_xml_with_cdata_and_mixed_content() {
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<document>
  <metadata>
    <title>Complex Document</title>
    <author>John Doe</author>
    <created>2024-01-01</created>
  </metadata>
  <content>
    <section id="intro">
      <heading>Introduction</heading>
      <paragraph>This is an introduction with <emphasis>important</emphasis> text.</paragraph>
    </section>
    <section id="details">
      <heading>Details</heading>
      <subsection>
        <title>Subsection 1</title>
        <items>
          <item priority="high">Item 1</item>
          <item priority="medium">Item 2</item>
          <item priority="low">Item 3</item>
        </items>
      </subsection>
      <subsection>
        <title>Subsection 2</title>
        <code><![CDATA[
function example() {
  console.log("Hello, World!");
}
        ]]></code>
      </subsection>
    </section>
  </content>
</document>"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Complex Document"));
}

#[test]
fn test_complex_csv_with_multiline_fields() {
    let input = r#"id,name,description,tags
1,"Product A","This is a multi-line
description for Product A
with several lines","electronics,gadget,new"
2,"Product B","Single line description","books,education"
3,"Product C","Another multi-line
description that spans
multiple lines
with detailed info","home,furniture,sale""#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Product"));
}

#[test]
fn test_complex_json_with_unicode_and_escape_sequences() {
    let input = r#"{
  "greeting": "Hello, 世界",
  "emoji": "🚀",
  "special": "Line 1\nLine 2\tTabbed",
  "unicode_escape": "\u0048\u0065\u006c\u006c\u006f",
  "nested": {
    "chinese": "你好",
    "arabic": "مرحبا",
    "russian": "Привет",
    "mixed": "Hello 世界 🌍",
  },
}"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Hello"));
}

#[test]
fn test_complex_yaml_with_multiline_strings() {
    let input = r#"description: |
  This is a multi-line
  description that spans
  multiple lines
  with preserved formatting

summary: >
  This is a folded
  multi-line string
  that will be collapsed
  into a single line

code: |
  def hello():
      print("Hello")
      return True

config:
  name: MyApp
  settings:
    - key: value1
    - key: value2"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("description"));
}

#[test]
fn test_complex_toml_with_inline_tables() {
    let input = r#"[package]
name = "myapp"
version = "0.1.0"

[dependencies]
point = { x = 1, y = 2 }
database = { url = "postgres://localhost", pool_size = 10, timeout = 30 }

[[products]]
name = "Hammer"
sku = 738594937
color = "gray"
physical = { color = "red", shape = "round", weight = 500 }

[[products]]
name = "Nail"
sku = 284758393
color = "gray"
physical = { color = "gray", shape = "cylinder", weight = 100 }"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("myapp"));
}

#[test]
fn test_complex_ini_with_special_characters() {
    let input = r#"[database]
connection_string = Server=localhost;Database=mydb;User=admin;Password=p@ssw0rd!
url = https://db.example.com:5432/mydb?ssl=true&timeout=30

[paths]
home = C:\Users\John\Documents
config = /etc/myapp/config.ini
temp = /tmp/myapp_${USER}_temp

[options]
enabled = yes
debug = no
log_level = INFO
timeout = 30
max_connections = 100"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("database"));
}

#[test]
fn test_complex_markdown_with_tables_and_lists() {
    let input = r#"# Data Report

## Summary Table

| Metric | Q1 | Q2 | Q3 | Q4 | Total |
|--------|----|----|----|----|-------|
| Revenue | 100K | 120K | 150K | 180K | 550K |
| Expenses | 60K | 70K | 80K | 90K | 300K |
| Profit | 40K | 50K | 70K | 90K | 250K |

## Detailed Breakdown

### Revenue Sources
- Product Sales
  - Category A: 200K
  - Category B: 150K
  - Category C: 100K
- Services
  - Consulting: 80K
  - Support: 20K

### Expense Categories
- Personnel: 150K
- Operations: 100K
- Marketing: 50K

## Recommendations
1. Increase marketing budget
   - Focus on digital channels
   - Expand social media presence
2. Optimize operations
   - Reduce overhead
   - Improve efficiency"#;

    let result = repair(input);
    assert!(result.is_ok());
    let repaired = result.unwrap();
    assert!(repaired.contains("Summary Table"));
}
