/**
 * Input sanitization utilities
 * 
 * Prevents XSS attacks by sanitizing user inputs before rendering
 */

import DOMPurify from 'dompurify';

/**
 * Sanitize HTML content to prevent XSS
 */
export function sanitizeHtml(dirty: string): string {
  return DOMPurify.sanitize(dirty, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
  });
}

/**
 * Sanitize user-provided filename
 * Removes potentially dangerous characters
 */
export function sanitizeFilename(filename: string): string {
  // Remove path separators and null bytes
  let sanitized = filename
    .replace(/[/\\]/g, '_')
    .replace(/\0/g, '')
    .trim();
  
  // Limit length
  if (sanitized.length > 255) {
    const ext = sanitized.split('.').pop() || '';
    const name = sanitized.slice(0, 255 - ext.length - 1);
    sanitized = `${name}.${ext}`;
  }
  
  return sanitized;
}

/**
 * Sanitize string for display (escape HTML entities)
 */
export function escapeHtml(str: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#x27;',
    '/': '&#x2F;',
  };
  return str.replace(/[&<>"'/]/g, (char) => map[char]);
}

/**
 * Validate and sanitize URL
 */
export function sanitizeUrl(url: string): string | null {
  try {
    const parsed = new URL(url);
    // Only allow http and https
    if (!['http:', 'https:'].includes(parsed.protocol)) {
      return null;
    }
    return parsed.toString();
  } catch {
    return null;
  }
}

/**
 * Validate file type
 */
export function isAllowedFileType(filename: string, allowedTypes: string[]): boolean {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  return allowedTypes.includes(ext);
}

/**
 * Maximum file size validation
 */
export function isValidFileSize(size: number, maxSizeBytes: number): boolean {
  return size > 0 && size <= maxSizeBytes;
}