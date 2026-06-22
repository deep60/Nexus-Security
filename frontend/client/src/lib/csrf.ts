/**
 * CSRF Protection utilities
 * 
 * CSRF (Cross-Site Request Forgery) protection prevents attackers from making
 * authenticated requests on behalf of the user.
 * 
 * This implementation generates and validates CSRF tokens for state-changing
 * operations (POST, PUT, DELETE, PATCH).
 */

const CSRF_TOKEN_KEY = 'verdyx_csrf_token';
const CSRF_HEADER_NAME = 'X-CSRF-Token';

/**
 * Generate a random CSRF token
 */
export function generateCsrfToken(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join('');
}

/**
 * Get the stored CSRF token
 */
export function getCsrfToken(): string | null {
  return localStorage.getItem(CSRF_TOKEN_KEY);
}

/**
 * Store a new CSRF token
 */
export function setCsrfToken(token: string): void {
  localStorage.setItem(CSRF_TOKEN_KEY, token);
}

/**
 * Initialize CSRF token if not present
 */
export function initCsrfToken(): string {
  let token = getCsrfToken();
  if (!token) {
    token = generateCsrfToken();
    setCsrfToken(token);
  }
  return token;
}

/**
 * Get the CSRF header for fetch requests
 */
export function getCsrfHeaders(): HeadersInit {
  const token = getCsrfToken();
  if (token) {
    return { [CSRF_HEADER_NAME]: token };
  }
  return {};
}

/**
 * Wrapper for fetch that automatically includes CSRF token
 * Use for state-changing requests (POST, PUT, DELETE, PATCH)
 */
export async function csrfFetch(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  // Ensure CSRF token exists
  initCsrfToken();
  
  const headers = new Headers(options.headers);
  const csrfHeaders = getCsrfHeaders();
  
  Object.entries(csrfHeaders).forEach(([key, value]) => {
    headers.set(key, value);
  });
  
  // Add Content-Type if not already set and body is present
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }
  
  return fetch(url, {
    ...options,
    headers,
  });
}

/**
 * Fetch wrapper that automatically handles CSRF for all requests
 */
export async function verdyxFetch(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  // Ensure CSRF token exists
  initCsrfToken();
  
  const headers = new Headers(options.headers);
  
  // Add CSRF token for state-changing methods
  const method = options.method?.toUpperCase();
  if (method && ['POST', 'PUT', 'DELETE', 'PATCH'].includes(method)) {
    const csrfHeaders = getCsrfHeaders();
    Object.entries(csrfHeaders).forEach(([key, value]) => {
      headers.set(key, value);
    });
  }
  
  // Add Content-Type if not already set and body is present
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }
  
  return fetch(url, {
    ...options,
    headers,
  });
}