import { useEffect, useRef, useState } from 'react';
import './ElementsBg.css';

const ElementsBg = () => {
  const containerRef = useRef(null);
  const [availableIcons, setAvailableIcons] = useState([]);
  const colorCacheRef = useRef({});
  const svgCacheRef = useRef({});
  const objectUrlCacheRef = useRef({});
  const imageCacheRef = useRef({});
  const svgFetchPromiseRef = useRef({}); 
  const colorPromiseRef = useRef({});
  const styleSheetRef = useRef(null);
  const keyframesAddedRef = useRef(new Set());

  const ICON_SPAWN_INTERVAL = 1500;
  const ANIMATION_DURATION_MIN = 6000;
  const ANIMATION_DURATION_MAX = 9000;
  const CIRCLE_FACTOR = 2;

  useEffect(() => {
    const fetchAvailableIcons = async () => {
      try {
        const iconNames = ['air.svg', 'earth.svg', 'energy.svg', 'fire.svg', 'water.svg', 'wood.svg'];
        setAvailableIcons(iconNames);
        
        for (const icon of iconNames) {
          await Promise.all([
            preloadImage(icon),
            extractIconColor(icon)
          ]);
        }
      } catch (error) {
        console.error('Failed to fetch available icons:', error);
        const iconNames = ['air.svg', 'earth.svg', 'energy.svg', 'fire.svg', 'water.svg', 'wood.svg'];
        setAvailableIcons(iconNames);
        for (const icon of iconNames) {
          preloadImage(icon).catch(err => console.error(`Failed to preload ${icon}:`, err));
        }
      }
    };
    
    fetchAvailableIcons();

    const style = document.createElement('style');
    document.head.appendChild(style);
    styleSheetRef.current = style;

    return () => {
      if (styleSheetRef.current) {
        styleSheetRef.current.remove();
      }
      Object.values(objectUrlCacheRef.current).forEach((url) => {
        if (typeof url === 'string') {
          URL.revokeObjectURL(url);
        }
      });
    };
  }, []);

  const loadSvgAsset = async (iconName) => {
    if (svgCacheRef.current[iconName]) {
      return svgCacheRef.current[iconName];
    }

    if (svgFetchPromiseRef.current[iconName]) {
      return svgFetchPromiseRef.current[iconName];
    }

    const loadPromise = (async () => {
      const response = await fetch(`/textures/elements/${iconName}`);
      const svgText = await response.text();
      svgCacheRef.current[iconName] = svgText;
      return svgText;
    })();

    svgFetchPromiseRef.current[iconName] = loadPromise;
    return loadPromise;
  };

  const ensureImageUrl = async (iconName) => {
    if (objectUrlCacheRef.current[iconName]) {
      return objectUrlCacheRef.current[iconName];
    }

    const svgText = await loadSvgAsset(iconName);
    const blob = new Blob([svgText], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    objectUrlCacheRef.current[iconName] = url;
    return url;
  };

  const preloadImage = async (iconName) => {
    if (imageCacheRef.current[iconName]) {
      return imageCacheRef.current[iconName];
    }

    const url = await ensureImageUrl(iconName);
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        imageCacheRef.current[iconName] = img;
        resolve(img);
      };
      img.onerror = () => {
        const fallbackImg = new Image();
        imageCacheRef.current[iconName] = fallbackImg;
        resolve(fallbackImg);
      };
      img.src = url;
    });
  };

  const extractIconColor = async (iconName) => {
    if (colorCacheRef.current[iconName]) {
      return colorCacheRef.current[iconName];
    }

    if (colorPromiseRef.current[iconName]) {
      return colorPromiseRef.current[iconName];
    }

    const colorPromise = (async () => {
      try {
        const svgText = await loadSvgAsset(iconName);
        const parser = new DOMParser();
        const svgDoc = parser.parseFromString(svgText, 'image/svg+xml');
      
        const elements = svgDoc.querySelectorAll('[style*="fill"]');
        const colors = [];
        
        for (const el of elements) {
          const style = el.getAttribute('style') || '';
          const fillMatch = style.match(/fill:(#[0-9a-fA-F]{6}|#[0-9a-fA-F]{3})/);
          if (fillMatch) {
            colors.push(fillMatch[1]);
          }
        }
        
        if (colors.length === 0) {
          const fillElements = svgDoc.querySelectorAll('[fill]');
          for (const el of fillElements) {
            const fill = el.getAttribute('fill');
            if (fill && fill.startsWith('#')) {
              colors.push(fill);
            }
          }
        }

        if (colors.length === 0) {
          const strokeElements = svgDoc.querySelectorAll('[style*="stroke"], [stroke]');
          for (const el of strokeElements) {
            const style = el.getAttribute('style') || '';
            const strokeAttr = el.getAttribute('stroke') || '';
            const strokeMatch = style.match(/stroke:(#[0-9a-fA-F]{6}|#[0-9a-fA-F]{3})/);
            if (strokeMatch) {
              colors.push(strokeMatch[1]);
              break;
            }
            if (strokeAttr && strokeAttr.startsWith('#')) {
              colors.push(strokeAttr);
              break;
            }
          }
        }

        const result = colors.length > 0 ? colors : ['#800080', '#600060'];
        colorCacheRef.current[iconName] = result;
        return result;
      } catch (error) {
        console.error(`Failed to extract color from ${iconName}:`, error);
        const fallback = ['#ff00ffff', '#800080ff'];
        colorCacheRef.current[iconName] = fallback;
        return fallback;
      }
    })();

    colorPromiseRef.current[iconName] = colorPromise;
    return colorPromise;
  };

  useEffect(() => {
    const container = containerRef.current;
    if (!container || availableIcons.length === 0) return;

    let animationCounter = 0;

    const spawnIcon = async () => {
      const x = Math.random() * 100;
      const y = Math.random() * 100;
      const duration = ANIMATION_DURATION_MIN + Math.random() * (ANIMATION_DURATION_MAX - ANIMATION_DURATION_MIN);
      const animId = `anim-${animationCounter++}`;

      const randomIcon = availableIcons[Math.floor(Math.random() * availableIcons.length)];
      const colors = await extractIconColor(randomIcon);
      const primaryColor = colors[0];

      const keyframes = `
        @keyframes ${animId}-icon {
          0% {
            opacity: 0;
            transform: scale(1);
          }
          30% {
            opacity: 0.50;
            transform: scale(1);
          }
          100% {
            opacity: 0;
            transform: scale(1.2);
          }
        }
        @keyframes ${animId}-gradient {
          0% {
            transform: translate(-50%, -50%) scale(0);
            opacity: 0.35;
          }
          100% {
            transform: translate(-50%, -50%) scale(3);
            opacity: 0;
          }
        }
      `;

      if (styleSheetRef.current && !keyframesAddedRef.current.has(animId)) {
        styleSheetRef.current.textContent += keyframes;
        keyframesAddedRef.current.add(animId);
      }

      const gradient = document.createElement('div');
      gradient.className = 'color-gradient';
      gradient.style.left = x + '%';
      gradient.style.top = y + '%';
      gradient.style.background = `radial-gradient(circle, ${primaryColor}, rgba(0,0,0,0))`;
      gradient.style.animation = `${animId}-gradient ${duration*CIRCLE_FACTOR}ms ease-out forwards`;
      container.appendChild(gradient);

      const iconWrapper = document.createElement('div');
      iconWrapper.className = 'icon-wrapper';
      iconWrapper.style.left = x + '%';
      iconWrapper.style.top = y + '%';

      const cachedImage = imageCacheRef.current[randomIcon];
      let icon;
      if (cachedImage) {
        icon = cachedImage.cloneNode(true);
      } else {
        const loadedImage = await preloadImage(randomIcon);
        icon = loadedImage.cloneNode(true);
      }

      icon.alt = 'element';
      icon.className = 'floating-icon';
      icon.style.animation = `${animId}-icon ${duration}ms ease-out forwards`;
      iconWrapper.appendChild(icon);

      container.appendChild(iconWrapper);

      setTimeout(() => {
        gradient.remove();
        iconWrapper.remove();
      }, duration * CIRCLE_FACTOR);
    };

    const spawnInterval = setInterval(() => {
      spawnIcon();
    }, ICON_SPAWN_INTERVAL);

    return () => clearInterval(spawnInterval);
  }, [availableIcons]);

  return <div ref={containerRef} className="elements-bg-container"></div>;
};

export default ElementsBg;