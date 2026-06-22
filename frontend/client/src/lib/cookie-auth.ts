/**
 * Cookie-based authentication utilities
 * 
 * NOTE: For full XSS protection, the backend should set httpOnly cookies.
 * This file provides helper functions to work with authentication cookies.
 */

const ACCESS_TOKEN_COOKIE = 'verdyx_access_token';
const REFRESH_TOKEN_COOKIE = 'verdyx_refresh_token';
const USER_COOKIE = 'verdyx_user';

/**
 * Get access token from cookie
 * Note: httpOnly cookies cannot be read by JavaScript - this is for non-httpOnly fallback
 */
export function getAccessTokenFromCookie(): string | null {
  if (typeof document === 'undefined') return null;
  
  const match = document.cookie.match(new RegExp('(^| )' + ACCESS_TOKEN_COOKIE + '=([^;]+)'));
  if (match) return match[2];
  return null;
}

/**
 * Get refresh token from cookie
 */
export function getRefreshTokenFromCookie(): string | null {
  if (typeof document === 'undefined') return null;
  
  const match = document.cookie.match(new RegExp('(^| )' + REFRESH_TOKEN_COOKIE + '=([^;]+)'));
  if (match) return match[2];
  return null;
}

/**
 * Set cookies (non-httpOnly - for demonstration)
 * In production, the backend should set httpOnly cookies
 */
export function setAuthCookies(accessToken: string, refreshToken: string, user: object, expiresIn: number): void {
  if (typeof document === 'undefined') return;
  
  const days = Math.ceil(expiresIn / (24 * 60 * 60));
  const expires = new Date(Date.now() + days * 24 * 60 * 60 * 1000).toUTCString();
  
  // Set cookies with secure flags for production
  const isProduction = import.meta.env.PROD;
  const secure = isProduction ? '; Secure' : '';
  const sameSite = '; SameSite=Lax';
  
  document.cookie = `${ACCESS_TOKEN_COOKIE}=${accessToken}; expires=${expires}; path=/${secure}${sameSite}`;
  document.cookie = `${REFRESH_TOKEN_COOKIE}=${refreshToken}; expires=${expires}; path=/${secure}${sameSite}`;
  document.cookie = `${USER_COOKIE}=${encodeURIComponent(JSON.stringify(user))}; expires=${expires}; path=/${secure}${sameSite}`;
}

/**
 * Clear authentication cookies
 */
export function clearAuthCookies(): void {
  if (typeof document === 'undefined') return;
  
  document.cookie = `${ACCESS_TOKEN_COOKIE}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/`;
  document.cookie = `${REFRESH_TOKEN_COOKIE}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/`;
  document.cookie = `${USER_COOKIE}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/`;
}

/**
 * Get user from cookie
 */
export function getUserFromCookie<T>(): T | null {
  if (typeof document === 'undefined') return null;
  
  const match = document.cookie.match(new RegExp('(^| )' + USER_COOKIE + '=([^;]+)'));
  if (match) {
    try {
      return JSON.parse(decodeURIComponent(match[2])) as T;
    } catch {
      return null;
    }
  }
  return null;
}

/**
 * Check if user is authenticated (has valid token)
 * This uses a combination of localStorage (for now) and cookie check
 */
export function isAuthenticated(): boolean {
  // Check both localStorage (fallback) and cookies
  const token = getAccessTokenFromCookie() || localStorage.getItem('token');
  return !!token;
}