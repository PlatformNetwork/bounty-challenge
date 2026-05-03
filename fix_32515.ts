// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';

// Dynamic import wrapper to avoid Tauri API calls in browser
const loadTauriModule = async () => {
  try {
    // Check if running in Tauri environment
    if (window.__TAURI_INTERNALS__) {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      return { getCurrentWebviewWindow };
    }
    return null;
  } catch {
    return null;
  }
};

const AppCore: React.FC = () => {
  const [isTauri, setIsTauri] = useState(false);
  const [tauriModule, setTauriModule] = useState<any>(null);

  useEffect(() => {
    const init = async () => {
      const module = await loadTauriModule();
      if (module) {
        setTauriModule(module);
        setIsTauri(true);
      }
    };
    init();
  }, []);

  // Only use Tauri APIs when available
  useEffect(() => {
    if (isTauri && tauriModule) {
      const appWindow = tauriModule.getCurrentWebviewWindow();
      // Your Tauri-specific logic here
    }
  }, [isTauri, tauriModule]);

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <div>
        {isTauri ? (
          // Tauri-specific UI
          <div>Tauri App</div>
        ) : (
          // Browser-specific UI
          <div>Browser App</div>
        )}
      </div>
    </Suspense>
  );
};

export default AppCore;
