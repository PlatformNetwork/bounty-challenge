// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { isTauri } from './utils/tauriCheck';

// Only import Tauri APIs when running in Tauri
let getCurrentWebviewWindow: any = null;
if (isTauri()) {
  // Dynamic import to avoid crashing in browser
  import('@tauri-apps/api/webviewWindow').then(mod => {
    getCurrentWebviewWindow = mod.getCurrentWebviewWindow;
  });
}

const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        if (isTauri()) {
          // Tauri-specific initialization
          const appWindow = getCurrentWebviewWindow();
          if (appWindow) {
            // Perform Tauri-specific operations
            await appWindow.setTitle('Cortex');
          }
        }
        
        // Common initialization logic
        setIsInitialized(true);
      } catch (err) {
        console.error('Initialization failed:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return <div className="error-screen">Failed to Initialize: {error}</div>;
  }

  if (!isInitialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return <div className="welcome-ui">Welcome to Cortex!</div>;
};

export default AppCore;
