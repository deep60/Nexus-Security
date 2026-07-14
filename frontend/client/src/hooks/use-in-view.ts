import { useEffect, useRef, useState } from "react";

interface UseInViewOptions {
  /** 0–1 fraction of the element that must be visible to trigger. */
  threshold?: number;
  /** Margin around the root; negative bottom margin fires slightly before the element is fully on screen. */
  rootMargin?: string;
  /** Trigger only once (default) or re-run every time the element enters/leaves. */
  once?: boolean;
}

/**
 * Lightweight IntersectionObserver hook for scroll-triggered animation.
 * Dependency-free and SSR-safe. Users who prefer reduced motion are reported
 * as "in view" immediately so nothing stays hidden or animates.
 */
export function useInView<T extends Element = HTMLElement>({
  threshold = 0.15,
  rootMargin = "0px 0px -12% 0px",
  once = true,
}: UseInViewOptions = {}) {
  const ref = useRef<T | null>(null);
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const prefersReduced =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    if (prefersReduced || typeof IntersectionObserver === "undefined") {
      setInView(true);
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setInView(true);
            if (once) observer.unobserve(entry.target);
          } else if (!once) {
            setInView(false);
          }
        });
      },
      { threshold, rootMargin }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [threshold, rootMargin, once]);

  return { ref, inView } as const;
}
