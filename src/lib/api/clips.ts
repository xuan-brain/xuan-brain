/**
 * Clipping API functions for managing web content clippings
 * Uses fetch to call REST API at http://localhost:3000/api/clips
 */

/**
 * Request structure for creating a new clipping
 */
export interface CreateClippingRequest {
  title: string;
  url: string;
  content: string;
  source_domain: string;
  author?: string | null;
  published_date?: string | null;
  excerpt?: string | null;
  thumbnail_url?: string | null;
  tags?: string[];
}

/**
 * Response structure for clipping data from the API
 */
export interface ClippingResponse {
  id: string;
  title: string;
  url: string;
  content: string;
  source_domain: string;
  author: string | null;
  published_date: string | null;
  excerpt: string | null;
  thumbnail_url: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
  readStatus: number;
  notes: string | null;
  imagePaths: string[];
}

const API_BASE_URL = 'http://localhost:3030/api/clips';

/**
 * Create a new clipping
 * @param data - Clipping data to create
 * @returns Promise resolving to the created clipping response
 */
export async function createClip(data: CreateClippingRequest): Promise<ClippingResponse> {
  try {
    const response = await fetch(API_BASE_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(
        `Failed to create clipping: ${response.status} ${response.statusText}${errorText ? ` - ${errorText}` : ''}`
      );
    }

    const result: ClippingResponse = await response.json();
    return result;
  } catch (error) {
    console.error('Error creating clipping:', error);
    throw new Error(
      `Failed to create clipping: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * List clippings with optional pagination
 * @param params - Optional pagination parameters (limit, offset)
 * @returns Promise resolving to array of clipping responses
 */
export async function listClips(params?: {
  limit?: number;
  offset?: number;
}): Promise<ClippingResponse[]> {
  try {
    const url = new URL(API_BASE_URL);

    if (params?.limit !== undefined) {
      url.searchParams.append('limit', params.limit.toString());
    }

    if (params?.offset !== undefined) {
      url.searchParams.append('offset', params.offset.toString());
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(
        `Failed to list clippings: ${response.status} ${response.statusText}${errorText ? ` - ${errorText}` : ''}`
      );
    }

    const result: ClippingResponse[] = await response.json();
    return result;
  } catch (error) {
    console.error('Error listing clippings:', error);
    throw new Error(
      `Failed to list clippings: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Get a single clipping by ID
 * @param id - The clipping ID to retrieve
 * @returns Promise resolving to the clipping response
 */
export async function getClip(id: string): Promise<ClippingResponse> {
  try {
    const url = `${API_BASE_URL}/${id}`;
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(
        `Failed to get clipping: ${response.status} ${response.statusText}${errorText ? ` - ${errorText}` : ''}`
      );
    }

    const result: ClippingResponse = await response.json();
    return result;
  } catch (error) {
    console.error(`Error getting clipping ${id}:`, error);
    throw new Error(
      `Failed to get clipping: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}
