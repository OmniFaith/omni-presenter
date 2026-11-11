use super::*;

#[test]
fn bundled_chapters_references_and_ranges_are_real() {
    let bible = BibleLibrary::default()
        .get(DEFAULT_BIBLE)
        .unwrap();
    assert_eq!(bible.books.len(), 66);
    assert_eq!(bible.chapter(16, 10).unwrap().len(), 3);
    let ester = bible.passage(16, 10, 2, 2).unwrap();
    assert!(ester.content.contains("Mardoqueu") || ester.content.contains("Mordecai"));
    assert!(ester.content.ends_with("Ester 10:2"));
    assert_ne!(ester.content, bible.passage(42, 3, 16, 16).unwrap().content);
    assert_eq!(bible.reference("1 João 2:1-3"), Some((61, 2, 1, 3)));
    assert_eq!(bible.reference("Jo3.16"), Some((42, 3, 16, 16)));
    assert!(bible.reference("Livro inexistente 1:1").is_none());
}

#[test]
fn imports_reject_invalid_content_and_accept_nested_arrays() {
    assert!(BibleTranslation::from_json(br#"{"books":[]}"#, "NAA", "Nova Almeida").is_err());
    let books: Vec<_> = BOOK_NAMES.iter().map(|name| serde_json::json!({"name":name,"chapters":[["Um versículo", "Outro versículo"]]})).collect();
    let translation = BibleTranslation::from_json(
        &serde_json::to_vec(&books).unwrap(),
        "TESTE",
        "Tradução teste",
    )
    .unwrap();
    assert_eq!(
        translation.passage(16, 1, 1, 2).unwrap().content,
        "Um versículo\nOutro versículo\nEster 1:1–2"
    );
    let mut broken = serde_json::to_value(&books).unwrap();
    broken[0]["chapters"][0][0] = serde_json::json!(42);
    assert!(
        BibleTranslation::from_json(&serde_json::to_vec(&broken).unwrap(), "TESTE", "Teste")
            .is_err()
    );

    // Test UTF-8 BOM stripping
    let mut bom_bytes = vec![0xef, 0xbb, 0xbf];
    bom_bytes.extend(serde_json::to_vec(&books).unwrap());
    let bom_res = BibleTranslation::from_json(&bom_bytes, "PT_AA", "Almeida Atualizada");
    assert!(bom_res.is_ok());
    assert_eq!(bom_res.unwrap().id, "PT_AA");
}

#[test]
fn bundled_bibles_load_cleanly_with_metadata() {
    let lib = BibleLibrary::default();
    let catalog = lib.catalog();
    assert!(!catalog.is_empty());
    let ara = lib.get("ARA");
    assert!(ara.is_some(), "Default bible ARA must load successfully");
    assert_eq!(ara.as_ref().unwrap().books[0].name, "Gênesis");
    assert_eq!(ara.as_ref().unwrap().books[42].name, "João");

    let kjv = lib.get("KJV");
    assert!(kjv.is_some(), "English KJV bible must load successfully");
    assert_eq!(kjv.as_ref().unwrap().books[0].name, "Genesis");
    assert_eq!(kjv.as_ref().unwrap().books[42].name, "John");
    assert_eq!(kjv.as_ref().unwrap().reference("John 3:16"), Some((42, 3, 16, 16)));

    let rvr = lib.get("RVR1960");
    assert!(rvr.is_some(), "Spanish RVR1960 bible must load successfully");
    assert_eq!(rvr.as_ref().unwrap().books[0].name, "Génesis");
    assert_eq!(rvr.as_ref().unwrap().books[42].name, "San Juan");
    assert_eq!(rvr.as_ref().unwrap().reference("San Juan 3:16"), Some((42, 3, 16, 16)));
    assert_eq!(rvr.as_ref().unwrap().reference("Juan 3:16"), Some((42, 3, 16, 16)));

    for (id, name) in &catalog {
        assert!(!id.is_empty());
        assert!(!name.is_empty());
        assert_ne!(name, "Bíblia em português", "Bibles should have real translation names: {}", id);
    }
}

