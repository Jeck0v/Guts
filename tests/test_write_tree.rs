use anyhow::Result;

use guts::core::cat::{parse_object,parse_tree_body,ParsedObject};


#[test]
fn test_parse_tree_body_single_entry() -> Result<()> {
    // Exemple minimal d'entrée d'un tree Git:
    // mode = "100644"
    // nom = "file.txt"
    // hash = 20 octets arbitraires (ici 0x01, 0x02, ..., 0x14)
    
    let mut data = Vec::new();
    data.extend(b"100644 ");           // mode + espace
    data.extend(b"file.txt");          // nom
    data.push(0);                      // null byte
    data.extend((1u8..=20).collect::<Vec<u8>>());  // hash SHA1 fictif 20 octets
    
    let entries = parse_tree_body(&data)?;
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].mode, "100644");
    assert_eq!(entries[0].name, "file.txt");
    assert_eq!(entries[0].hash, {
        let mut h = [0u8; 20];
        for (i, b) in (1u8..=20).enumerate() {
            h[i] = b;
        }
        h
    });
    
    Ok(())
}

#[test]
fn test_parse_object_tree() -> Result<()> {
    // Construire un objet git complet (header + body) simulant un tree à 1 entrée
    let mut body = Vec::new();
    body.extend(b"100644 ");
    body.extend(b"file.txt");
    body.push(0);
    body.extend((1u8..=20).collect::<Vec<u8>>());
    
    let header = format!("tree {}\0", body.len());
    let mut data = header.into_bytes();
    data.extend(body);
    
    // Parse the full object
    let parsed = parse_object(&data)?;
    
    match parsed {
        ParsedObject::Tree(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].mode, "100644");
            assert_eq!(entries[0].name, "file.txt");
            assert_eq!(entries[0].hash, {
                let mut h = [0u8; 20];
                for (i, b) in (1u8..=20).enumerate() {
                    h[i] = b;
                }
                h
            });
        }
        _ => panic!("Expected ParsedObject::Tree"),
    }
    
    Ok(())
}

#[test]
fn test_parse_object_blob() -> Result<()> {
    // Construire un objet git blob (header + content)
    let content = b"Hello, world!";
    let header = format!("blob {}\0", content.len());
    let mut data = header.into_bytes();
    data.extend(content);
    
    let parsed = parse_object(&data)?;
    
    match parsed {
        ParsedObject::Blob(bytes) => {
            assert_eq!(bytes, b"Hello, world!");
        }
        _ => panic!("Expected ParsedObject::Blob"),
    }
    
    Ok(())
}

