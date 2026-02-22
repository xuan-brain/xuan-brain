-- ============================================
-- XUAN-BRAIN DATABASE SCHEMA (SurrealDB 3.0)
-- ============================================
-- A multi-model schema for research paper management
-- Supports: Documents, Graph Relations, BM25 Full-Text Search
-- ============================================

-- ============================================
-- PAPER (文献主表，带全文索引)
-- ============================================
DEFINE TABLE paper SCHEMAFULL;
DEFINE FIELD title ON paper TYPE string;
DEFINE FIELD abstract_text ON paper TYPE option<string>;
DEFINE FIELD doi ON paper TYPE option<string>;
DEFINE FIELD publication_year ON paper TYPE option<int>;
DEFINE FIELD publication_date ON paper TYPE option<string>;
DEFINE FIELD journal_name ON paper TYPE option<string>;
DEFINE FIELD conference_name ON paper TYPE option<string>;
DEFINE FIELD volume ON paper TYPE option<string>;
DEFINE FIELD issue ON paper TYPE option<string>;
DEFINE FIELD pages ON paper TYPE option<string>;
DEFINE FIELD url ON paper TYPE option<string>;
DEFINE FIELD citation_count ON paper TYPE int DEFAULT 0;
DEFINE FIELD read_status ON paper TYPE string DEFAULT 'unread';
DEFINE FIELD notes ON paper TYPE option<string>;
DEFINE FIELD attachment_path ON paper TYPE option<string>;
DEFINE FIELD created_at ON paper TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON paper TYPE datetime DEFAULT time::now();
DEFINE FIELD deleted_at ON paper TYPE option<datetime>;

-- DOI 唯一索引
DEFINE INDEX doi_unique ON paper FIELDS doi UNIQUE;

-- BM25 全文索引 (SurrealDB 3.0 uses FULLTEXT ANALYZER instead of SEARCH ANALYZER)
DEFINE ANALYZER paper_analyzer TOKENIZERS blank,class FILTERS lowercase,snowball(english);
DEFINE INDEX paper_title_search ON paper FIELDS title FULLTEXT ANALYZER paper_analyzer;
DEFINE INDEX paper_abstract_text_search ON paper FIELDS abstract_text FULLTEXT ANALYZER paper_analyzer;

-- ============================================
-- AUTHOR (作者表)
-- ============================================
DEFINE TABLE author SCHEMAFULL;
DEFINE FIELD name ON author TYPE string;
DEFINE FIELD affiliation ON author TYPE option<string>;
DEFINE FIELD email ON author TYPE option<string>;
DEFINE FIELD created_at ON author TYPE datetime DEFAULT time::now();

-- ============================================
-- KEYWORD (关键词表)
-- ============================================
DEFINE TABLE keyword SCHEMAFULL;
DEFINE FIELD word ON keyword TYPE string;
DEFINE INDEX keyword_word_unique ON keyword FIELDS word UNIQUE;

-- ============================================
-- LABEL (标签表)
-- ============================================
DEFINE TABLE label SCHEMAFULL;
DEFINE FIELD name ON label TYPE string;
DEFINE FIELD color ON label TYPE string DEFAULT '#1976D2';
DEFINE FIELD document_count ON label TYPE int DEFAULT 0;
DEFINE FIELD created_at ON label TYPE datetime DEFAULT time::now();

-- ============================================
-- CATEGORY (分类表 - 树形结构)
-- ============================================
DEFINE TABLE category SCHEMAFULL;
DEFINE FIELD name ON category TYPE string;
DEFINE FIELD parent ON category TYPE option<record<category>>;
DEFINE FIELD sort_order ON category TYPE int DEFAULT 0;
DEFINE FIELD created_at ON category TYPE datetime DEFAULT time::now();

-- ============================================
-- RELATIONSHIP TABLES (图边表)
-- SurrealDB RELATION type enables graph traversal
-- ============================================

-- Paper-Author relationship with order and corresponding author info
DEFINE TABLE paper_author SCHEMAFULL TYPE RELATION FROM paper TO author;
DEFINE FIELD author_order ON paper_author TYPE int DEFAULT 0;
DEFINE FIELD is_corresponding ON paper_author TYPE bool DEFAULT false;

-- Paper-Keyword relationship
DEFINE TABLE paper_keyword SCHEMAFULL TYPE RELATION FROM paper TO keyword;

-- Paper-Label relationship
DEFINE TABLE paper_label SCHEMAFULL TYPE RELATION FROM paper TO label;

-- Paper-Category relationship
DEFINE TABLE paper_category SCHEMAFULL TYPE RELATION FROM paper TO category;

-- ============================================
-- ATTACHMENT (附件表)
-- ============================================
DEFINE TABLE attachment SCHEMAFULL;
DEFINE FIELD paper ON attachment TYPE record<paper>;
DEFINE FIELD file_type ON attachment TYPE option<string>;
DEFINE FIELD file_name ON attachment TYPE option<string>;
DEFINE FIELD file_path ON attachment TYPE option<string>;
DEFINE FIELD file_size ON attachment TYPE option<int>;
DEFINE FIELD created_at ON attachment TYPE datetime DEFAULT time::now();

-- ============================================
-- FUTURE: Vector Search (可选扩展)
-- ============================================
-- Uncomment when implementing semantic search with embeddings
--
-- DEFINE TABLE paper_embedding SCHEMAFULL;
-- DEFINE FIELD paper ON paper_embedding TYPE record<paper>;
-- DEFINE FIELD embedding ON paper_embedding TYPE array<float> ASSERT array::len($value) = 1536;
-- DEFINE INDEX paper_vector_idx ON paper_embedding FIELDS embedding HNSW DIMENSION 1536;
