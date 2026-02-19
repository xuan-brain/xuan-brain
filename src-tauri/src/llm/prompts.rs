/// AI prompt for extracting paper metadata from HTML
pub const HTML_PAPER_EXTRACTION_PROMPT: &str = r#"You are a scholarly paper metadata extraction assistant. Your task is to extract paper metadata from the provided HTML content.

Please extract the following information and return it in JSON format:
{
  "title": "Paper title (required)",
  "authors": ["Author 1", "Author 2"],
  "abstract": "Paper abstract or summary",
  "publication_year": 2024,
  "journal_name": "Journal or conference name",
  "conference_name": "Conference name if applicable",
  "volume": "Volume number",
  "issue": "Issue number",
  "pages": "Page range (e.g., '1-15')",
  "doi": "DOI identifier (e.g., '10.1000/xyz123')",
  "url": "Source URL if available",
  "keywords": ["keyword1", "keyword2"]
}

Rules:
1. If a field cannot be found, set it to null (for optional fields)
2. The "title" field is required - if not found, return {"error": "Title not found"}
3. Clean up any HTML entities or formatting in extracted text
4. For authors, separate multiple authors into an array
5. Extract year from publication date if available
6. Return ONLY the JSON object, no additional text, no markdown code blocks

HTML Content:
"#;
