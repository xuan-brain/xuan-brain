/**
 * Clipping API functions for managing web content clippings
 * Uses Tauri commands for communication with the backend
 */

import { invokeCommand } from '../tauri';

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
  read_status: number;
  notes: string | null;
  image_paths: string[];
}

/**
 * Create a new clipping using Tauri command
 * @param data - Clipping data to create
 * @returns Promise resolving to the created clipping response
 */
export async function createClip(data: CreateClippingRequest): Promise<ClippingResponse> {
  try {
    const result = await invokeCommand<ClippingResponse>('create_clip', {
      payload: data,
    });
    console.info('Clipping created successfully:', result.id);
    return result;
  } catch (error) {
    console.error('Error creating clipping:', error);
    throw new Error(
      `Failed to create clipping: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * List clippings with optional pagination using Tauri command
 * @param params - Optional pagination parameters (limit, offset)
 * @returns Promise resolving to array of clipping responses
 */
export async function listClips(params?: {
  limit?: number;
  offset?: number;
}): Promise<ClippingResponse[]> {
  try {
    const result = await invokeCommand<ClippingResponse[]>('list_clips', {
      limit: params?.limit,
      offset: params?.offset,
    });
    console.info('Clippings loaded successfully:', result.length);
    return result;
  } catch (error) {
    console.error('Error listing clippings:', error);
    throw new Error(
      `Failed to list clippings: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Get a single clipping by ID using Tauri command
 * @param id - The clipping ID to retrieve
 * @returns Promise resolving to the clipping response
 */
export async function getClip(id: string): Promise<ClippingResponse> {
  try {
    const result = await invokeCommand<ClippingResponse | null>('get_clip', { id });
    if (!result) {
      throw new Error(`Clip not found: ${id}`);
    }
    console.info('Clip loaded successfully:', id);
    return result;
  } catch (error) {
    console.error(`Error getting clipping ${id}:`, error);
    throw new Error(
      `Failed to get clipping: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}
