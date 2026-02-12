/**
 * PDF API functions for handling blob-based PDF loading and saving
 */

import { invokeCommand } from '@/lib/tauri';

export interface PdfBlobResponse {
  file_name: string;
  paper_title: string;
  paper_id: number;
  base64_data: string;
  size_bytes: number;
}

export interface PdfSaveResponse {
  success: boolean;
  file_path: string;
  size_bytes: number;
  message: string;
}

/**
 * Load PDF as blob from Rust backend
 * Converts base64 data to blob URL for use in PDF viewer
 * @param paperId - The paper ID to load
 * @returns Promise resolving to blob URL and metadata
 */
export async function loadPdfAsBlob(paperId: number): Promise<{
  blobUrl: string;
  fileName: string;
  paperTitle: string;
  sizeMB: number;
}> {
  try {
    const response = await invokeCommand<PdfBlobResponse>('read_pdf_as_blob', {
      paperId: paperId,
    });

    // Convert base64 to blob
    const binaryString = atob(response.base64_data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const blobUrl = URL.createObjectURL(blob);

    return {
      blobUrl,
      fileName: response.file_name,
      paperTitle: response.paper_title,
      sizeMB: response.size_bytes / (1024 * 1024),
    };
  } catch (error) {
    console.error(`Failed to load PDF blob for paper ${paperId}:`, error);
    throw new Error(
      `Failed to load PDF: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Save PDF blob data to backend
 * Converts blob to base64 and sends to Rust for storage
 * @param paperId - The paper ID
 * @param blob - The PDF blob to save
 * @returns Promise resolving to save response with file path and size
 */
export async function savePdfBlob(
  paperId: number,
  blob: Blob
): Promise<{
  success: boolean;
  filePath: string;
  sizeMB: number;
  message: string;
}> {
  try {
    // Convert blob to base64
    const base64Data = await blobToBase64(blob);

    const response = await invokeCommand<PdfSaveResponse>('save_pdf_blob', {
      paperId: paperId,
      base64Data: base64Data,
    });

    return {
      success: response.success,
      filePath: response.file_path,
      sizeMB: response.size_bytes / (1024 * 1024),
      message: response.message,
    };
  } catch (error) {
    console.error(`Failed to save PDF blob for paper ${paperId}:`, error);
    throw new Error(
      `Failed to save PDF: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Save PDF blob with annotations to backend
 * Converts blob to base64 and saves both PDF and annotation data
 * @param paperId - The paper ID
 * @param blob - The PDF blob to save
 * @param annotationsJson - Optional annotation data as JSON string
 * @returns Promise resolving to save response with file path and size
 */
export async function savePdfWithAnnotations(
  paperId: number,
  blob: Blob,
  annotationsJson?: string
): Promise<{
  success: boolean;
  filePath: string;
  sizeMB: number;
  message: string;
}> {
  try {
    // Convert blob to base64
    const base64Data = await blobToBase64(blob);

    const response = await invokeCommand<PdfSaveResponse>('save_pdf_with_annotations', {
      paperId: paperId,
      base64Data: base64Data,
      annotationsJson: annotationsJson || null,
    });

    return {
      success: response.success,
      filePath: response.file_path,
      sizeMB: response.size_bytes / (1024 * 1024),
      message: response.message,
    };
  } catch (error) {
    console.error(`Failed to save PDF with annotations for paper ${paperId}:`, error);
    throw new Error(
      `Failed to save PDF: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Convert blob to base64 string
 * @param blob - The blob to convert
 * @returns Promise resolving to base64 string
 */
export function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Remove 'data:application/pdf;base64,' prefix if present
      const base64 = result.includes(',') ? result.split(',')[1] : result;
      resolve(base64);
    };
    reader.onerror = () => {
      reject(new Error('Failed to convert blob to base64'));
    };
    reader.readAsDataURL(blob);
  });
}

/**
 * Revoke blob URL to free up memory
 * @param blobUrl - The blob URL to revoke
 */
export function revokePdfBlobUrl(blobUrl: string): void {
  URL.revokeObjectURL(blobUrl);
}
