use guts::core::cat::{parse_object, ParsedObject};
use guts::core::object::{Commit, GitObject};

#[test]
fn test_parse_commit_object() {
    // Étape 1 : créer un objet commit avec un tree fictif
    let commit = Commit {
        tree: "1234567890abcdef1234567890abcdef12345678".to_string(),
        parent: Some("abcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string()),
        message: "Initial commit".to_string(),
    };

    // Étape 2 : construire l’objet raw comme Git (header + contenu)
    let content = commit.content();
    let header = format!("commit {}\0", content.len());
    let raw = [header.as_bytes(), &content].concat();

    // Étape 3 : parser avec `parse_object`
    let parsed = parse_object(&raw).expect("parse failed");

    // Étape 4 : vérifier que le commit parsé correspond à ce qu'on a créé
    match parsed {
        ParsedObject::Commit(parsed_commit) => {
            assert_eq!(parsed_commit.tree, commit.tree, "Tree SHA mismatch");
            assert_eq!(parsed_commit.parent, commit.parent, "Parent SHA mismatch");
            assert_eq!(parsed_commit.message, commit.message, "Message mismatch");
        }
        ParsedObject::Other(obj_type, _) => {
            panic!(
                "Unexpected object type: got ParsedObject::Other: {}",
                obj_type
            );
        }
        _ => panic!("Unexpected object type"),
    }
}
