use lopdf::Document;
use std::path::Path;

fn main() {
    let pdf_path = Path::new("test/data/gpt-4o-system-card.pdf");
    
    match Document::load(pdf_path) {
        Ok(doc) => {
            println!("PDF loaded successfully!");
            let pages = doc.get_pages();
            println!("Page count: {}", pages.len());
            
            // Check if Info dictionary exists
            if let Ok(info_dict) = doc.trailer.get(b"Info") {
                println!("Info dictionary found!");
                if let Ok(info_ref) = info_dict.as_reference() {
                    if let Ok(info_obj) = doc.get_object(info_ref) {
                        if let Ok(info_dict) = info_obj.as_dict() {
                            println!("Info dictionary contents:");
                            for (key, value) in info_dict.iter() {
                                let key_str = String::from_utf8_lossy(key);
                                println!("  {}: {:?}", key_str, value);
                            }
                        }
                    }
                }
            } else {
                println!("No Info dictionary found in PDF");
            }
            
            // Try to extract some text
            if let Ok(text) = doc.extract_text(&[1]) {
                println!("First page text preview (first 200 chars):");
                let preview = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text
                };
                println!("{}", preview);
            } else {
                println!("Failed to extract text from first page");
            }
        }
        Err(e) => {
            println!("Failed to load PDF: {}", e);
        }
    }
}