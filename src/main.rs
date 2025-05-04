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

fn batch_convert(pdf_app: &PdfApplication, items: Vec<(String, String, Option<ConversionOptions>)>) -> Vec<Result<(), String>> {
    println!("Starting batch conversion of {} files...", items.len());
    
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
            convert_single(pdf_app, html, output_path, options)
        })
        .collect();
    
    println!("Batch conversion completed!");
    results
}

fn batch_convert_from_files(pdf_app: &PdfApplication, file_paths: Vec<(String, String, Option<ConversionOptions>)>) -> Vec<Result<(), String>> {
    // Convert file paths to HTML content
    let items: Vec<(String, String, Option<ConversionOptions>)> = file_paths
        .into_iter()
        .map(|(html_path, output_path, options)| {
            match fs::read_to_string(&html_path) {
                Ok(content) => {
                    // Get the absolute directory of the HTML file
                    let html_dir = Path::new(&html_path).parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| String::from("."));
                    
                    // Replace relative image paths with absolute file:/// URLs
                    let fixed_content = fix_image_paths(&content, &html_dir);
                    
                    (fixed_content, output_path, options)
                },
                Err(e) => (
                    String::new(), 
                    output_path.clone(), 
                    None
                ), // Empty content will cause an error in conversion
            }
        })
        .collect();
    
    batch_convert(pdf_app, items)
}

// Function to fix relative image paths in HTML content by converting them to base64
fn fix_image_paths(html_content: &str, base_dir: &str) -> String {
    // Simple regex-free approach to replace image src attributes
    let mut result = String::new();
    let mut remaining = html_content;
    
    // Get the project root directory (where Cargo.toml is located)
    let project_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let project_dir_str = project_dir.to_string_lossy().to_string();
    
    println!("Project directory: {}", project_dir_str);
    println!("Base directory: {}", base_dir);
    
    while let Some(img_pos) = remaining.find("<img ") {
        // Add everything before the img tag
        result.push_str(&remaining[..img_pos + 5]); // +5 to include "<img "
        
        // Move remaining past "<img "
        remaining = &remaining[img_pos + 5..];
        
        // Find src attribute
        if let Some(src_pos) = remaining.find("src=\"") {
            // Add everything up to the src value
            result.push_str(&remaining[..src_pos + 5]); // +5 to include "src=\""
            
            // Move remaining past "src=\""
            remaining = &remaining[src_pos + 5..];
            
            // Find the end of the src attribute
            if let Some(end_pos) = remaining.find('"') {
                let src_value = &remaining[..end_pos];
                
                // Only process relative paths (not data: URLs or absolute URLs)
                if !src_value.starts_with("data:") && !src_value.starts_with("http:") && 
                   !src_value.starts_with("https:") && !src_value.starts_with("file:") {
                    
                    // Resolve the image path
                    let image_path = if src_value.starts_with("../") {
                        // For "../image.jpg" style paths, resolve them relative to project root
                        let file_name = src_value.trim_start_matches("../");
                        format!("{}/{}", project_dir_str, file_name)
                    } else if src_value.starts_with("./") {
                        // For "./image.jpg" style paths
                        let file_name = src_value.trim_start_matches("./");
                        format!("{}/{}", base_dir, file_name)
                    } else {
                        // For regular relative paths
                        format!("{}/{}", base_dir, src_value)
                    };
                    
                    println!("Trying to read image from: {}", image_path);
                    
                    // Try to read the image file
                    match fs::read(&image_path) {
                        Ok(image_data) => {
                            // Determine MIME type based on file extension
                            let mime_type = match Path::new(&image_path).extension().and_then(|ext| ext.to_str()) {
                                Some("jpg") | Some("jpeg") => "image/jpeg",
                                Some("png") => "image/png",
                                Some("gif") => "image/gif",
                                Some("svg") => "image/svg+xml",
                                Some("webp") => "image/webp",
                                _ => "image/jpeg", // Default to JPEG if unknown
                            };
                            
                            // Encode the image as base64
                            let image_base64 = general_purpose::STANDARD.encode(&image_data);
                            
                            // Create data URL
                            let data_url = format!("data:{};base64,{}", mime_type, image_base64);
                            
                            // Add the data URL
                            result.push_str(&data_url);
                            println!("Successfully embedded image: {}", image_path);
                        },
                        Err(e) => {
                            // If we can't read the image, try one more approach - look in the project root
                            let file_name = Path::new(src_value).file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| src_value.to_string());
                            
                            let root_image_path = format!("{}/{}", project_dir_str, file_name);
                            println!("Trying alternative path: {}", root_image_path);
                            
                            match fs::read(&root_image_path) {
                                Ok(image_data) => {
                                    // Determine MIME type based on file extension
                                    let mime_type = match Path::new(&root_image_path).extension().and_then(|ext| ext.to_str()) {
                                        Some("jpg") | Some("jpeg") => "image/jpeg",
                                        Some("png") => "image/png",
                                        Some("gif") => "image/gif",
                                        Some("svg") => "image/svg+xml",
                                        Some("webp") => "image/webp",
                                        _ => "image/jpeg", // Default to JPEG if unknown
                                    };
                                    
                                    // Encode the image as base64
                                    let image_base64 = general_purpose::STANDARD.encode(&image_data);
                                    
                                    // Create data URL
                                    let data_url = format!("data:{};base64,{}", mime_type, image_base64);
                                    
                                    // Add the data URL
                                    result.push_str(&data_url);
                                    println!("Successfully embedded image from root: {}", root_image_path);
                                },
                                Err(e2) => {
                                    // If we still can't read the image, keep the original path
                                    println!("Warning: Could not read image file {} or {}: {} / {}", 
                                             image_path, root_image_path, e, e2);
                                    result.push_str(src_value);
                                }
                            }
                        }
                    }
                } else {
                    // Keep the original src for non-relative paths
                    result.push_str(src_value);
                }
                
                // Move remaining past the src value and its closing quote
                remaining = &remaining[end_pos..];
            } else {
                // No closing quote found, just add the rest and break
                result.push_str(remaining);
                break;
            }
        } else {
            // No src attribute found, just add the rest and break
            result.push_str(remaining);
            break;
        }
    }
    
    // Add any remaining content
    result.push_str(remaining);
    
    result
}

