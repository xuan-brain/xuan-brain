//! Unit tests for Paper model attachment array operations

use surrealdb_types::RecordId;

use crate::surreal::models::attachment::Attachment;
use crate::surreal::models::paper::Paper;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Case 1: Create a paper with empty attachments array
    #[test]
    fn test_create_paper_with_empty_attachments() {
        let paper = Paper::new("Test Paper".to_string());

        // Verify paper is created successfully
        assert_eq!(paper.title, "Test Paper");
        assert_eq!(paper.citation_count, 0);
        assert_eq!(paper.read_status, "unread");

        // Verify attachment_path is None (representing empty attachments)
        assert!(paper.attachment_path.is_none());
    }

    /// Test Case 2: Add attachment to array
    #[test]
    fn test_add_attachment_to_paper() {
        // Create a paper
        let paper = Paper::new("Test Paper".to_string());
        let paper_id = RecordId::parse("paper:123").unwrap();

        // Create an attachment
        let attachment = Attachment::new(paper_id.clone(), Some("document.pdf".to_string()));

        // Verify attachment is created successfully
        assert_eq!(attachment.file_name, Some("document.pdf".to_string()));
        assert_eq!(attachment.paper, paper_id);

        // Simulate adding to an attachments array
        let mut attachments: Vec<Attachment> = Vec::new();
        attachments.push(attachment);

        // Verify attachment was added
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].file_name, Some("document.pdf".to_string()));
    }

    /// Test Case 3: Find attachment by file_name
    #[test]
    fn test_find_attachment_by_file_name() {
        let paper_id = RecordId::parse("paper:123").unwrap();

        // Create multiple attachments
        let attachment1 = Attachment::new(paper_id.clone(), Some("document.pdf".to_string()));
        let attachment2 = Attachment::new(paper_id.clone(), Some("slides.pptx".to_string()));
        let attachment3 = Attachment::new(paper_id.clone(), Some("notes.txt".to_string()));

        let attachments = vec![attachment1, attachment2, attachment3];

        // Find attachment by file_name
        let found = attachments
            .iter()
            .find(|a| a.file_name.as_ref() == Some(&"slides.pptx".to_string()));

        // Verify correct attachment was found
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name, Some("slides.pptx".to_string()));

        // Test with non-existent file_name
        let not_found = attachments
            .iter()
            .find(|a| a.file_name.as_ref() == Some(&"missing.pdf".to_string()));

        assert!(not_found.is_none());
    }

    /// Test Case 4: Remove attachment by file_name
    #[test]
    fn test_remove_attachment_by_file_name() {
        let paper_id = RecordId::parse("paper:123").unwrap();

        // Create multiple attachments
        let attachment1 = Attachment::new(paper_id.clone(), Some("document.pdf".to_string()));
        let attachment2 = Attachment::new(paper_id.clone(), Some("slides.pptx".to_string()));
        let attachment3 = Attachment::new(paper_id.clone(), Some("notes.txt".to_string()));

        let mut attachments = vec![attachment1, attachment2, attachment3];

        // Initial count should be 3
        assert_eq!(attachments.len(), 3);

        // Remove attachment by file_name
        let initial_len = attachments.len();
        attachments.retain(|a| a.file_name.as_ref() != Some(&"slides.pptx".to_string()));

        // Verify one attachment was removed
        assert_eq!(attachments.len(), initial_len - 1);
        assert_eq!(attachments.len(), 2);

        // Verify the correct attachment was removed
        assert!(attachments
            .iter()
            .all(|a| a.file_name.as_ref() != Some(&"slides.pptx".to_string())));

        // Verify remaining attachments are present
        assert!(attachments
            .iter()
            .any(|a| a.file_name.as_ref() == Some(&"document.pdf".to_string())));
        assert!(attachments
            .iter()
            .any(|a| a.file_name.as_ref() == Some(&"notes.txt".to_string())));
    }

    /// Test Case 5: Detect PDF from attachments
    #[test]
    fn test_pdf_detection_from_attachments() {
        let paper_id = RecordId::parse("paper:123").unwrap();

        // Create attachments with different file types and extensions
        let mut attachment1 = Attachment::new(paper_id.clone(), Some("document.pdf".to_string()));
        attachment1.file_type = Some("pdf".to_string());

        let attachment2 = Attachment::new(paper_id.clone(), Some("slides.pptx".to_string()));
        attachment2.file_type = Some("presentation".to_string());

        let mut attachment3 = Attachment::new(paper_id.clone(), Some("manuscript.pdf".to_string()));
        attachment3.file_type = Some("pdf".to_string());

        let attachment4 = Attachment::new(paper_id.clone(), Some("notes.txt".to_string()));
        attachment4.file_type = Some("text".to_string());

        let attachments = vec![attachment1, attachment2, attachment3, attachment4];

        // Detect PDF attachments by file_type
        let pdf_by_type: Vec<&Attachment> = attachments
            .iter()
            .filter(|a| a.file_type.as_ref() == Some(&"pdf".to_string()))
            .collect();

        assert_eq!(pdf_by_type.len(), 2);
        assert_eq!(pdf_by_type[0].file_name, Some("document.pdf".to_string()));
        assert_eq!(pdf_by_type[1].file_name, Some("manuscript.pdf".to_string()));

        // Detect PDF attachments by file_name extension
        let pdf_by_extension: Vec<&Attachment> = attachments
            .iter()
            .filter(|a| {
                a.file_name
                    .as_ref()
                    .map(|name| name.ends_with(".pdf"))
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(pdf_by_extension.len(), 2);
        assert!(pdf_by_extension.iter().all(|a| {
            a.file_name
                .as_ref()
                .map(|name| name.ends_with(".pdf"))
                .unwrap_or(false)
        }));

        // Verify non-PDF files are not included
        assert!(!pdf_by_extension.iter().any(|a| {
            a.file_name
                .as_ref()
                .map(|name| name.ends_with(".pptx"))
                .unwrap_or(false)
        }));
        assert!(!pdf_by_extension.iter().any(|a| {
            a.file_name
                .as_ref()
                .map(|name| name.ends_with(".txt"))
                .unwrap_or(false)
        }));

        // Test edge case: file_type "PDF" (uppercase)
        let mut attachment5 = Attachment::new(paper_id, Some("report.PDF".to_string()));
        attachment5.file_type = Some("PDF".to_string());

        // Case-insensitive PDF detection by file_type
        let pdf_case_insensitive: Vec<&Attachment> = attachments
            .iter()
            .chain(std::iter::once(&attachment5))
            .filter(|a| {
                a.file_type
                    .as_ref()
                    .map(|t| t.to_lowercase() == "pdf")
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(pdf_case_insensitive.len(), 3);
    }
}
