export function SEO({ title, description, url = "https://nexus-security.com" }: { title: string; description: string; url?: string }) {
  // In React 19, these tags are automatically hoisted to the document <head>
  return (
    <>
      <title>{title} | Nexus-Security</title>
      <meta name="description" content={description} />
      
      {/* Open Graph */}
      <meta property="og:type" content="website" />
      <meta property="og:url" content={url} />
      <meta property="og:title" content={`${title} | Nexus-Security`} />
      <meta property="og:description" content={description} />
      <meta property="og:image" content={`${url}/og-image.png`} />
      
      {/* Twitter */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:url" content={url} />
      <meta name="twitter:title" content={`${title} | Nexus-Security`} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={`${url}/og-image.png`} />
    </>
  );
}