fn main() {
    // Initialize the PDF application once for all conversions
    let pdf_app = match PdfApplication::new() {
        Ok(app) => app,
        Err(e) => {
            println!("Failed to initialize PDF application: {}", e);
            return;
        }
    };

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
            format!("<html><body><h1>Third document</h1><p>Yet another PDF</p><img src=\"data:image/jpeg;base64,{image_base64}\" alt=\"Image\" style=\"width:400px;\"></body></html>"),
            "output/file3.pdf".to_string(),
            None
        ),
    ];
    
    println!("Example 1: Converting HTML strings");
    let results = batch_convert(&pdf_app, html_items);
    
    // Print any errors that occurred during batch conversion
    for (i, result) in results.iter().enumerate() {
        if let Err(err) = result {
            println!("Error converting item {}: {}", i + 1, err);
        }
    }
    
    // Example 2: Converting HTML files to PDFs
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
    
    let file_results = batch_convert_from_files(&pdf_app, file_items);
    
    // Print any errors that occurred during batch conversion
    for (i, result) in file_results.iter().enumerate() {
        if let Err(err) = result {
            println!("Error converting file {}: {}", i + 1, err);
        }
    }

    // Example 3: Converting HTML string with relative image path
    println!("\nExample 3: Converting HTML string with relative image path");
    
    // Create HTML content with a relative image path
    let html_with_relative_image = r#"
    <html>
    <head>
        <title>Example 3 - Relative Image Path</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            h1 { color: #2c3e50; }
            img { max-width: 100%; border: 1px solid #ddd; }
        </style>
    </head>
    <body>
        <h1>Example with Relative Image Path</h1>
        <p>This example demonstrates fixing a relative image path in an HTML string.</p>
        <img src="image.jpg" alt="Test Image" style="width:300px;">
    </body>
    </html>
    "#;
    
    // Fix the image paths in the HTML content
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let current_dir_str = current_dir.to_string_lossy().to_string();
    println!("Processing HTML string with current directory: {}", current_dir_str);
    
    let fixed_html = fix_image_paths(html_with_relative_image, &current_dir_str);
    
    // Convert the fixed HTML to PDF
    let example3_items = vec![
        (
            fixed_html,
            "output/example3.pdf".to_string(),
            Some(ConversionOptions {
                title: "Example 3 - Fixed Relative Path".to_string(),
                ..ConversionOptions::default()
            })
        ),
    ];
    
    let example3_results = batch_convert(&pdf_app, example3_items);
    
    // Print any errors that occurred during conversion
    for (i, result) in example3_results.iter().enumerate() {
        if let Err(err) = result {
            println!("Error converting example 3 item {}: {}", i + 1, err);
        } else {
            println!("Successfully converted HTML string with relative image to PDF: output/example3.pdf");
        }
    }
}
