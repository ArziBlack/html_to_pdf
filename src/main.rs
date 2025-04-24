use std::{fs, io};
use wkhtmltopdf::{Margin, Orientation, PdfApplication, Size, pdf};

fn convert(html_content: &str) {
    println!("Hello, world!");
    // make program wait for 4 seconds before proceeding
    std::thread::sleep(std::time::Duration::from_secs(4));
    println!("waited for 4 seconds...");
    let pdf_app = PdfApplication::new().expect("failed to initialize app");
    let mut pdf_builder = pdf_app
        .builder()
        .orientation(Orientation::Portrait)
        .page_size(pdf::PageSize::A4)
        .margin(Margin::from(Size::Inches(2)))
        .title("HTML to PDF testing")
        .build_from_html(&html_content)
        .expect("failed to build pdf");
    pdf_builder.save("test.pdf").expect("failed to save pdf");
    println!("generated PDF saved as: test.pdf");
    manipulate_files().expect("failed to manipulate files");
}

fn manipulate_files() -> io::Result<()> {
    let original_file = "test.pdf";
    let new_file = "test_new.pdf";

    fs::rename(original_file, new_file).expect("failed to rename file");
    println!("File renamed to: {}", new_file);

    // create directories
    fs::create_dir_all("path/to/destination1").expect("failed to create directory");
    fs::create_dir_all("path/to/destination2").expect("failed to create directory");
    fs::create_dir_all("path/to/destination3").expect("failed to create directory");
    fs::create_dir_all("path/to/destination4").expect("failed to create directory");
    fs::create_dir_all("path/to/destination5").expect("failed to create directory");

    // List of 5 destination paths
    let destinations = vec![
        "path/to/destination1/file.pdf",
        "path/to/destination2/file-2.pdf",
        "path/to/destination3/file-3.pdf",
        "path/to/destination4/file-4.pdf",
        "path/to/destination5/file-5.pdf",
    ];

    // Copy to each destination
    for dest in destinations {
        fs::copy(&new_file, dest)?;
        println!("Copied to {}", dest);
    }

    Ok(())
}

fn main() {
    convert("<html><body><h1>Hello, world!</h1></body></html>");
}
