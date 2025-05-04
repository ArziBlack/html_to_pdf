use std::{fs, path::Path};
use base64::{engine::general_purpose, Engine};
use wkhtmltopdf::{Margin, Orientation, PdfApplication, Size, pdf};

struct ConversionOptions {
    orientation: Orientation,
    page_size: pdf::PageSize,
    margin_inches: u32,
    title: String,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            orientation: Orientation::Portrait,
            page_size: pdf::PageSize::A4,
            margin_inches: 1,
            title: "HTML to PDF conversion".to_string(),
        }
    }
}

fn convert_single(pdf_app: &PdfApplication, html_content: &str, output_path: &str, options: &ConversionOptions) -> Result<(), String> {
    let mut pdf_builder = pdf_app
        .builder()
        .orientation(options.orientation)
        .page_size(options.page_size)
        .margin(Margin::from(Size::Inches(options.margin_inches)))
        .title(&options.title)
        .build_from_html(&html_content)
        .map_err(|e| format!("failed to build pdf: {}", e))?;
    
    pdf_builder.save(output_path).map_err(|e| format!("failed to save pdf to {}: {}", output_path, e))?;
    println!("Generated PDF saved as: {}", output_path);
    Ok(())
}

fn batch_convert(items: Vec<(String, String, Option<ConversionOptions>)>) -> Vec<Result<(), String>> {
    println!("Starting batch conversion of {} files...", items.len());
    
    // Initialize the PDF application once for all conversions
    let pdf_app = match PdfApplication::new() {
        Ok(app) => app,
        Err(e) => {
            println!("Failed to initialize PDF application: {}", e);
            return vec![Err(format!("Failed to initialize PDF application: {}", e)); items.len()];
        }
    };
    
    let results = items.iter()
        .map(|(html, output_path, options)| {
            println!("Converting to: {}", output_path);
            // Create parent directory if it doesn't exist
            if let Some(parent) = Path::new(output_path).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))
                        .unwrap_or_else(|e| println!("Warning: {}", e));
                }
            }
            
            // Use provided options or default
            let default_options = ConversionOptions::default();
            let options = options.as_ref().unwrap_or(&default_options);
            convert_single(&pdf_app, html, output_path, options)
        })
        .collect();
    
    println!("Batch conversion completed!");
    results
}

fn batch_convert_from_files(file_paths: Vec<(String, String, Option<ConversionOptions>)>) -> Vec<Result<(), String>> {
    // Convert file paths to HTML content
    let items: Vec<(String, String, Option<ConversionOptions>)> = file_paths
        .into_iter()
        .map(|(html_path, output_path, options)| {
            match fs::read_to_string(&html_path) {
                Ok(content) => (content, output_path, options),
                Err(_e) => (
                    String::new(), 
                    output_path.clone(), 
                    None
                ), // Empty content will cause an error in conversion
            }
        })
        .collect();
    
    batch_convert(items)
}

fn main() {
    // Example 1: Converting HTML strings to PDFs
    let image_data = fs::read("image.jpg").unwrap();
    let image_base64 = general_purpose::STANDARD.encode(&image_data);
    let html_items = vec![
        (
            format!("<html><body><h1>Hello, world!</h1><img src=\"data:image/jpeg;base64,{image_base64}\" alt=\"Image\" style=\"width:400px;\"></body></html>"),
            "output/file1.pdf".to_string(),
            None
        ),
        (
            "<html><body><h1>Second document</h1><p>This is another PDF</p></body></html>".to_string(),
            "output/file2.pdf".to_string(),
            Some(ConversionOptions {
                orientation: Orientation::Landscape,
                margin_inches: 1,
                ..ConversionOptions::default()
            })
        ),
        (
            "<html><body><h1>Third document</h1><p>Yet another PDF</p></body></html>".to_string(),
            "output/file3.pdf".to_string(),
            None
        ),
    ];
    
    println!("Example 1: Converting HTML strings");
    let results = batch_convert(html_items);
    
    // Print any errors that occurred during batch conversion
    for (i, result) in results.iter().enumerate() {
        if let Err(err) = result {
            println!("Error converting item {}: {}", i + 1, err);
        }
    }
    
    // Example 2: Converting HTML files to PDFs (uncomment and modify paths as needed)
    /*
    println!("\nExample 2: Converting HTML files");
    let file_items = vec![
        (
            "input/page1.html".to_string(),
            "output/from_file1.pdf".to_string(),
            None
        ),
        (
            "input/page2.html".to_string(),
            "output/from_file2.pdf".to_string(),
            Some(ConversionOptions {
                title: "From File 2".to_string(),
                ..ConversionOptions::default()
            })
        ),
    ];
    
    let file_results = batch_convert_from_files(file_items);
    
    // Print any errors that occurred during batch conversion
    for (i, result) in file_results.iter().enumerate() {
        if let Err(err) = result {
            println!("Error converting file {}: {}", i + 1, err);
        }
    }
    */
}
